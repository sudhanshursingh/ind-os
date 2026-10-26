//! IndFS Core Implementation: Superblock, Inode Management, and Checksummed Storage
//! Compliant with Bible §8.10 (SP-25), INDOS-SPEC-INDFS-01, Rule N-01, Rule N-04, and Invariant INV-FS-001.

use crate::fs::block::{MemBlockDevice, BLOCK_SIZE, TOTAL_DEVICE_BLOCKS};
use crate::fs::crc32::crc32c;
use crate::fs::journal::{Journal, OP_WRITE_BLOCK, OP_WRITE_INODE};

pub const INDFS_MAGIC: [u8; 8] = *b"INDFS01\0";
pub const INDFS_VERSION: u32 = 1;

pub const SUPERBLOCK_BLOCK_NUM: u64 = 1;
pub const JOURNAL_START_BLOCK: u64 = 3;
pub const JOURNAL_BLOCK_COUNT: u32 = 8;
pub const INODE_TABLE_BLOCK: u64 = 12;
pub const CHECKSUM_TABLE_BLOCK: u64 = 13;
pub const DATA_START_BLOCK: u64 = 14;
pub const MAX_INODES: usize = 16;

/// On-disk IndFS Superblock structure
#[derive(Clone, Copy, Debug)]
pub struct Superblock {
    pub magic: [u8; 8],
    pub version: u32,
    pub block_size: u32,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub total_inodes: u32,
    pub free_inodes: u32,
    pub journal_start_block: u64,
    pub journal_block_count: u32,
    pub inode_table_block: u64,
    pub root_inode: u32,
    pub clean_unmount: u8,
    pub crc32c: u32,
}

impl Superblock {
    pub fn new(total_blocks: u64) -> Self {
        let mut sb = Self {
            magic: INDFS_MAGIC,
            version: INDFS_VERSION,
            block_size: BLOCK_SIZE as u32,
            total_blocks,
            free_blocks: total_blocks.saturating_sub(DATA_START_BLOCK),
            total_inodes: MAX_INODES as u32,
            free_inodes: (MAX_INODES - 1) as u32,
            journal_start_block: JOURNAL_START_BLOCK,
            journal_block_count: JOURNAL_BLOCK_COUNT,
            inode_table_block: INODE_TABLE_BLOCK,
            root_inode: 1,
            clean_unmount: 1,
            crc32c: 0,
        };
        sb.crc32c = sb.compute_checksum();
        sb
    }

    pub fn compute_checksum(&self) -> u32 {
        let mut buf = [0u8; 64];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.version.to_le_bytes());
        buf[12..16].copy_from_slice(&self.block_size.to_le_bytes());
        buf[16..24].copy_from_slice(&self.total_blocks.to_le_bytes());
        buf[24..32].copy_from_slice(&self.free_blocks.to_le_bytes());
        buf[32..36].copy_from_slice(&self.total_inodes.to_le_bytes());
        buf[36..40].copy_from_slice(&self.free_inodes.to_le_bytes());
        buf[40..48].copy_from_slice(&self.journal_start_block.to_le_bytes());
        buf[48..52].copy_from_slice(&self.journal_block_count.to_le_bytes());
        buf[52..60].copy_from_slice(&self.inode_table_block.to_le_bytes());
        buf[60..64].copy_from_slice(&self.root_inode.to_le_bytes());

        crc32c(&buf)
    }

    pub fn serialize(&self, buf: &mut [u8; BLOCK_SIZE]) {
        buf.fill(0);
        buf[0..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.version.to_le_bytes());
        buf[12..16].copy_from_slice(&self.block_size.to_le_bytes());
        buf[16..24].copy_from_slice(&self.total_blocks.to_le_bytes());
        buf[24..32].copy_from_slice(&self.free_blocks.to_le_bytes());
        buf[32..36].copy_from_slice(&self.total_inodes.to_le_bytes());
        buf[36..40].copy_from_slice(&self.free_inodes.to_le_bytes());
        buf[40..48].copy_from_slice(&self.journal_start_block.to_le_bytes());
        buf[48..52].copy_from_slice(&self.journal_block_count.to_le_bytes());
        buf[52..60].copy_from_slice(&self.inode_table_block.to_le_bytes());
        buf[60..64].copy_from_slice(&self.root_inode.to_le_bytes());
        buf[64] = self.clean_unmount;
        buf[68..72].copy_from_slice(&self.crc32c.to_le_bytes());
    }

    pub fn deserialize(buf: &[u8; BLOCK_SIZE]) -> Result<Self, &'static str> {
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&buf[0..8]);
        if magic != INDFS_MAGIC {
            return Err("EINVAL: Invalid IndFS superblock magic");
        }

        let version = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let block_size = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let total_blocks = u64::from_le_bytes([buf[16], buf[17], buf[18], buf[19], buf[20], buf[21], buf[22], buf[23]]);
        let free_blocks = u64::from_le_bytes([buf[24], buf[25], buf[26], buf[27], buf[28], buf[29], buf[30], buf[31]]);
        let total_inodes = u32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]);
        let free_inodes = u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]);
        let journal_start_block = u64::from_le_bytes([buf[40], buf[41], buf[42], buf[43], buf[44], buf[45], buf[46], buf[47]]);
        let journal_block_count = u32::from_le_bytes([buf[48], buf[49], buf[50], buf[51]]);
        let inode_table_block = u64::from_le_bytes([buf[52], buf[53], buf[54], buf[55], buf[56], buf[57], buf[58], buf[59]]);
        let root_inode = u32::from_le_bytes([buf[60], buf[61], buf[62], buf[63]]);
        let clean_unmount = buf[64];
        let stored_crc = u32::from_le_bytes([buf[68], buf[69], buf[70], buf[71]]);

        let sb = Self {
            magic,
            version,
            block_size,
            total_blocks,
            free_blocks,
            total_inodes,
            free_inodes,
            journal_start_block,
            journal_block_count,
            inode_table_block,
            root_inode,
            clean_unmount,
            crc32c: stored_crc,
        };

        if sb.compute_checksum() != stored_crc {
            return Err("EIO: Superblock CRC32C checksum mismatch (INV-FS-001)");
        }

        Ok(sb)
    }
}

/// On-disk Inode structure (64 bytes)
#[derive(Clone, Copy, Debug)]
pub struct Inode {
    pub inode_number: u32,
    pub mode: u16, // 1 = regular file, 2 = directory
    pub size_bytes: u64,
    pub block_num: u64, // direct block pointer
    pub cap_badge: u64,
    pub name: [u8; 24],
    pub name_len: u8,
    pub crc32c: u32,
}

impl Inode {
    pub const fn empty() -> Self {
        Self {
            inode_number: 0,
            mode: 0,
            size_bytes: 0,
            block_num: 0,
            cap_badge: 0,
            name: [0u8; 24],
            name_len: 0,
            crc32c: 0,
        }
    }

    pub fn compute_checksum(&self) -> u32 {
        let mut buf = [0u8; 60];
        buf[0..4].copy_from_slice(&self.inode_number.to_le_bytes());
        buf[4..6].copy_from_slice(&self.mode.to_le_bytes());
        buf[6..14].copy_from_slice(&self.size_bytes.to_le_bytes());
        buf[14..22].copy_from_slice(&self.block_num.to_le_bytes());
        buf[22..30].copy_from_slice(&self.cap_badge.to_le_bytes());
        buf[30..54].copy_from_slice(&self.name);
        buf[54] = self.name_len;
        crc32c(&buf)
    }

    pub fn serialize(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.inode_number.to_le_bytes());
        buf[4..6].copy_from_slice(&self.mode.to_le_bytes());
        buf[6..14].copy_from_slice(&self.size_bytes.to_le_bytes());
        buf[14..22].copy_from_slice(&self.block_num.to_le_bytes());
        buf[22..30].copy_from_slice(&self.cap_badge.to_le_bytes());
        buf[30..54].copy_from_slice(&self.name);
        buf[54] = self.name_len;
        buf[56..60].copy_from_slice(&self.crc32c.to_le_bytes());
    }

    pub fn deserialize(buf: &[u8]) -> Option<Self> {
        let inode_number = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if inode_number == 0 {
            return None;
        }
        let mode = u16::from_le_bytes([buf[4], buf[5]]);
        let size_bytes = u64::from_le_bytes([buf[6], buf[7], buf[8], buf[9], buf[10], buf[11], buf[12], buf[13]]);
        let block_num = u64::from_le_bytes([buf[14], buf[15], buf[16], buf[17], buf[18], buf[19], buf[20], buf[21]]);
        let cap_badge = u64::from_le_bytes([buf[22], buf[23], buf[24], buf[25], buf[26], buf[27], buf[28], buf[29]]);
        let mut name = [0u8; 24];
        name.copy_from_slice(&buf[30..54]);
        let name_len = buf[54];
        let stored_crc = u32::from_le_bytes([buf[56], buf[57], buf[58], buf[59]]);

        let inode = Self {
            inode_number,
            mode,
            size_bytes,
            block_num,
            cap_badge,
            name,
            name_len,
            crc32c: stored_crc,
        };

        if inode.compute_checksum() == stored_crc {
            Some(inode)
        } else {
            None
        }
    }
}

/// IndFS Persistent Filesystem Manager (lightweight struct, device passed by reference)
pub struct IndFs {
    pub sb: Superblock,
    pub journal: Journal,
    pub inodes: [Inode; MAX_INODES],
    pub block_checksums: [u32; TOTAL_DEVICE_BLOCKS],
    pub is_mounted: bool,
}

impl IndFs {
    /// Formats a block device with initial IndFS filesystem metadata
    pub fn format(dev: &mut MemBlockDevice) -> Result<Self, &'static str> {
        let sb = Superblock::new(dev.block_count());
        let mut block_buf = [0u8; BLOCK_SIZE];

        // 1. Write Superblock
        sb.serialize(&mut block_buf);
        dev.write_block(SUPERBLOCK_BLOCK_NUM, &block_buf)?;

        // 2. Initialize Inode table with root directory (inode 1)
        block_buf.fill(0);
        let mut root_inode = Inode::empty();
        root_inode.inode_number = 1;
        root_inode.mode = 2; // Directory
        root_inode.name[0] = b'/';
        root_inode.name_len = 1;
        root_inode.cap_badge = 0x1007;
        root_inode.crc32c = root_inode.compute_checksum();
        root_inode.serialize(&mut block_buf[0..64]);

        dev.write_block(INODE_TABLE_BLOCK, &block_buf)?;

        let mut inodes = [Inode::empty(); MAX_INODES];
        inodes[0] = root_inode;

        let journal = Journal::new(JOURNAL_START_BLOCK, JOURNAL_BLOCK_COUNT);

        Ok(Self {
            sb,
            journal,
            inodes,
            block_checksums: [0u32; TOTAL_DEVICE_BLOCKS],
            is_mounted: true,
        })
    }

    /// Mounts an existing IndFS filesystem from block device
    pub fn mount(dev: &mut MemBlockDevice) -> Result<Self, &'static str> {
        let mut block_buf = [0u8; BLOCK_SIZE];
        dev.read_block(SUPERBLOCK_BLOCK_NUM, &mut block_buf)?;

        let mut sb = Superblock::deserialize(&block_buf)?;

        // Mark dirty upon mount (set clean_unmount = 0)
        sb.clean_unmount = 0;
        sb.crc32c = sb.compute_checksum();
        sb.serialize(&mut block_buf);
        dev.write_block(SUPERBLOCK_BLOCK_NUM, &block_buf)?;

        // Load Inode table
        dev.read_block(INODE_TABLE_BLOCK, &mut block_buf)?;
        let mut inodes = [Inode::empty(); MAX_INODES];
        for i in 0..MAX_INODES {
            let offset = i * 64;
            if let Some(inode) = Inode::deserialize(&block_buf[offset..offset + 64]) {
                inodes[i] = inode;
            }
        }

        // Load Checksum table
        dev.read_block(CHECKSUM_TABLE_BLOCK, &mut block_buf)?;
        let mut block_checksums = [0u32; TOTAL_DEVICE_BLOCKS];
        for i in 0..TOTAL_DEVICE_BLOCKS {
            let offset = i * 4;
            block_checksums[i] = u32::from_le_bytes([
                block_buf[offset],
                block_buf[offset + 1],
                block_buf[offset + 2],
                block_buf[offset + 3],
            ]);
        }

        let journal = Journal::new(sb.journal_start_block, sb.journal_block_count);

        Ok(Self {
            sb,
            journal,
            inodes,
            block_checksums,
            is_mounted: true,
        })
    }

    /// Creates a regular file with journal logging (INV-FS-002)
    pub fn create_file(&mut self, dev: &mut MemBlockDevice, name: &str) -> Result<u32, &'static str> {
        let mut free_slot = None;
        for i in 1..MAX_INODES {
            if self.inodes[i].inode_number == 0 {
                free_slot = Some(i);
                break;
            }
        }
        let slot = free_slot.ok_or("ENOSPC: Inode table full")?;
        let inode_num = (slot + 1) as u32;

        let data_block = DATA_START_BLOCK + (slot as u64);

        let mut inode = Inode::empty();
        inode.inode_number = inode_num;
        inode.mode = 1; // Regular file
        inode.size_bytes = 0;
        inode.block_num = data_block;
        inode.cap_badge = 0xF11E;

        let bytes = name.as_bytes();
        let len = if bytes.len() > 24 { 24 } else { bytes.len() };
        inode.name[..len].copy_from_slice(&bytes[..len]);
        inode.name_len = len as u8;
        inode.crc32c = inode.compute_checksum();

        // 1. Log inode creation to journal
        let mut inode_bytes = [0u8; 64];
        inode.serialize(&mut inode_bytes);
        self.journal.log_op(dev, OP_WRITE_INODE, inode_num as u64, &inode_bytes)?;

        // 2. Commit transaction
        self.journal.commit(dev)?;

        // 3. Write inode to in-memory cache and flush to inode table block
        self.inodes[slot] = inode;
        self.flush_inode_table(dev)?;

        Ok(inode_num)
    }

    /// Writes data to a file with WAL journaling and per-block CRC32C checksum (INV-FS-001, INV-FS-002)
    pub fn write_file(&mut self, dev: &mut MemBlockDevice, inode_num: u32, offset: usize, data: &[u8]) -> Result<usize, &'static str> {
        let slot = (inode_num as usize).checked_sub(1).ok_or("EINVAL: Invalid inode number")?;
        if slot >= MAX_INODES || self.inodes[slot].inode_number != inode_num {
            return Err("ENOENT: Inode not found");
        }

        let target_block = self.inodes[slot].block_num;
        let mut block_buf = [0u8; BLOCK_SIZE];

        if self.inodes[slot].size_bytes > 0 {
            dev.read_block(target_block, &mut block_buf)?;
        }

        let avail = BLOCK_SIZE - offset;
        let to_write = if data.len() < avail { data.len() } else { avail };
        block_buf[offset..offset + to_write].copy_from_slice(&data[..to_write]);

        // Compute block CRC32C
        let new_crc = crc32c(&block_buf);

        // 1. Write-Ahead Log: Log block payload and inode update
        self.journal.log_op(dev, OP_WRITE_BLOCK, target_block, &block_buf[0..128])?;

        if offset + to_write > (self.inodes[slot].size_bytes as usize) {
            self.inodes[slot].size_bytes = (offset + to_write) as u64;
            self.inodes[slot].crc32c = self.inodes[slot].compute_checksum();

            let mut inode_bytes = [0u8; 64];
            self.inodes[slot].serialize(&mut inode_bytes);
            self.journal.log_op(dev, OP_WRITE_INODE, inode_num as u64, &inode_bytes)?;
        }

        // 2. Commit transaction
        self.journal.commit(dev)?;

        // 3. Checkpoint to storage block
        dev.write_block(target_block, &block_buf)?;

        // 4. Update checksum table (INV-FS-001)
        self.block_checksums[target_block as usize] = new_crc;
        self.flush_checksum_table(dev)?;
        self.flush_inode_table(dev)?;

        Ok(to_write)
    }

    /// Reads data from a file with strict CRC32C anti-bitrot checksum verification (INV-FS-001)
    pub fn read_file(&self, dev: &MemBlockDevice, inode_num: u32, offset: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
        let slot = (inode_num as usize).checked_sub(1).ok_or("EINVAL: Invalid inode number")?;
        if slot >= MAX_INODES || self.inodes[slot].inode_number != inode_num {
            return Err("ENOENT: Inode not found");
        }

        let target_block = self.inodes[slot].block_num;
        let mut block_buf = [0u8; BLOCK_SIZE];

        dev.read_block(target_block, &mut block_buf)?;

        // INV-FS-001: Validate block CRC32C checksum against recorded table
        let expected_crc = self.block_checksums[target_block as usize];
        if expected_crc != 0 {
            let actual_crc = crc32c(&block_buf);
            if actual_crc != expected_crc {
                return Err("EIO: Block CRC32C checksum mismatch: silent data corruption detected (INV-FS-001)");
            }
        }

        let size = self.inodes[slot].size_bytes as usize;
        if offset >= size {
            return Ok(0);
        }

        let avail = size - offset;
        let to_read = if buf.len() < avail { buf.len() } else { avail };
        buf[..to_read].copy_from_slice(&block_buf[offset..offset + to_read]);

        Ok(to_read)
    }

    /// Flushes inode table block to device
    fn flush_inode_table(&mut self, dev: &mut MemBlockDevice) -> Result<(), &'static str> {
        let mut block_buf = [0u8; BLOCK_SIZE];
        for i in 0..MAX_INODES {
            let offset = i * 64;
            if self.inodes[i].inode_number != 0 {
                self.inodes[i].serialize(&mut block_buf[offset..offset + 64]);
            }
        }
        dev.write_block(INODE_TABLE_BLOCK, &block_buf)
    }

    /// Flushes checksum table to device
    fn flush_checksum_table(&mut self, dev: &mut MemBlockDevice) -> Result<(), &'static str> {
        let mut block_buf = [0u8; BLOCK_SIZE];
        for i in 0..TOTAL_DEVICE_BLOCKS {
            let offset = i * 4;
            block_buf[offset..offset + 4].copy_from_slice(&self.block_checksums[i].to_le_bytes());
        }
        dev.write_block(CHECKSUM_TABLE_BLOCK, &block_buf)
    }

    /// Cleanly unmounts filesystem, setting clean_unmount = 1
    pub fn unmount(&mut self, dev: &mut MemBlockDevice) -> Result<(), &'static str> {
        self.sb.clean_unmount = 1;
        self.sb.crc32c = self.sb.compute_checksum();

        let mut block_buf = [0u8; BLOCK_SIZE];
        self.sb.serialize(&mut block_buf);
        dev.write_block(SUPERBLOCK_BLOCK_NUM, &block_buf)?;

        self.is_mounted = false;
        Ok(())
    }

    pub fn is_clean(&self) -> bool {
        self.sb.clean_unmount == 1
    }
}
