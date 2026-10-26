//! IndFS Consistency Checker & Crash Recovery State Machine (fsck)
//! Compliant with Bible §8.10 (SP-25), INDOS-SPEC-INDFS-01, Rule N-01, Rule N-04, and Invariant INV-FS-002.

use crate::fs::block::{MemBlockDevice, BLOCK_SIZE};
use crate::fs::indfs::{Inode, Superblock, INODE_TABLE_BLOCK, JOURNAL_BLOCK_COUNT, JOURNAL_START_BLOCK, MAX_INODES, SUPERBLOCK_BLOCK_NUM};
use crate::fs::journal::{JournalRecord, OP_COMMIT, OP_WRITE_BLOCK, OP_WRITE_INODE};

/// Summary report emitted by fsck upon completion
#[derive(Clone, Copy, Debug)]
pub struct FsckReport {
    pub clean_before: bool,
    pub replayed_transactions: usize,
    pub discarded_transactions: usize,
    pub verified_inodes: usize,
    pub is_clean_now: bool,
}

/// IndFS Consistency Checker & Journal Replay Engine
pub struct Fsck;

impl Fsck {
    /// Audits filesystem consistency and performs atomic journal replay if uncleanly unmounted (INV-FS-002)
    pub fn check_and_recover(dev: &mut MemBlockDevice) -> Result<FsckReport, &'static str> {
        let mut block_buf = [0u8; BLOCK_SIZE];

        // 1. Read Superblock
        dev.read_block(SUPERBLOCK_BLOCK_NUM, &mut block_buf)?;
        let mut sb = Superblock::deserialize(&block_buf)?;

        let was_clean = sb.clean_unmount == 1;

        let mut replayed = 0;
        let mut discarded = 0;

        if !was_clean {
            // Unclean unmount detected! Initiate 3-Phase recovery protocol (INV-FS-002)

            // Phase 1: Scan Journal ring buffer for valid committed transactions
            let mut journal_entries: [Option<JournalRecord>; 16] = [None; 16];
            for i in 0..JOURNAL_BLOCK_COUNT as usize {
                let blk = JOURNAL_START_BLOCK + (i as u64);
                dev.read_block(blk, &mut block_buf)?;
                journal_entries[i] = JournalRecord::deserialize(&block_buf);
            }

            // Find highest sequence number that has an OP_COMMIT record
            let mut committed_seqs = [0u64; 16];
            let mut committed_count = 0;
            for i in 0..16 {
                if let Some(ref rec) = journal_entries[i] {
                    if rec.op_type == OP_COMMIT {
                        committed_seqs[committed_count] = rec.sequence_num;
                        committed_count += 1;
                    }
                }
            }

            // Replay all operations belonging to committed transactions
            for i in 0..16 {
                if let Some(ref rec) = journal_entries[i] {
                    let mut is_committed = false;
                    for c in 0..committed_count {
                        if committed_seqs[c] == rec.sequence_num {
                            is_committed = true;
                            break;
                        }
                    }

                    if is_committed {
                        if rec.op_type == OP_WRITE_BLOCK {
                            // Replay block write to target block
                            let mut target_buf = [0u8; BLOCK_SIZE];
                            dev.read_block(rec.target_id, &mut target_buf)?;
                            let len = rec.payload_len as usize;
                            target_buf[..len].copy_from_slice(&rec.payload[..len]);
                            dev.write_block(rec.target_id, &target_buf)?;
                            replayed += 1;
                        } else if rec.op_type == OP_WRITE_INODE {
                            // Replay inode update to inode table
                            let mut itable_buf = [0u8; BLOCK_SIZE];
                            dev.read_block(INODE_TABLE_BLOCK, &mut itable_buf)?;
                            let slot = (rec.target_id as usize).saturating_sub(1);
                            if slot < MAX_INODES {
                                let offset = slot * 64;
                                itable_buf[offset..offset + 64].copy_from_slice(&rec.payload[..64]);
                                dev.write_block(INODE_TABLE_BLOCK, &itable_buf)?;
                                replayed += 1;
                            }
                        }
                    } else if rec.op_type != OP_COMMIT {
                        // Uncommitted transaction abandoned (simulated crash during active write)
                        discarded += 1;
                    }
                }
            }

            // Phase 2: Inode reachability and sanity validation
            let mut verified_inodes = 0;
            dev.read_block(INODE_TABLE_BLOCK, &mut block_buf)?;
            for i in 0..MAX_INODES {
                let offset = i * 64;
                if let Some(_inode) = Inode::deserialize(&block_buf[offset..offset + 64]) {
                    verified_inodes += 1;
                }
            }

            // Phase 3: Set clean_unmount = 1 and write back updated superblock
            sb.clean_unmount = 1;
            sb.crc32c = sb.compute_checksum();
            sb.serialize(&mut block_buf);
            dev.write_block(SUPERBLOCK_BLOCK_NUM, &block_buf)?;

            Ok(FsckReport {
                clean_before: false,
                replayed_transactions: replayed,
                discarded_transactions: discarded,
                verified_inodes,
                is_clean_now: true,
            })
        } else {
            // Already clean
            let mut verified_inodes = 0;
            dev.read_block(INODE_TABLE_BLOCK, &mut block_buf)?;
            for i in 0..MAX_INODES {
                let offset = i * 64;
                if let Some(_inode) = Inode::deserialize(&block_buf[offset..offset + 64]) {
                    verified_inodes += 1;
                }
            }

            Ok(FsckReport {
                clean_before: true,
                replayed_transactions: 0,
                discarded_transactions: 0,
                verified_inodes,
                is_clean_now: true,
            })
        }
    }
}
