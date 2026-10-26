//! INDBIN: Ind OS Native Micro-Executable Format & Loader
//! Ratified under INDOS-BIBLE-001 §8.8 and ADR-001.
//!
//! Binary layout:
//! - 80-byte fixed header (IndbinHeader)
//! - Text segment payload
//! - Data segment payload

use crate::mm::address_space::{is_valid_user_address, AddressSpace, USER_SPACE_END};
use crate::mm::frame::{MemoryError, PAGE_SIZE};
use crate::mm::page_table::PageTableFlags;
use crate::mm::FRAME_ALLOCATOR;

pub const INDBIN_MAGIC: [u8; 4] = *b"INDB";
pub const INDBIN_VERSION: u32 = 1;
pub const USER_STACK_TOP: usize = 0x0000_7FFF_FFFF_0000;
pub const DEFAULT_USER_STACK_SIZE: usize = 16 * 1024; // 16 KB (4 pages)

/// 80-Byte Fixed Header for INDBIN executables
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndbinHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub entry_point: u64,
    pub text_vaddr: u64,
    pub text_offset: u64,
    pub text_size: u64,
    pub data_vaddr: u64,
    pub data_offset: u64,
    pub data_size: u64,
    pub stack_size: u64,
    pub flags: u32,
    pub checksum: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoaderError {
    InvalidMagic,
    UnsupportedVersion,
    InvalidHeader,
    AddressOutOfBounds(usize),
    NullTrapViolation(usize),
    SegmentOverlap,
    ImageTruncated,
    Memory(MemoryError),
}

impl From<MemoryError> for LoaderError {
    fn from(err: MemoryError) -> Self {
        LoaderError::Memory(err)
    }
}

/// Validates and parses an INDBIN header from a byte slice
pub fn parse_header(image: &[u8]) -> Result<IndbinHeader, LoaderError> {
    if image.len() < core::mem::size_of::<IndbinHeader>() {
        return Err(LoaderError::ImageTruncated);
    }

    // SAFETY: We checked that image has at least size_of::<IndbinHeader>() bytes.
    let header = unsafe { core::ptr::read_unaligned(image.as_ptr() as *const IndbinHeader) };

    if header.magic != INDBIN_MAGIC {
        return Err(LoaderError::InvalidMagic);
    }

    if header.version != INDBIN_VERSION {
        return Err(LoaderError::UnsupportedVersion);
    }

    // Validate entry point and text segment bounds
    if !is_valid_user_address(header.entry_point as usize) {
        return Err(LoaderError::NullTrapViolation(header.entry_point as usize));
    }

    if !is_valid_user_address(header.text_vaddr as usize) {
        return Err(LoaderError::NullTrapViolation(header.text_vaddr as usize));
    }

    let text_end = (header.text_vaddr + header.text_size) as usize;
    if text_end > USER_SPACE_END {
        return Err(LoaderError::AddressOutOfBounds(text_end));
    }

    // Check file bounds for text
    let text_file_end = (header.text_offset + header.text_size) as usize;
    if text_file_end > image.len() {
        return Err(LoaderError::ImageTruncated);
    }

    // Validate data segment if present
    if header.data_size > 0 {
        if !is_valid_user_address(header.data_vaddr as usize) {
            return Err(LoaderError::NullTrapViolation(header.data_vaddr as usize));
        }
        let data_end = (header.data_vaddr + header.data_size) as usize;
        if data_end > USER_SPACE_END {
            return Err(LoaderError::AddressOutOfBounds(data_end));
        }
        let data_file_end = (header.data_offset + header.data_size) as usize;
        if data_file_end > image.len() {
            return Err(LoaderError::ImageTruncated);
        }

        // Check for segment overlap
        if (header.text_vaddr as usize) < data_end && text_end > (header.data_vaddr as usize) {
            return Err(LoaderError::SegmentOverlap);
        }
    }

    Ok(header)
}

/// Loads an INDBIN executable image into a per-process address space
/// Allocates physical frames, copies segments, enforces permissions (INV-MEM-002),
/// and maps the user stack. Returns (entry_point, stack_top).
pub fn load_indbin(
    address_space: &mut AddressSpace,
    image: &[u8],
) -> Result<(usize, usize), LoaderError> {
    let header = parse_header(image)?;

    // 1. Load Text Segment (USER_CODE permissions: RO, UXN=0, PXN=1)
    let text_pages = (header.text_size as usize + PAGE_SIZE - 1) / PAGE_SIZE;
    let mut file_offset = header.text_offset as usize;
    let mut vaddr = header.text_vaddr as usize;

    for _ in 0..text_pages {
        let mut allocator = FRAME_ALLOCATOR.lock();
        let frame = allocator.alloc_frame()?;
        drop(allocator);

        let phys = frame.addr();
        let frame_ptr = phys as *mut u8;

        // SAFETY: frame_ptr points to a newly allocated, owned physical page frame of PAGE_SIZE bytes.
        unsafe {
            core::ptr::write_bytes(frame_ptr, 0, PAGE_SIZE);
            let bytes_to_copy = core::cmp::min(
                PAGE_SIZE,
                (header.text_offset as usize + header.text_size as usize).saturating_sub(file_offset),
            );
            if bytes_to_copy > 0 {
                core::ptr::copy_nonoverlapping(image.as_ptr().add(file_offset), frame_ptr, bytes_to_copy);
                file_offset += bytes_to_copy;
            }
        }

        address_space.map_user(vaddr, phys, PageTableFlags::USER_CODE)?;
        vaddr += PAGE_SIZE;
    }

    // 2. Load Data Segment if present (USER_DATA permissions: RW, UXN=1, PXN=1)
    if header.data_size > 0 {
        let data_pages = (header.data_size as usize + PAGE_SIZE - 1) / PAGE_SIZE;
        let mut data_file_offset = header.data_offset as usize;
        let mut data_vaddr = header.data_vaddr as usize;

        for _ in 0..data_pages {
            let mut allocator = FRAME_ALLOCATOR.lock();
            let frame = allocator.alloc_frame()?;
            drop(allocator);

            let phys = frame.addr();
            let frame_ptr = phys as *mut u8;

            // SAFETY: frame_ptr points to a newly allocated, owned physical page frame of PAGE_SIZE bytes.
            unsafe {
                core::ptr::write_bytes(frame_ptr, 0, PAGE_SIZE);
                let bytes_to_copy = core::cmp::min(
                    PAGE_SIZE,
                    (header.data_offset as usize + header.data_size as usize).saturating_sub(data_file_offset),
                );
                if bytes_to_copy > 0 {
                    core::ptr::copy_nonoverlapping(image.as_ptr().add(data_file_offset), frame_ptr, bytes_to_copy);
                    data_file_offset += bytes_to_copy;
                }
            }

            address_space.map_user(data_vaddr, phys, PageTableFlags::USER_DATA)?;
            data_vaddr += PAGE_SIZE;
        }
    }

    // 3. Allocate and map User Stack (anchored at USER_STACK_TOP, growing downwards)
    let stack_size = if header.stack_size > 0 {
        (header.stack_size as usize + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE
    } else {
        DEFAULT_USER_STACK_SIZE
    };
    let stack_pages = stack_size / PAGE_SIZE;
    let stack_bottom = USER_STACK_TOP - stack_size;

    for i in 0..stack_pages {
        let page_vaddr = stack_bottom + (i * PAGE_SIZE);
        let mut allocator = FRAME_ALLOCATOR.lock();
        let frame = allocator.alloc_frame()?;
        drop(allocator);

        let phys = frame.addr();
        let frame_ptr = phys as *mut u8;
        // SAFETY: frame_ptr points to a newly allocated, owned physical page frame of PAGE_SIZE bytes.
        unsafe {
            core::ptr::write_bytes(frame_ptr, 0, PAGE_SIZE);
        }

        address_space.map_user(page_vaddr, phys, PageTableFlags::USER_DATA)?;
    }

    Ok((header.entry_point as usize, USER_STACK_TOP))
}

/// Helper to build an in-memory INDBIN executable image
pub fn build_indbin_image(
    entry_point: usize,
    text_vaddr: usize,
    text_payload: &[u8],
    data_vaddr: usize,
    data_payload: &[u8],
    stack_size: usize,
) -> alloc::vec::Vec<u8> {
    use alloc::vec::Vec;
    let header_size = core::mem::size_of::<IndbinHeader>();
    let text_offset = header_size;
    let data_offset = text_offset + text_payload.len();
    let total_size = data_offset + data_payload.len();

    let mut image = Vec::with_capacity(total_size);
    let header = IndbinHeader {
        magic: INDBIN_MAGIC,
        version: INDBIN_VERSION,
        entry_point: entry_point as u64,
        text_vaddr: text_vaddr as u64,
        text_offset: text_offset as u64,
        text_size: text_payload.len() as u64,
        data_vaddr: data_vaddr as u64,
        data_offset: data_offset as u64,
        data_size: data_payload.len() as u64,
        stack_size: stack_size as u64,
        flags: 1, // AArch64
        checksum: 0,
    };

    // SAFETY: IndbinHeader is repr(C) and contains Plain Old Data.
    let header_slice = unsafe {
        core::slice::from_raw_parts(&header as *const _ as *const u8, header_size)
    };
    image.extend_from_slice(header_slice);
    image.extend_from_slice(text_payload);
    image.extend_from_slice(data_payload);
    image
}

/// Embedded AArch64 machine code for first user-mode "Hello" process
/// Executes:
///   1. SYS_WRITE(1, "Hello from Ind OS User Mode (EL0)!\n", 35) via SVC #0
///   2. SYS_EXIT(0) via SVC #0
pub const HELLO_USER_CODE: [u8; 71] = [
    0x28, 0x00, 0x80, 0xd2, // mov x8, #1 (SYS_WRITE)
    0x20, 0x00, 0x80, 0xd2, // mov x0, #1 (stdout)
    0xe1, 0x00, 0x00, 0x10, // adr x1, msg
    0x62, 0x04, 0x80, 0xd2, // mov x2, #35 (length)
    0x01, 0x00, 0x00, 0xd4, // svc #0
    0x48, 0x00, 0x80, 0xd2, // mov x8, #2 (SYS_EXIT)
    0x00, 0x00, 0x80, 0xd2, // mov x0, #0 (status = 0)
    0x01, 0x00, 0x00, 0xd4, // svc #0
    0x00, 0x00, 0x00, 0x14, // b .
    b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r', b'o', b'm', b' ', b'I', b'n', b'd', b' ',
    b'O', b'S', b' ', b'U', b's', b'e', b'r', b' ', b'M', b'o', b'd', b'e', b' ', b'(', b'E',
    b'L', b'0', b')', b'!', b'\n',
];

extern "C" {
    static user_task_alpha_start: u8;
    static user_task_alpha_end: u8;
    static user_task_beta_start: u8;
    static user_task_beta_end: u8;
}

/// Returns the embedded machine code slice for test user process Alpha
pub fn get_alpha_user_code() -> &'static [u8] {
    let start = core::ptr::addr_of!(user_task_alpha_start) as usize;
    let end = core::ptr::addr_of!(user_task_alpha_end) as usize;
    let len = end.saturating_sub(start);
    // SAFETY: user_task_alpha is placed in .rodata by user_trap.s with valid start/end bounds.
    unsafe { core::slice::from_raw_parts(start as *const u8, len) }
}

/// Returns the embedded machine code slice for test user process Beta
pub fn get_beta_user_code() -> &'static [u8] {
    let start = core::ptr::addr_of!(user_task_beta_start) as usize;
    let end = core::ptr::addr_of!(user_task_beta_end) as usize;
    let len = end.saturating_sub(start);
    // SAFETY: user_task_beta is placed in .rodata by user_trap.s with valid start/end bounds.
    unsafe { core::slice::from_raw_parts(start as *const u8, len) }
}

