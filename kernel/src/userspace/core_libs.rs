//! Kernel-Side Core Userspace Library Invariant Verifier
//!
//! Constitutional References:
//! - REQ-USE-202: Core libraries: alloc, collections, IPC bindings with capability-first ergonomics
//! - INV-LIB-001: Safe Bounded no_std Allocation & Collections
//! - INV-LIB-002: Capability-First IPC Ergonomics with Zero Ambient Authority
//! - REQ-ARCH-101: Independent crate compilation (no userspace crate dependencies)
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const TEST_HEAP_SIZE: usize = 16384; // 16 KiB test heap

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
    InvalidAlignment,
}

pub struct KernelUserHeap {
    buffer: [u8; TEST_HEAP_SIZE],
    allocated: usize,
}

impl KernelUserHeap {
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; TEST_HEAP_SIZE],
            allocated: 0,
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> Result<usize, AllocError> {
        if align == 0 || (align & (align - 1)) != 0 {
            return Err(AllocError::InvalidAlignment);
        }
        let current = self.allocated;
        let aligned = (current + (align - 1)) & !(align - 1);
        let total = (aligned - current) + size;

        if self.allocated + total > TEST_HEAP_SIZE {
            return Err(AllocError::OutOfMemory);
        }

        self.allocated += total;
        Ok(aligned)
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn reset(&mut self) {
        self.allocated = 0;
    }
}

pub const CAP_RIGHTS_READ: u32 = 0x01;
pub const CAP_RIGHTS_WRITE: u32 = 0x02;
pub const CAP_RIGHTS_DELEGATE: u32 = 0x04;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    PermissionDenied,
    BufferOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelCapToken {
    handle: u32,
    rights: u32,
}

impl KernelCapToken {
    pub const fn new(handle: u32, rights: u32) -> Self {
        Self { handle, rights }
    }

    pub fn rights(&self) -> u32 {
        self.rights
    }

    pub fn can_write(&self) -> bool {
        (self.rights & CAP_RIGHTS_WRITE) != 0
    }

    pub fn can_read(&self) -> bool {
        (self.rights & CAP_RIGHTS_READ) != 0
    }

    pub fn attenuate(&self, mask: u32) -> Self {
        Self {
            handle: self.handle,
            rights: self.rights & mask,
        }
    }
}

pub struct KernelIpcClient {
    endpoint_id: u32,
    cap: KernelCapToken,
}

impl KernelIpcClient {
    pub const fn new(endpoint_id: u32, cap: KernelCapToken) -> Self {
        Self { endpoint_id, cap }
    }

    pub fn endpoint_id(&self) -> u32 {
        self.endpoint_id
    }

    pub fn call(&self, msg_id: u32) -> Result<u32, IpcError> {
        if !self.cap.can_write() || !self.cap.can_read() {
            return Err(IpcError::PermissionDenied);
        }
        Ok(msg_id + 1)
    }
}
