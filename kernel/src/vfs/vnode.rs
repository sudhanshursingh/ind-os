//! VFS Virtual Node (vnode) Abstraction Layer
//! Compliant with Bible §8.10 (SP-24), REQ-KRN-106 and Invariant INV-VFS-001.

/// Vnode file system object classifications
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VnodeType {
    RegularFile,
    Directory,
    Device,
    Symlink,
}

/// Unique identifier for virtual file system nodes
pub type VnodeId = u64;

/// In-kernel virtual node (vnode) structure
#[derive(Clone, Copy, Debug)]
pub struct Vnode {
    pub id: VnodeId,
    pub vtype: VnodeType,
    pub size: usize,
    pub name: [u8; 32],
    pub name_len: usize,
    pub parent_id: Option<VnodeId>,
    pub data: [u8; 128],
    pub data_len: usize,
    pub is_allocated: bool,
}

impl Vnode {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            vtype: VnodeType::RegularFile,
            size: 0,
            name: [0u8; 32],
            name_len: 0,
            parent_id: None,
            data: [0u8; 128],
            data_len: 0,
            is_allocated: false,
        }
    }

    pub fn new(id: VnodeId, name_str: &str, vtype: VnodeType, parent_id: Option<VnodeId>) -> Self {
        let mut name = [0u8; 32];
        let bytes = name_str.as_bytes();
        let len = if bytes.len() > 32 { 32 } else { bytes.len() };
        let mut i = 0;
        while i < len {
            name[i] = bytes[i];
            i += 1;
        }

        Self {
            id,
            vtype,
            size: 0,
            name,
            name_len: len,
            parent_id,
            data: [0u8; 128],
            data_len: 0,
            is_allocated: true,
        }
    }

    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    pub fn read(&self, offset: usize, buf: &mut [u8]) -> usize {
        if offset >= self.data_len {
            return 0;
        }
        let available = self.data_len - offset;
        let to_read = if buf.len() < available { buf.len() } else { available };
        buf[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
        to_read
    }

    pub fn write(&mut self, offset: usize, buf: &[u8]) -> usize {
        if offset >= 128 {
            return 0;
        }
        let available = 128 - offset;
        let to_write = if buf.len() < available { buf.len() } else { available };
        self.data[offset..offset + to_write].copy_from_slice(&buf[..to_write]);
        if offset + to_write > self.data_len {
            self.data_len = offset + to_write;
            self.size = self.data_len;
        }
        to_write
    }
}
