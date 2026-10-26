# Ind OS Architecture Specification: IndFS (Journaling, Checksummed Persistent File System)

**Document Identifier:** `INDOS-SPEC-INDFS-01`  
**Classification:** Core Kernel Architecture / Phase 4 Persistent Storage  
**Status:** Approved Design Specification (Delivered in Sprint SP-24 for Implementation in SP-25)  
**Governing Standard:** Ind OS Development Bible v1.0.0.0 (§8.10, Phase 4)  
**Target Sprint for Implementation:** Sprint SP-25 (`tmpfs` + `IndFS`)  

---

## 1. Executive Summary & Design Principles

IndFS is the native, crash-resilient, journaling, and checksummed persistent file system designed for Ind OS. Built from first principles to operate atop NVMe, eMMC, and flash block storage, IndFS provides zero-ambient-authority capability integration, deterministic crash consistency, and mathematical verification against bitrot and silent corruption.

### Core Design Tenets:
1. **Crash Consistency via Write-Ahead Logging (WAL):** Metadata modifications are logged to a dedicated transactional journal before committing to storage structures, guaranteeing atomic updates across unexpected power cuts.
2. **End-to-End Block Checksumming:** Every 4 KiB metadata block and data block is accompanied by a 32-bit CRC32C or cryptographic hash to detect silent hardware corruption and bitrot.
3. **Capability-Mediated Inodes:** Every inode encodes capability binding parameters, enabling direct synthesis of unforgeable `CapType::File(vnode_id)` access control tokens per `REQ-KRN-106`.
4. **Deterministic `fsck` Recovery:** A bounded recovery state machine replays committed transactions in seconds and detects orphaned or corrupted blocks deterministically.

---

## 2. On-Disk Layout & Geometries

IndFS organizes storage into uniform 4096-byte (4 KiB) blocks, structured as follows:

```
+-----------------------------------------------------------------------------+
|                               IndFS Disk Layout                             |
|                                                                             |
| +-------------------+-----------------------------------------------------+ |
| | Block 0           | Bootloader Reserved / MBR / Partition Header        | |
| +-------------------+-----------------------------------------------------+ |
| | Block 1           | Primary Superblock (4 KiB)                          | |
| +-------------------+-----------------------------------------------------+ |
| | Block 2           | Backup Superblock (Mirror)                          | |
| +-------------------+-----------------------------------------------------+ |
| | Blocks 3 .. 66    | Transactional Journal (WAL Ring Buffer, 256 KiB)    | |
| +-------------------+-----------------------------------------------------+ |
| | Blocks 67 .. 74   | Block Allocation Bitmap                             | |
| +-------------------+-----------------------------------------------------+ |
| | Blocks 75 .. 82   | Inode Allocation Bitmap                             | |
| +-------------------+-----------------------------------------------------+ |
| | Blocks 83 .. 594  | Inode Table (512 blocks = 16,384 Inodes @ 128B each)| |
| +-------------------+-----------------------------------------------------+ |
| | Blocks 595 .. End | Data Extents & Block Checksum Tables                | |
| +-------------------+-----------------------------------------------------+ |
+-----------------------------------------------------------------------------+
```

---

## 3. Core On-Disk Structures

### 3.1 Superblock (`Superblock`)
The primary superblock is located at Block 1 (offset `0x1000`):

```rust
#[repr(C, packed)]
pub struct Superblock {
    pub magic: [u8; 8],            // b"INDFS01\0" (0x494e444653303100)
    pub version: u32,              // File system version (1)
    pub block_size: u32,           // 4096 bytes
    pub total_blocks: u64,         // Total storage capacity in blocks
    pub free_blocks: u64,          // Number of unallocated data blocks
    pub total_inodes: u32,         // Total inode capacity (e.g. 16,384)
    pub free_inodes: u32,          // Number of unallocated inodes
    pub journal_start_block: u64,  // Block 3
    pub journal_block_count: u32,  // 64 blocks (256 KiB)
    pub inode_table_block: u64,    // Block 83
    pub root_inode: u32,           // Root directory inode (default 1)
    pub mount_count: u32,          // Cumulative mount count
    pub clean_unmount: u8,         // 1 if unmounted cleanly, 0 if dirty
    pub reserved: [u8; 431],       // Padding to 508 bytes
    pub crc32c: u32,               // Checksum over superblock contents
}
```

### 3.2 Inode Structure (`Inode`)
Each inode is 128 bytes, allowing 32 inodes per 4096-byte block:

```rust
#[repr(C, packed)]
pub struct Inode {
    pub inode_number: u32,         // Unique identifier
    pub mode: u16,                 // File type (Directory, File, Symlink) & permissions
    pub nlink: u16,                // Hard link count
    pub uid: u32,                  // Owner ID
    pub gid: u32,                  // Group ID
    pub size_bytes: u64,           // File size in bytes
    pub atime: u64,                // Access timestamp (POSIX epoch ns)
    pub mtime: u64,                // Modification timestamp
    pub ctime: u64,                // Metadata change timestamp
    pub flags: u32,                // Immutable, encrypted, or compressed flags
    pub direct_extents: [Extent; 4], // 4 direct extents (block_num, count)
    pub indirect_block: u64,       // Block pointer for extended extent tree
    pub cap_badge: u64,            // Bound capability badge for access control
    pub crc32c: u32,               // Inode integrity checksum
}

#[repr(C, packed)]
pub struct Extent {
    pub logical_block: u32,        // Starting offset within file
    pub physical_block: u64,       // Starting block on disk
    pub count: u32,                // Number of contiguous blocks
}
```

---

## 4. Journaling Subsystem (Write-Ahead Logging)

To achieve atomicity and high performance, IndFS implements **Metadata Journaling** with optional data journaling for high-integrity assets.

```
       [Userspace / VFS Mutation]
                   |
                   v
       +-----------------------+
       | Stage Inode / Extent  |
       +-----------+-----------+
                   |
                   v
       +-----------------------+
       | Write Journal Entry   |  <--- Circular Ring Buffer (Blocks 3..66)
       +-----------+-----------+
                   |
                   v
       +-----------------------+
       | Issue Storage Barrier |  <--- Force NVMe/Flash flush
       +-----------+-----------+
                   |
                   v
       +-----------------------+
       | Write Commit Record   |  <--- Atomic transaction commit
       +-----------+-----------+
                   |
                   v
       +-----------------------+
       | Checkpoint to Disk    |  <--- Inode table & metadata updated in place
       +-----------------------+
```

### 4.1 Journal Transaction Record
```rust
#[repr(C, packed)]
pub struct JournalRecordHeader {
    pub magic: u32,                // 0x4A4F5552 (b"JOUR")
    pub sequence_num: u64,         // Monotonically increasing transaction ID
    pub op_type: u16,              // AllocBlock, FreeBlock, WriteInode, UpdateSuperblock
    pub payload_len: u16,          // Length of following payload
    pub crc32c: u32,               // Checksum over header + payload
}
```

---

## 5. End-to-End Integrity & Checksumming

To eliminate silent bitrot, IndFS maintains checksum descriptors for all blocks:
1. **Metadata Blocks:** Every inode and directory block includes its own embedded CRC32C.
2. **Data Blocks:** A global Checksum Table stores a 32-bit CRC32C for every active block index:
   $$\text{Checksum}[B] = \text{CRC32C}(\text{Block}[B])$$
3. **Verification on Read:** The block driver validates `CRC32C` upon retrieval. If a mismatch occurs, `EIO` is returned and an audit event (`EventType::IntegrityViolation`) is appended to the SHA-256 hash-chained audit log (`INV-AUD-001`).

---

## 6. Crash Recovery Protocol (`fsck`)

If `Superblock::clean_unmount == 0` upon mount:
1. **Phase 1: Journal Replay:** Scan the journal ring buffer for records where `sequence_num > last_checkpoint`.
   - If a valid `CommitRecord` with matching checksum is found, replay operations in order.
   - If an incomplete transaction is detected, discard it immediately.
2. **Phase 2: Inode Reachability Validation:**
   - Traverse directory hierarchy from root inode (1).
   - Verify link counts and reconcile allocated inodes against the Inode Allocation Bitmap.
3. **Phase 3: Extent Reconciliation:**
   - Scan all extents; verify no double-allocations exist.
   - Recompute free block count and write back updated superblock with `clean_unmount = 1`.

---

## 7. Integration with Phase 4 VFS

IndFS registers as a concrete filesystem driver with the VFS Core:
- Mount point registration in [`MountTable`](file:///Users/dhan/IndOS/kernel/src/vfs/mount.rs) (e.g. `/data` or `/persist`).
- Inode-to-vnode translation: Each active IndFS inode maps 1:1 to a VFS [`Vnode`](file:///Users/dhan/IndOS/kernel/src/vfs/vnode.rs).
- Capability binding: File creation and open operations mint [`Capability`](file:///Users/dhan/IndOS/kernel/src/capability/table.rs) tokens bound to `CapType::File(vnode_id)`, ensuring seamless zero-ambient-authority enforcement (`REQ-KRN-106`, `INV-VFS-001`).
