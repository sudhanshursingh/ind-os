# UFS Power-Loss Resilience & Journaling Specification (SP-75)

- **Sprint:** SP-75 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** Qualcomm UFS 2.2 Storage Resilience & Crash-Consistent Journaling
- **Document ID:** SPEC-ARCH-SP75-UFS-RESILIENCE
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, REQ-HW-105, REQ-STR-102

---

## 1. Objectives & Scope

Sprint SP-75 extends the Qualcomm UFS 2.2 host controller driver (SP-74) with power-loss resilience, write barriers, and write-ahead intent log (WAL) rollback mechanisms. On physical flash storage, abrupt power cuts during block writes risk corrupting file system superblocks, inode trees, and partition tables unless hardware flush commands (SCSI `SYNCHRONIZE CACHE`) and transactional write journals are enforced.

---

## 2. Architectural Invariants

### INV-UFS-JRN-001: Hardware Write Barrier & Sync Cache Enforcement
All transactional updates must issue a SCSI `SYNCHRONIZE_CACHE` (0x35) command to the UFS device, guaranteeing that all volatile controller buffers are committed to persistent NAND flash before declaring a transaction committed.

### INV-UFS-JRN-002: Atomic Journal Rollback & Checksum Verification
Every journal entry contains a header, sequence number, target LBA, payload, and CRC32/SHA256 checksum. If a simulated sudden power loss interrupts a transaction, recovery re-reads the journal, detects incomplete or corrupt entries, and rolls back state to the last consistent checkpoint.

### INV-UFS-JRN-003: Recovery-First Partition Write Protection
Partitions flagged as critical (`boot_a`, `boot_b`, `vendor_boot`, `recovery`, `persist`) are hardware-write-protected or gated by strict kernel capability checks (`CAP_STORAGE_RECOVERY`). System partitions cannot be modified without cryptographic authorization and dual-slot A/B safety.

---

## 3. Interfaces & Data Structures

- `UfsJournalEntry`: Represents an atomic storage transaction record with LBA, sector count, CRC32, and commit flag.
- `UfsJournalHarness`: Simulates and verifies write-ahead logging, sudden power loss disruption, and atomic journal playback.
- `KernelUfsJournalHarness`: In-kernel implementation validating write barriers and rollback safety.
