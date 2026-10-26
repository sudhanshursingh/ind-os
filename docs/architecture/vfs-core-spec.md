# Ind OS Architecture Specification: Virtual File System Core Subsystem (VFS Core)

**Document Identifier:** `INDOS-SPEC-VFS-01`  
**Classification:** Core Kernel Architecture / Phase 4 Storage & IO  
**Status:** Approved & Implemented (Sprint SP-24)  
**Governing Standard:** Ind OS Development Bible v1.0.0.0 (§8.10, Phase 4)  
**Requirements Satisfied:** `REQ-KRN-106` (Capability-Mediated File Handles)  
**Invariants Enforced:** `INV-VFS-001`, `INV-VFS-002`  

---

## 1. Executive Summary

Phase 4 of Ind OS initiates the storage, user-mode driver framework, and I/O infrastructure. The Virtual File System (VFS) Core establishes the fundamental abstraction layer through which all storage objects, pseudofilesystems, device nodes, and persistent volumes are accessed.

Crucially, in adherence to the Phase 3 Security Spine and `REQ-KRN-106`, file access in Ind OS is **capability-mediated from day one**. There is zero ambient authority: traditional POSIX integer file descriptors decoupled from capabilities do not exist. Every file open operation returns both an in-kernel descriptor and an unforgeable `Capability` token of type `CapType::File(vnode_id)` encoding exact access rights (`CapRights::READ`, `CapRights::WRITE`). Every subsequent read, write, or mutation must present this capability token, which is verified dynamically against revocation, expiration, and right attenuation (`INV-VFS-001`).

Furthermore, path resolution enforces strict canonicalization and root isolation (`INV-VFS-002`), ensuring that relative path traversing (`..`) cannot escape the mounted root hierarchy.

---

## 2. Normative Invariants

### `INV-VFS-001`: Capability-Mediated File Operations
> **Invariant Statement:** Every file operation (`read`, `write`, `truncate`, `ioctl`) on an open file descriptor must present an active, valid `Capability` token where `cap_type == CapType::File(vnode_id)` and `rights` contains the required permission (`READ` for reading, `WRITE` for writing). Access with an invalid or revoked capability, an mismatched vnode token, or insufficient rights must be deterministically rejected with `EPERM`.

### `INV-VFS-002`: Hierarchical Path Resolution & Jail Isolation
> **Invariant Statement:** The path resolver must canonicalize all paths into an absolute normalized form (resolving redundant slashes, `.`, and `..` components). Traversal of parent directory components (`..`) must never ascend above the root directory (`/`). Any attempt to traverse above root (`/../../`) must resolve safely to `/`, guaranteeing that process namespaces cannot escape their mount jail.

---

## 3. Subsystem Architecture

The VFS Core subsystem is structured into five cohesive modules within [`kernel/src/vfs/`](file:///Users/dhan/IndOS/kernel/src/vfs/):

```
+-----------------------------------------------------------------------------+
|                            IndK Microkernel                                 |
|                                                                             |
|   +---------------------------------------------------------------------+   |
|   |                      Global VFS Singleton (VFS)                     |   |
|   |                    (kernel/src/vfs/mod.rs)                          |   |
|   +----------------------------------+----------------------------------+   |
|                                      |                                      |
|          +---------------------------+---------------------------+          |
|          v                                                       v          |
|   +-----------------------------+               +-----------------------+   |
|   |      Mount Table            |               |  Vnode Pool           |   |
|   |  - MountEntry[MAX_MOUNTS]   |               |  - Vnode[MAX_VNODES]  |   |
|   |  - Prefix matching (/, dev) |               |  - RegularFile, Dir,  |   |
|   |  (kernel/src/vfs/mount.rs)  |               |    Device, Symlink    |   |
|   +--------------+--------------+               |  (vfs/vnode.rs)       |   |
|                  |                              +-----------+-----------+   |
|                  v                                          ^               |
|   +-----------------------------+                           |               |
|   |     Path Resolver           |                           |               |
|   |  - Path normalization       |                           |               |
|   |  - Root isolation           +---------------------------+               |
|   |    (INV-VFS-002)            |        (Lookup)                           |
|   |  (kernel/src/vfs/path.rs)   |                                           |
|   +-----------------------------+                                           |
|                                                                             |
|   +---------------------------------------------------------------------+   |
|   |             Capability-Guarded File Descriptors                     |   |
|   |  - FileDescriptor[MAX_OPEN_FILES]                                   |   |
|   |  - Bound CapType::File(vnode_id) (REQ-KRN-106)                      |   |
|   |  - Dynamic rights verification: READ / WRITE (INV-VFS-001)          |   |
|   |  - Revocation & attenuation enforcement                             |   |
|   |  (kernel/src/vfs/file.rs & core.rs)                                 |   |
|   +---------------------------------------------------------------------+   |
+-----------------------------------------------------------------------------+
```

### 3.1 Vnode Layer (`vnode.rs`)
- [`VnodeType`](file:///Users/dhan/IndOS/kernel/src/vfs/vnode.rs): Discriminated union of filesystem entity classes:
  - `RegularFile`: Data container with byte-level read/write.
  - `Directory`: Hierarchical container maintaining child node associations.
  - `Device`: Special character or block device node (e.g. `/dev/null`).
  - `Symlink`: Symbolic path alias.
- [`Vnode`](file:///Users/dhan/IndOS/kernel/src/vfs/vnode.rs): Fixed-capacity, allocation-free kernel structure holding metadata (`id`, `vtype`, `size`, `name`, `parent_id`) and embedded storage buffers for early boot files.

### 3.2 Mount Table (`mount.rs`)
- [`MountEntry`](file:///Users/dhan/IndOS/kernel/src/vfs/mount.rs): Associations between path prefixes (e.g., `/`, `/dev`, `/etc`) and corresponding root vnode IDs.
- [`MountTable`](file:///Users/dhan/IndOS/kernel/src/vfs/mount.rs): Supports up to `MAX_MOUNTS` (8) filesystems with longest-prefix-match resolution.

### 3.3 Path Resolution (`path.rs`)
- [`PathResolver::canonicalize`](file:///Users/dhan/IndOS/kernel/src/vfs/path.rs):
  1. Tokenizes path by `/`.
  2. Eliminates redundant consecutive `/` and current directory `.` references.
  3. Evaluates parent directory references `..` by decrementing directory depth stack.
  4. Bounds lower limit of stack depth to 0 (`/`), preventing upward escape (`INV-VFS-002`).
  5. Serializes normalized path with leading `/` into caller-provided buffer without dynamic allocation.

### 3.4 Capability-Bound File Descriptors (`file.rs`)
- [`FileDescriptor`](file:///Users/dhan/IndOS/kernel/src/vfs/file.rs):
  - In-kernel handle storing file offset and granted `CapRights`.
  - `verify_read_access(&self, cap: &Capability) -> Result<(), &'static str>`: Validates that `cap.is_valid`, `cap.cap_type == CapType::File(vnode_id)`, and `cap.rights.contains(CapRights::READ)`.
  - `verify_write_access(&self, cap: &Capability) -> Result<(), &'static str>`: Validates that `cap.is_valid`, `cap.cap_type == CapType::File(vnode_id)`, and `cap.rights.contains(CapRights::WRITE)`.

### 3.5 Core State Machine (`core.rs`)
- [`Vfs`](file:///Users/dhan/IndOS/kernel/src/vfs/core.rs):
  - Pre-allocated vnode table (`[Vnode; 64]`) and open file table (`[FileDescriptor; 32]`).
  - Zero dynamic heap allocation for maximum determinism and safety.
  - Methods:
    - `init()`: Establishes root filesystem hierarchy (`/`, `/dev`, `/dev/null`, `/etc`, `/etc/version`).
    - `lookup(&self, path: &str) -> Result<VnodeId, &'static str>`.
    - `create_file(&mut self, dir_path: &str, name: &str) -> Result<VnodeId, &'static str>`.
    - `open_file(&mut self, path: &str, rights: CapRights) -> Result<(u32, Capability), &'static str>`.
    - `read_file(&mut self, fd: u32, cap: &Capability, buf: &mut [u8]) -> Result<usize, &'static str>`.
    - `write_file(&mut self, fd: u32, cap: &Capability, buf: &[u8]) -> Result<usize, &'static str>`.
    - `seek(&mut self, fd: u32, offset: usize) -> Result<(), &'static str>`.
    - `close_file(&mut self, fd: u32) -> Result<(), &'static str>`.

---

## 4. Verification & Testing

The VFS Core subsystem is validated during microkernel boot across Stages 62–65:
- **Stage 62 (`62_VFS_CORE_INIT`):** Confirms vnode table allocation, root filesystem mounting, and baseline hierarchy setup.
- **Stage 63 (`63_VFS_PATH_RESOLUTION`):** Validates normalization (`/dev/../etc/./version` $\rightarrow$ `/etc/version`), lookup of `/dev/null` and `/etc/version`, and jail containment (`/../../` $\rightarrow$ `/`).
- **Stage 64 (`64_VFS_CAPABILITY_FILE_ACCESS`):** Validates file creation, capability minting, authorized read/write, seek, and adversarial rejection of write without `WRITE` rights and read with revoked capabilities.
- **Stage 65 (`65_VFS_CORE_READY`):** Reaches the SP-24 milestone cleanly.
