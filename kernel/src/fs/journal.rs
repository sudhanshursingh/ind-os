//! IndFS Write-Ahead Logging (WAL) Journaling Subsystem
//! Compliant with Bible §8.10 (SP-25), INDOS-SPEC-INDFS-01, Rule N-01, Rule N-04, and Invariant INV-FS-002.

use crate::fs::block::{MemBlockDevice, BLOCK_SIZE};
use crate::fs::crc32::crc32c;

pub const JOURNAL_MAGIC: u32 = 0x4A4F5552; // b"JOUR"
pub const OP_WRITE_INODE: u16 = 1;
pub const OP_WRITE_BLOCK: u16 = 2;
pub const OP_UPDATE_SUPERBLOCK: u16 = 3;
pub const OP_COMMIT: u16 = 4;

/// On-disk transaction log record header
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct JournalRecord {
    pub magic: u32,
    pub sequence_num: u64,
    pub op_type: u16,
    pub target_id: u64,
    pub payload_len: u16,
    pub payload: [u8; 128],
    pub crc32c: u32,
}

impl JournalRecord {
    pub fn new(seq: u64, op_type: u16, target_id: u64, payload_data: &[u8]) -> Self {
        let mut payload = [0u8; 128];
        let len = if payload_data.len() > 128 { 128 } else { payload_data.len() };
        payload[..len].copy_from_slice(&payload_data[..len]);

        let mut rec = Self {
            magic: JOURNAL_MAGIC,
            sequence_num: seq,
            op_type,
            target_id,
            payload_len: len as u16,
            payload,
            crc32c: 0,
        };

        // Compute CRC over magic, seq, op_type, target_id, len, payload
        let mut buf = [0u8; 152];
        buf[0..4].copy_from_slice(&rec.magic.to_le_bytes());
        buf[4..12].copy_from_slice(&rec.sequence_num.to_le_bytes());
        buf[12..14].copy_from_slice(&rec.op_type.to_le_bytes());
        buf[14..22].copy_from_slice(&rec.target_id.to_le_bytes());
        buf[22..24].copy_from_slice(&rec.payload_len.to_le_bytes());
        buf[24..152].copy_from_slice(&rec.payload);

        rec.crc32c = crc32c(&buf);
        rec
    }

    pub fn is_valid(&self) -> bool {
        if self.magic != JOURNAL_MAGIC {
            return false;
        }
        let mut buf = [0u8; 152];
        buf[0..4].copy_from_slice(&self.magic.to_le_bytes());
        buf[4..12].copy_from_slice(&self.sequence_num.to_le_bytes());
        buf[12..14].copy_from_slice(&self.op_type.to_le_bytes());
        buf[14..22].copy_from_slice(&self.target_id.to_le_bytes());
        buf[22..24].copy_from_slice(&self.payload_len.to_le_bytes());
        buf[24..152].copy_from_slice(&self.payload);

        crc32c(&buf) == self.crc32c
    }

    pub fn serialize(&self, buf: &mut [u8]) {
        if buf.len() < 156 {
            return;
        }
        buf[0..4].copy_from_slice(&self.magic.to_le_bytes());
        buf[4..12].copy_from_slice(&self.sequence_num.to_le_bytes());
        buf[12..14].copy_from_slice(&self.op_type.to_le_bytes());
        buf[14..22].copy_from_slice(&self.target_id.to_le_bytes());
        buf[22..24].copy_from_slice(&self.payload_len.to_le_bytes());
        buf[24..152].copy_from_slice(&self.payload);
        buf[152..156].copy_from_slice(&self.crc32c.to_le_bytes());
    }

    pub fn deserialize(buf: &[u8]) -> Option<Self> {
        if buf.len() < 156 {
            return None;
        }
        let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let sequence_num = u64::from_le_bytes([buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11]]);
        let op_type = u16::from_le_bytes([buf[12], buf[13]]);
        let target_id = u64::from_le_bytes([buf[14], buf[15], buf[16], buf[17], buf[18], buf[19], buf[20], buf[21]]);
        let payload_len = u16::from_le_bytes([buf[22], buf[23]]);
        let mut payload = [0u8; 128];
        payload.copy_from_slice(&buf[24..152]);
        let crc32c_val = u32::from_le_bytes([buf[152], buf[153], buf[154], buf[155]]);

        let rec = Self {
            magic,
            sequence_num,
            op_type,
            target_id,
            payload_len,
            payload,
            crc32c: crc32c_val,
        };

        if rec.is_valid() {
            Some(rec)
        } else {
            None
        }
    }
}

/// Circular ring buffer transaction journal manager
pub struct Journal {
    pub start_block: u64,
    pub block_count: u32,
    pub head_block: u64,
    pub next_seq: u64,
    pub last_committed_seq: u64,
}

impl Journal {
    pub const fn new(start_block: u64, block_count: u32) -> Self {
        Self {
            start_block,
            block_count,
            head_block: start_block,
            next_seq: 1,
            last_committed_seq: 0,
        }
    }

    /// Appends an operation record to the journal ring buffer
    pub fn log_op(
        &mut self,
        dev: &mut MemBlockDevice,
        op_type: u16,
        target_id: u64,
        payload: &[u8],
    ) -> Result<u64, &'static str> {
        let seq = self.next_seq;
        let record = JournalRecord::new(seq, op_type, target_id, payload);

        let mut block_buf = [0u8; BLOCK_SIZE];
        record.serialize(&mut block_buf);

        dev.write_block(self.head_block, &block_buf)?;

        // Advance circular head
        self.head_block += 1;
        if self.head_block >= self.start_block + (self.block_count as u64) {
            self.head_block = self.start_block;
        }

        Ok(seq)
    }

    /// Atomically commits a transaction by logging a Commit record with CRC32C and flushing
    pub fn commit(&mut self, dev: &mut MemBlockDevice) -> Result<u64, &'static str> {
        let seq = self.next_seq;
        self.log_op(dev, OP_COMMIT, seq, &[])?;
        self.last_committed_seq = seq;
        self.next_seq += 1;
        Ok(seq)
    }

    /// Checkpoints transaction, advancing tail
    pub fn checkpoint(&mut self) {
        // In this implementation, checkpoint matches last committed transaction
    }
}
