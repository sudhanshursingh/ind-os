//! Pure no_std Devicetree Blob (DTB / FDT) Parser for IndK
//! Conforms to Devicetree Specification v0.4 without external dependencies (Rule N-01).

const FDT_MAGIC: u32 = 0xd00dfeed;

// FDT Structure Block Tokens
const FDT_BEGIN_NODE: u32 = 0x00000001;
const FDT_END_NODE: u32   = 0x00000002;
const FDT_PROP: u32       = 0x00000003;
const FDT_NOP: u32        = 0x00000004;
const FDT_END: u32        = 0x00000009;

#[derive(Debug, Clone, Copy)]
pub struct PhysicalMemoryRegion {
    pub base: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum DtbError {
    InvalidMagic(u32),
    CorruptedHeader,
    StructureMalformed,
    MemoryNodeNotFound,
}

/// Minimal Devicetree Blob reader
pub struct DtbParser {
    base_ptr: *const u8,
    totalsize: usize,
    struct_offset: usize,
    strings_offset: usize,
}

impl DtbParser {
    /// Construct a DTB parser from a raw physical memory pointer
    pub unsafe fn from_raw(dtb_ptr: usize) -> Result<Self, DtbError> {
        let ptr = dtb_ptr as *const u8;
        let magic = u32::from_be(*(ptr as *const u32));
        if magic != FDT_MAGIC {
            return Err(DtbError::InvalidMagic(magic));
        }

        let totalsize = u32::from_be(*(ptr.add(4) as *const u32)) as usize;
        let struct_offset = u32::from_be(*(ptr.add(8) as *const u32)) as usize;
        let strings_offset = u32::from_be(*(ptr.add(12) as *const u32)) as usize;

        if struct_offset >= totalsize || strings_offset >= totalsize {
            return Err(DtbError::CorruptedHeader);
        }

        Ok(Self {
            base_ptr: ptr,
            totalsize,
            struct_offset,
            strings_offset,
        })
    }

    pub fn total_size(&self) -> usize {
        self.totalsize
    }

    /// Read a string from the DTB strings block at the specified offset
    unsafe fn get_string(&self, name_offset: usize) -> &str {
        let start = self.base_ptr.add(self.strings_offset + name_offset);
        let mut len = 0;
        while *start.add(len) != 0 {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(start, len);
        core::str::from_utf8(slice).unwrap_or("")
    }

    /// Discover physical RAM bounds by parsing the `/memory` node
    pub fn discover_memory(&self) -> Result<PhysicalMemoryRegion, DtbError> {
        unsafe {
            let mut cursor = self.struct_offset;
            let mut is_memory_node = false;
            let mut found_region: Option<PhysicalMemoryRegion> = None;

            while cursor + 4 <= self.totalsize {
                let token = u32::from_be(core::ptr::read_unaligned(self.base_ptr.add(cursor) as *const u32));
                cursor += 4;

                match token {
                    FDT_BEGIN_NODE => {
                        // Node name is null-terminated string, padded to 4 bytes
                        let name_start = self.base_ptr.add(cursor);
                        let mut name_len = 0;
                        while *name_start.add(name_len) != 0 {
                            name_len += 1;
                        }
                        let name_slice = core::slice::from_raw_parts(name_start, name_len);
                        let name = core::str::from_utf8(name_slice).unwrap_or("");
                        cursor += (name_len + 1 + 3) & !3; // Align to 4 bytes

                        if name.starts_with("memory@") || name == "memory" {
                            is_memory_node = true;
                        } else if !name.is_empty() {
                            is_memory_node = false;
                        }
                    }
                    FDT_END_NODE => {
                        if is_memory_node && found_region.is_some() {
                            return Ok(found_region.unwrap());
                        }
                        is_memory_node = false;
                    }
                    FDT_PROP => {
                        let prop_len = u32::from_be(core::ptr::read_unaligned(self.base_ptr.add(cursor) as *const u32)) as usize;
                        let name_offset = u32::from_be(core::ptr::read_unaligned(self.base_ptr.add(cursor + 4) as *const u32)) as usize;
                        cursor += 8;

                        let prop_name = self.get_string(name_offset);
                        let val_ptr = self.base_ptr.add(cursor);
                        cursor += (prop_len + 3) & !3; // Align to 4 bytes

                        if is_memory_node {
                            if prop_name == "device_type" {
                                let dev_type = core::str::from_utf8(core::slice::from_raw_parts(val_ptr, prop_len.saturating_sub(1))).unwrap_or("");
                                if dev_type == "memory" {
                                    is_memory_node = true;
                                }
                            } else if prop_name == "reg" && prop_len >= 16 {
                                // In QEMU virt AArch64: #address-cells = 2 (8 bytes), #size-cells = 2 (8 bytes)
                                let base = u64::from_be(core::ptr::read_unaligned(val_ptr as *const u64)) as usize;
                                let size = u64::from_be(core::ptr::read_unaligned(val_ptr.add(8) as *const u64)) as usize;
                                found_region = Some(PhysicalMemoryRegion { base, size });
                            }
                        }
                    }
                    FDT_NOP => {}
                    FDT_END => break,
                    _ => return Err(DtbError::StructureMalformed),
                }
            }

            found_region.ok_or(DtbError::MemoryNodeNotFound)
        }
    }
}
