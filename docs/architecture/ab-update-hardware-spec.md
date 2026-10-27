# Ind OS SP-102 Architecture Specification
## Hardware A/B Dual-Slot Update Service on UFS Partitions

**Sprint:** SP-102 (Stage 460–464)  
**Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-SEC-112, REQ-STR-104  
**Target Hardware:** Motorola Moto G60 (Qualcomm Snapdragon 732G SM7150-AC, UFS 2.2 dual-slot partition table: `boot_a`/`boot_b`, `system_a`/`system_b`)  
**Status:** APPROVED FOR IMPLEMENTATION

---

### 1. Purpose and Scope

Sprint SP-102 implements the hardware A/B dual-slot update mechanism on physical UFS storage. It guarantees atomic system updates without bricking risk, implementing Qualcomm standard dual-slot metadata management (Active slot, Bootable flag, Successful flag, and Tries-Remaining counter) along with hardware fallback/rollback on watchdog failure.

---

### 2. Architectural Invariants

- **INV-OTA-AB-001 (Atomic Slot State Management):**  
  Slot metadata (`SlotInfo { slot: SlotId, is_active: bool, is_successful: bool, tries_remaining: u8 }`) is stored in dedicated non-volatile UFS metadata blocks. Modifications require capability token `CAP_STORAGE_ADMIN` (`0x0002_0000`).

- **INV-OTA-AB-002 (Cryptographic Integrity Verification):**  
  Payload chunks written to the inactive slot must be verified against their cryptographic digest (SHA-256) prior to slot activation. Unsigned or corrupted images are rejected with `AbUpdateError::IntegrityMismatch`.

- **INV-OTA-AB-003 (Rollback Trigger & Tries-Remaining Countdown):**  
  When flipping the active slot to Slot B, `tries_remaining` is initialized to 3 and `is_successful` is set to false. If the new slot fails to mark itself successful within 3 boot attempts, the bootloader automatically reverts to Slot A.

---

### 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** Pure memory-safe Rust in userspace implementation (`userspace/store/src/ab_update.rs`).
- **Rule N-06 / PRD-001 §7.5:** Fastboot/EDL recovery paths remain preserved; dual-slot layout prevents bricking.
