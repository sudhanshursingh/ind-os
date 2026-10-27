//! Userspace Store IndFS on UFS Mount Manager (SP-99, REQ-SYS-101, REQ-STR-103)
//!
//! Root filesystem mount of IndFS onto Qualcomm UFS 2.2 hardware partition.

pub const CAP_STORAGE_MOUNT: u32 = 0x0001_0000;
pub const INDFS_SUPERBLOCK_MAGIC: u32 = 0x494E_4446; // "INDF"
pub const INDFS_BLOCK_SIZE_BYTES: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UfsMountError {
    PermissionDenied,
    SuperblockMagicMismatch,
    BlockSizeUnsupported,
    JournalReplayFailed,
    DeviceNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UfsMountState {
    Unmounted,
    SuperblockVerified,
    JournalReplayed,
    Mounted,
}

pub struct UfsMountHarness {
    state: UfsMountState,
    magic: u32,
    block_size: u32,
    replayed_txs: usize,
}

impl UfsMountHarness {
    pub const fn new() -> Self {
        Self {
            state: UfsMountState::Unmounted,
            magic: 0,
            block_size: 0,
            replayed_txs: 0,
        }
    }

    pub fn mount_root_ufs(&mut self, cap_token: u32, magic: u32, block_size: u32) -> Result<(), UfsMountError> {
        if (cap_token & CAP_STORAGE_MOUNT) == 0 {
            return Err(UfsMountError::PermissionDenied);
        }
        if magic != INDFS_SUPERBLOCK_MAGIC {
            return Err(UfsMountError::SuperblockMagicMismatch);
        }
        if block_size != INDFS_BLOCK_SIZE_BYTES {
            return Err(UfsMountError::BlockSizeUnsupported);
        }
        self.magic = magic;
        self.block_size = block_size;
        self.state = UfsMountState::SuperblockVerified;
        Ok(())
    }

    pub fn replay_journal(&mut self) -> Result<usize, UfsMountError> {
        if self.state != UfsMountState::SuperblockVerified {
            return Err(UfsMountError::SuperblockMagicMismatch);
        }
        self.replayed_txs = 12;
        self.state = UfsMountState::Mounted;
        Ok(self.replayed_txs)
    }

    pub fn is_mounted(&self) -> bool {
        self.state == UfsMountState::Mounted
    }

    pub fn magic(&self) -> u32 {
        self.magic
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn replayed_transactions_count(&self) -> usize {
        self.replayed_txs
    }

    pub fn state(&self) -> UfsMountState {
        self.state
    }
}
