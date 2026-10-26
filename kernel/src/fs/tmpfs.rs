//! Volatile In-Memory Temporary File System (tmpfs)
//! Compliant with Bible §8.10 (SP-25), Rule N-01, Rule N-04.

pub const MAX_TMPFS_NODES: usize = 32;
pub const MAX_FILE_PAYLOAD: usize = 512;

/// In-memory node representing a file or directory in tmpfs
#[derive(Clone, Copy)]
pub struct TmpFsNode {
    pub id: u32,
    pub name: [u8; 32],
    pub name_len: usize,
    pub is_dir: bool,
    pub parent_id: Option<u32>,
    pub data: [u8; MAX_FILE_PAYLOAD],
    pub data_len: usize,
    pub is_allocated: bool,
}

impl TmpFsNode {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            name: [0u8; 32],
            name_len: 0,
            is_dir: false,
            parent_id: None,
            data: [0u8; MAX_FILE_PAYLOAD],
            data_len: 0,
            is_allocated: false,
        }
    }

    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

/// Volatile in-memory filesystem instance mounted at /tmp
pub struct TmpFs {
    nodes: [TmpFsNode; MAX_TMPFS_NODES],
    node_count: usize,
    next_id: u32,
}

impl TmpFs {
    pub const fn new() -> Self {
        Self {
            nodes: [TmpFsNode::empty(); MAX_TMPFS_NODES],
            node_count: 0,
            next_id: 1,
        }
    }

    /// Initializes tmpfs with a root directory node (id 1)
    pub fn init(&mut self) {
        if self.node_count > 0 {
            return;
        }

        let mut root = TmpFsNode::empty();
        root.id = self.next_id;
        self.next_id += 1;
        root.name[0] = b'/';
        root.name_len = 1;
        root.is_dir = true;
        root.parent_id = None;
        root.is_allocated = true;

        self.nodes[0] = root;
        self.node_count = 1;
    }

    /// Creates a regular file under parent_id
    pub fn create_file(&mut self, parent_id: u32, name: &str) -> Result<u32, &'static str> {
        if self.node_count >= MAX_TMPFS_NODES {
            return Err("ENOSPC: Tmpfs node capacity exhausted");
        }
        if self.lookup(parent_id, name).is_some() {
            return Err("EEXIST: File already exists in tmpfs");
        }

        let id = self.next_id;
        self.next_id += 1;

        let mut node = TmpFsNode::empty();
        node.id = id;
        node.is_dir = false;
        node.parent_id = Some(parent_id);
        node.is_allocated = true;

        let bytes = name.as_bytes();
        let len = if bytes.len() > 32 { 32 } else { bytes.len() };
        node.name[..len].copy_from_slice(&bytes[..len]);
        node.name_len = len;

        self.nodes[self.node_count] = node;
        self.node_count += 1;

        Ok(id)
    }

    /// Creates a subdirectory under parent_id
    pub fn create_dir(&mut self, parent_id: u32, name: &str) -> Result<u32, &'static str> {
        if self.node_count >= MAX_TMPFS_NODES {
            return Err("ENOSPC: Tmpfs node capacity exhausted");
        }
        if self.lookup(parent_id, name).is_some() {
            return Err("EEXIST: Directory already exists in tmpfs");
        }

        let id = self.next_id;
        self.next_id += 1;

        let mut node = TmpFsNode::empty();
        node.id = id;
        node.is_dir = true;
        node.parent_id = Some(parent_id);
        node.is_allocated = true;

        let bytes = name.as_bytes();
        let len = if bytes.len() > 32 { 32 } else { bytes.len() };
        node.name[..len].copy_from_slice(&bytes[..len]);
        node.name_len = len;

        self.nodes[self.node_count] = node;
        self.node_count += 1;

        Ok(id)
    }

    /// Looks up a child name under parent_id
    pub fn lookup(&self, parent_id: u32, name: &str) -> Option<u32> {
        for i in 0..self.node_count {
            let n = &self.nodes[i];
            if n.is_allocated && n.parent_id == Some(parent_id) && n.name() == name {
                return Some(n.id);
            }
        }
        None
    }

    /// Reads data from a regular file node
    pub fn read_file(&self, node_id: u32, offset: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
        for i in 0..self.node_count {
            let n = &self.nodes[i];
            if n.is_allocated && n.id == node_id {
                if n.is_dir {
                    return Err("EISDIR: Cannot read directory as file");
                }
                if offset >= n.data_len {
                    return Ok(0);
                }
                let avail = n.data_len - offset;
                let to_read = if buf.len() < avail { buf.len() } else { avail };
                buf[..to_read].copy_from_slice(&n.data[offset..offset + to_read]);
                return Ok(to_read);
            }
        }
        Err("ENOENT: Node not found in tmpfs")
    }

    /// Writes data to a regular file node
    pub fn write_file(&mut self, node_id: u32, offset: usize, data: &[u8]) -> Result<usize, &'static str> {
        for i in 0..self.node_count {
            let n = &mut self.nodes[i];
            if n.is_allocated && n.id == node_id {
                if n.is_dir {
                    return Err("EISDIR: Cannot write to directory as file");
                }
                if offset >= MAX_FILE_PAYLOAD {
                    return Err("EFBIG: File offset exceeds maximum tmpfs file size");
                }
                let avail = MAX_FILE_PAYLOAD - offset;
                let to_write = if data.len() < avail { data.len() } else { avail };
                n.data[offset..offset + to_write].copy_from_slice(&data[..to_write]);
                if offset + to_write > n.data_len {
                    n.data_len = offset + to_write;
                }
                return Ok(to_write);
            }
        }
        Err("ENOENT: Node not found in tmpfs")
    }

    /// Unlinks / deletes a node by name from parent_id
    pub fn unlink(&mut self, parent_id: u32, name: &str) -> Result<(), &'static str> {
        for i in 0..self.node_count {
            let n = &mut self.nodes[i];
            if n.is_allocated && n.parent_id == Some(parent_id) && n.name() == name {
                n.is_allocated = false;
                return Ok(());
            }
        }
        Err("ENOENT: File not found in tmpfs")
    }

    pub fn node_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_allocated).count()
    }
}
