//! Capability-Guarded File Descriptor & Handle Layer
//! Compliant with Bible §8.10 (SP-24), REQ-KRN-106 and Invariant INV-VFS-001.

use crate::capability::rights::CapRights;
use crate::capability::table::{CapType, Capability};
use crate::vfs::vnode::VnodeId;

/// In-kernel file handle bound to a capability token (REQ-KRN-106)
#[derive(Clone, Copy, Debug)]
pub struct FileDescriptor {
    pub fd: u32,
    pub vnode_id: VnodeId,
    pub offset: usize,
    pub rights: CapRights,
    pub is_open: bool,
}

impl FileDescriptor {
    pub const fn empty() -> Self {
        Self {
            fd: 0,
            vnode_id: 0,
            offset: 0,
            rights: CapRights::NONE,
            is_open: false,
        }
    }

    pub fn new(fd: u32, vnode_id: VnodeId, rights: CapRights) -> Self {
        Self {
            fd,
            vnode_id,
            offset: 0,
            rights,
            is_open: true,
        }
    }

    /// Validates whether a presented capability token authorizes reading from this file descriptor (INV-VFS-001)
    pub fn verify_read_access(&self, cap: &Capability) -> Result<(), &'static str> {
        if !self.is_open {
            return Err("EBADF: File descriptor is not open");
        }
        if !cap.is_valid {
            return Err("EPERM: Presented capability token is invalid (INV-VFS-001)");
        }
        if cap.cap_type != CapType::File(self.vnode_id) {
            return Err("EPERM: Capability token does not match target file vnode (INV-VFS-001)");
        }
        if !cap.rights.contains(CapRights::READ) {
            return Err("EPERM: Capability token lacks READ rights (INV-VFS-001)");
        }
        Ok(())
    }

    /// Validates whether a presented capability token authorizes writing to this file descriptor (INV-VFS-001)
    pub fn verify_write_access(&self, cap: &Capability) -> Result<(), &'static str> {
        if !self.is_open {
            return Err("EBADF: File descriptor is not open");
        }
        if !cap.is_valid {
            return Err("EPERM: Presented capability token is invalid (INV-VFS-001)");
        }
        if cap.cap_type != CapType::File(self.vnode_id) {
            return Err("EPERM: Capability token does not match target file vnode (INV-VFS-001)");
        }
        if !cap.rights.contains(CapRights::WRITE) {
            return Err("EPERM: Capability token lacks WRITE rights (INV-VFS-001)");
        }
        Ok(())
    }
}
