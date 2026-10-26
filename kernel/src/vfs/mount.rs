//! VFS Mount Table & Filesystem Registration
//! Compliant with Bible §8.10 (SP-24) and Invariant INV-VFS-002.

use crate::vfs::vnode::VnodeId;

/// Maximum number of mounted filesystems in kernel
pub const MAX_MOUNTS: usize = 8;

/// Mount table entry linking path prefix to root vnode
#[derive(Clone, Copy, Debug)]
pub struct MountEntry {
    pub prefix: [u8; 32],
    pub prefix_len: usize,
    pub root_vnode_id: VnodeId,
    pub is_mounted: bool,
}

impl MountEntry {
    pub const fn empty() -> Self {
        Self {
            prefix: [0u8; 32],
            prefix_len: 0,
            root_vnode_id: 0,
            is_mounted: false,
        }
    }

    pub fn prefix(&self) -> &str {
        core::str::from_utf8(&self.prefix[..self.prefix_len]).unwrap_or("")
    }
}

/// Global kernel VFS mount table
pub struct MountTable {
    entries: [MountEntry; MAX_MOUNTS],
    count: usize,
}

impl MountTable {
    pub const fn new() -> Self {
        Self {
            entries: [MountEntry::empty(); MAX_MOUNTS],
            count: 0,
        }
    }

    /// Mounts a filesystem root vnode at a specific prefix
    pub fn mount(&mut self, prefix_str: &str, root_vnode_id: VnodeId) -> Result<(), &'static str> {
        if self.count >= MAX_MOUNTS {
            return Err("Mount table capacity exceeded");
        }

        // Check if prefix already mounted
        for i in 0..self.count {
            if self.entries[i].is_mounted && self.entries[i].prefix() == prefix_str {
                return Err("Mount prefix already exists");
            }
        }

        let bytes = prefix_str.as_bytes();
        let len = if bytes.len() > 32 { 32 } else { bytes.len() };
        let mut prefix = [0u8; 32];
        let mut i = 0;
        while i < len {
            prefix[i] = bytes[i];
            i += 1;
        }

        self.entries[self.count] = MountEntry {
            prefix,
            prefix_len: len,
            root_vnode_id,
            is_mounted: true,
        };
        self.count += 1;

        Ok(())
    }

    /// Unmounts a mounted prefix
    pub fn unmount(&mut self, prefix_str: &str) -> Result<(), &'static str> {
        for i in 0..self.count {
            if self.entries[i].is_mounted && self.entries[i].prefix() == prefix_str {
                self.entries[i].is_mounted = false;
                return Ok(());
            }
        }
        Err("Mount prefix not found")
    }

    /// Finds longest matching mount point prefix for a given path
    pub fn find_mount(&self, path: &str) -> Option<(VnodeId, usize)> {
        let mut best_match: Option<(VnodeId, usize)> = None;
        let mut best_len = 0;

        for i in 0..self.count {
            if !self.entries[i].is_mounted {
                continue;
            }
            let pref = self.entries[i].prefix();
            if path.starts_with(pref) && (pref.len() > best_len || best_match.is_none()) {
                best_len = pref.len();
                best_match = Some((self.entries[i].root_vnode_id, best_len));
            }
        }

        best_match
    }

    pub fn len(&self) -> usize {
        self.count
    }
}
