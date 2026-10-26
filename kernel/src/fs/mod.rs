//! File Systems & Storage Subsystem for IndK
//! Compliant with Bible §8.10 (SP-25), Rules N-01, N-04, N-10, INV-FS-001, and INV-FS-002.

pub mod block;
pub mod crc32;
pub mod fsck;
pub mod indfs;
pub mod journal;
pub mod tmpfs;

pub use block::{BlockDevice, MemBlockDevice, BLOCK_SIZE};
pub use crc32::crc32c;
pub use fsck::{Fsck, FsckReport};
pub use indfs::{IndFs, Inode, Superblock};
pub use journal::{Journal, JournalRecord};
pub use tmpfs::{TmpFs, TmpFsNode};
