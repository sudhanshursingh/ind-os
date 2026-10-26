//! VFS Core State Machine & Capability-Mediated File Operations
//! Compliant with Bible §8.10 (SP-24), REQ-KRN-106, INV-VFS-001, and INV-VFS-002.

use crate::capability::rights::CapRights;
use crate::capability::table::{CapType, Capability};
use crate::vfs::file::FileDescriptor;
use crate::vfs::mount::MountTable;
use crate::vfs::path::PathResolver;
use crate::vfs::vnode::{Vnode, VnodeId, VnodeType};

pub const MAX_VNODES: usize = 64;
pub const MAX_OPEN_FILES: usize = 32;

/// Core Virtual File System state manager
pub struct Vfs {
    vnodes: [Vnode; MAX_VNODES],
    vnode_count: usize,
    mount_table: MountTable,
    open_files: [FileDescriptor; MAX_OPEN_FILES],
    open_count: usize,
    next_vnode_id: VnodeId,
    next_fd: u32,
}

impl Vfs {
    pub const fn new() -> Self {
        Self {
            vnodes: [Vnode::empty(); MAX_VNODES],
            vnode_count: 0,
            mount_table: MountTable::new(),
            open_files: [FileDescriptor::empty(); MAX_OPEN_FILES],
            open_count: 0,
            next_vnode_id: 1,
            next_fd: 10,
        }
    }

    /// Initializes early root filesystem hierarchy
    pub fn init(&mut self) {
        if self.vnode_count > 0 {
            return;
        }

        // 1. Root directory (vnode 1)
        let root_id = self.alloc_vnode("/", VnodeType::Directory, None);
        let _ = self.mount_table.mount("/", root_id);

        // 2. /dev directory (vnode 2)
        let dev_id = self.alloc_vnode("dev", VnodeType::Directory, Some(root_id));

        // 3. /dev/null device (vnode 3)
        let _null_id = self.alloc_vnode("null", VnodeType::Device, Some(dev_id));

        // 4. /etc directory (vnode 4)
        let etc_id = self.alloc_vnode("etc", VnodeType::Directory, Some(root_id));

        // 5. /etc/version file (vnode 5) with sample kernel version text
        let ver_id = self.alloc_vnode("version", VnodeType::RegularFile, Some(etc_id));
        let ver_node = self.get_vnode_mut(ver_id).unwrap();
        ver_node.write(0, b"Ind OS Microkernel v0.4.0 (Phase 4)");
    }

    fn alloc_vnode(&mut self, name: &str, vtype: VnodeType, parent: Option<VnodeId>) -> VnodeId {
        let id = self.next_vnode_id;
        self.next_vnode_id += 1;

        let node = Vnode::new(id, name, vtype, parent);
        self.vnodes[self.vnode_count] = node;
        self.vnode_count += 1;

        id
    }

    pub fn get_vnode(&self, id: VnodeId) -> Option<&Vnode> {
        for i in 0..self.vnode_count {
            if self.vnodes[i].id == id && self.vnodes[i].is_allocated {
                return Some(&self.vnodes[i]);
            }
        }
        None
    }

    pub fn get_vnode_mut(&mut self, id: VnodeId) -> Option<&mut Vnode> {
        for i in 0..self.vnode_count {
            if self.vnodes[i].id == id && self.vnodes[i].is_allocated {
                return Some(&mut self.vnodes[i]);
            }
        }
        None
    }

    /// Hierarchical path lookup resolving through mounted root and directories (INV-VFS-002)
    pub fn lookup(&self, path: &str) -> Result<VnodeId, &'static str> {
        let mut canon_buf = [0u8; 64];
        let canon_path = PathResolver::canonicalize(path, &mut canon_buf)?;

        if canon_path == "/" {
            return Ok(1); // Root vnode ID
        }

        // Iteratively traverse path parts starting from root
        let mut current_id = 1;

        let bytes = canon_path.as_bytes();
        let mut start = 0;

        while start < bytes.len() {
            while start < bytes.len() && bytes[start] == b'/' {
                start += 1;
            }
            if start >= bytes.len() {
                break;
            }

            let mut end = start;
            while end < bytes.len() && bytes[end] != b'/' {
                end += 1;
            }

            let part = core::str::from_utf8(&bytes[start..end]).map_err(|_| "Invalid UTF-8")?;
            start = end;

            // Search children of current_id
            let mut found = None;
            for i in 0..self.vnode_count {
                let node = &self.vnodes[i];
                if node.is_allocated && node.parent_id == Some(current_id) && node.name() == part {
                    found = Some(node.id);
                    break;
                }
            }

            match found {
                Some(next_id) => current_id = next_id,
                None => return Err("ENOENT: Path component not found"),
            }
        }

        Ok(current_id)
    }

    /// Creates a new regular file under an existing directory
    pub fn create_file(&mut self, dir_path: &str, name: &str) -> Result<VnodeId, &'static str> {
        if self.vnode_count >= MAX_VNODES {
            return Err("Vnode pool exhausted");
        }
        let parent_id = self.lookup(dir_path)?;
        let parent = self.get_vnode(parent_id).ok_or("Parent not found")?;
        if parent.vtype != VnodeType::Directory {
            return Err("ENOTDIR: Parent is not a directory");
        }

        // Verify child does not already exist
        for i in 0..self.vnode_count {
            let node = &self.vnodes[i];
            if node.is_allocated && node.parent_id == Some(parent_id) && node.name() == name {
                return Err("EEXIST: File already exists");
            }
        }

        let id = self.alloc_vnode(name, VnodeType::RegularFile, Some(parent_id));
        Ok(id)
    }

    /// Opens a file and mints a capability-bound file descriptor (REQ-KRN-106, INV-VFS-001)
    pub fn open_file(&mut self, path: &str, requested_rights: CapRights) -> Result<(u32, Capability), &'static str> {
        if self.open_count >= MAX_OPEN_FILES {
            return Err("EMFILE: Open file descriptor limit reached");
        }

        let vnode_id = self.lookup(path)?;

        let fd = self.next_fd;
        self.next_fd += 1;

        let descriptor = FileDescriptor::new(fd, vnode_id, requested_rights);
        self.open_files[self.open_count] = descriptor;
        self.open_count += 1;

        // Mint capability token bound to the vnode and requested rights
        let cap = Capability::new(CapType::File(vnode_id), requested_rights, 0xF11E);

        Ok((fd, cap))
    }

    /// Capability-mediated file read (REQ-KRN-106, INV-VFS-001)
    pub fn read_file(&mut self, fd: u32, cap: &Capability, buf: &mut [u8]) -> Result<usize, &'static str> {
        let mut desc_idx = None;
        for i in 0..self.open_count {
            if self.open_files[i].is_open && self.open_files[i].fd == fd {
                desc_idx = Some(i);
                break;
            }
        }

        let idx = desc_idx.ok_or("EBADF: Bad file descriptor")?;
        self.open_files[idx].verify_read_access(cap)?;

        let vnode_id = self.open_files[idx].vnode_id;
        let offset = self.open_files[idx].offset;

        let node = self.get_vnode(vnode_id).ok_or("ENOENT: Vnode missing")?;
        let bytes_read = node.read(offset, buf);
        self.open_files[idx].offset += bytes_read;

        Ok(bytes_read)
    }

    /// Capability-mediated file write (REQ-KRN-106, INV-VFS-001)
    pub fn write_file(&mut self, fd: u32, cap: &Capability, buf: &[u8]) -> Result<usize, &'static str> {
        let mut desc_idx = None;
        for i in 0..self.open_count {
            if self.open_files[i].is_open && self.open_files[i].fd == fd {
                desc_idx = Some(i);
                break;
            }
        }

        let idx = desc_idx.ok_or("EBADF: Bad file descriptor")?;
        self.open_files[idx].verify_write_access(cap)?;

        let vnode_id = self.open_files[idx].vnode_id;
        let offset = self.open_files[idx].offset;

        let node = self.get_vnode_mut(vnode_id).ok_or("ENOENT: Vnode missing")?;
        let bytes_written = node.write(offset, buf);
        self.open_files[idx].offset += bytes_written;

        Ok(bytes_written)
    }

    /// Seeks the file offset of an open file descriptor
    pub fn seek(&mut self, fd: u32, offset: usize) -> Result<(), &'static str> {
        for i in 0..self.open_count {
            if self.open_files[i].is_open && self.open_files[i].fd == fd {
                self.open_files[i].offset = offset;
                return Ok(());
            }
        }
        Err("EBADF: Bad file descriptor")
    }

    /// Closes an open file descriptor
    pub fn close_file(&mut self, fd: u32) -> Result<(), &'static str> {
        for i in 0..self.open_count {
            if self.open_files[i].is_open && self.open_files[i].fd == fd {
                self.open_files[i].is_open = false;
                return Ok(());
            }
        }
        Err("EBADF: Bad file descriptor")
    }

    pub fn vnode_count(&self) -> usize {
        self.vnode_count
    }

    pub fn open_count(&self) -> usize {
        self.open_count
    }
}
