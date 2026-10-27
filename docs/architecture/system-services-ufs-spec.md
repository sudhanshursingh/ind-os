# Ind OS Architecture Specification: System Services Hardware Port — IndFS Mount on UFS (Sprint SP-99)

- **Sprint:** SP-99 (Phase 9: Moto G60 Bring-Up)
- **Component:** IndFS Root Mount & System Services Storage Hardware Port
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2 (REQ-SYS-101, REQ-STR-103, INV-SYS-FS-001..003, Rule N-01, Rule N-04, Rule N-06).

---

## 1. Architectural Role & Motivation

Following hardware storage driver bring-up (SP-74/75), Sprint SP-99 transitions the operating system from ramdisk/initrd memory mounts to mounting IndFS directly on physical Qualcomm UFS 2.2 storage (`/dev/ufs0p1`). This establishes persistent storage for system services (`init`, `store`, `logd`, and app configuration).

---

## 2. Invariants & Hardware Contracts

- **INV-SYS-FS-001 (Capability-Gated UFS Root Mount):**
  Mounting IndFS on physical UFS block device `0x0001_0000` is capability-gated by `CAP_STORAGE_MOUNT` (`0x0001_0000`). Unprivileged mount invocations fail with `PermissionDenied`.

- **INV-SYS-FS-002 (Superblock Validation & Journal Replay):**
  Mount verification validates IndFS superblock magic `0x494E_4446` ("INDF"), block size 4096 bytes, and replays uncommitted transaction journals to guarantee zero fsck errors upon hardware cold boot.

- **INV-SYS-FS-003 (System Hierarchy Namespaces):**
  Mounts `/sys`, `/dev`, `/cfg`, and `/data` directories into the root VFS with appropriate least-privilege capability protections.

---

## 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** 100% memory-safe Rust in userspace storage services.
- **Rule N-06:** Deterministic mount checks with crash recovery.
