# Ind OS SP-105 Architecture Specification
## Recovery Route Re-Tested After Experiments

**Sprint:** SP-105 (Stage 475–479)  
**Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; Rule N-06, REQ-SEC-111, REQ-REC-102  
**Target Hardware:** Motorola Moto G60 (Qualcomm Snapdragon 732G SM7150-AC, `hanoip`)  
**Status:** APPROVED FOR IMPLEMENTATION

---

### 1. Purpose and Scope

Rule N-06 mandates: *"A hardware development journey is illegitimate if the device cannot be restored to known stock state or unbricked via documented recovery mechanisms."*

Following the deep bare-metal experiments of Phase 9 (UFS partition writing, PMIC power rail manipulation, display DSI overclocking to 120Hz, and A/B slot flipping), Sprint SP-105 explicitly re-verifies that the recovery mechanisms (Fastboot, EDL test points, and full stock image restoration) remain 100% operational and reproducible.

---

### 2. Architectural Invariants

- **INV-RECOV-001 (Fastboot Protocol Integrity):**  
  The secondary bootloader (SBL/ABOOT) fastboot interface must respond to critical commands (`getvar:current-slot`, `getvar:has-slot:boot`, `flash:boot`, `continue`) with zero command stalls under capability token `CAP_RECOVERY_ADMIN` (`0x0008_0000`).

- **INV-RECOV-002 (EDL Test-Point & Sahara Protocol Proof):**  
  The Emergency Download mode (Qualcomm HS-USB QDLoader 9008) triggered via physical test points (TP_FORCE_USB_BOOT) must complete the Sahara hello handshake (cmd `0x01`) and firehose payload staging.

- **INV-RECOV-003 (Deterministic Stock Restoration Verification):**  
  The original stock Motorola G60 factory firmware image (`boot.img`, `system.img`, `vbmeta.img`) is verified against its master SHA-256 and CRC32 checksums, proving that stock restoration can be executed in under 120 seconds.

---

### 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** Pure memory-safe Rust in userspace implementation (`userspace/init/src/recovery_audit.rs`).
- **Rule N-06 / PRD-001 §7.5:** Mandatory unbrick and stock restoration route rigorously verified.
