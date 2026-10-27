# Ind OS SP-104 Architecture Specification
## Hardware Compatibility Matrix v1 Specification

**Sprint:** SP-104 (Stage 470–474)  
**Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; PRD-001 §8.3, REQ-HW-114  
**Target Hardware:** Motorola Moto G60 (Qualcomm Snapdragon 732G SM7150-AC, `hanoip`)  
**Status:** APPROVED FOR IMPLEMENTATION

---

### 1. Purpose and Scope

Sprint SP-104 formalizes and publishes the comprehensive Hardware Compatibility Matrix v1 for Ind OS on the Motorola Moto G60 reference platform (`hanoip`), categorizing all 15 major system components into deterministic status buckets per PRD-001 §8.3:
- **WORKS:** Fully validated, operational on bare metal with deterministic bounds.
- **PARTIAL:** Basic initialization/probing verified; advanced pipeline staged for Phase 10.
- **BLOCKED:** Blocked by upstream hardware defect or missing documentation.
- **OUT-OF-SCOPE-WITH-REASON:** Deliberately excluded to maintain constitutional compliance (e.g., Rule N-01 zero proprietary closed binaries).

---

### 2. Architectural Invariants

- **INV-HW-MAT-001 (Deterministic Peripheral Classification):**  
  Every hardware device on the SM7150-AC SoC must have a formal status record with peripheral identifier, bus location, capability access token, and verified status.

- **INV-HW-MAT-002 (TCB Minimum Operating Subset):**  
  All 12 foundational subsystems required for Gate G9 (CPU, DRAM, UART, UFS, Display, Touch, PMIC, USB, Wi-Fi, BT, Sensors, Audio) must be classified strictly as `WORKS`.

- **INV-HW-MAT-003 (Constitutional Proprietary Blob Exclusion):**  
  Proprietary DSP/NPU binary firmware blobs (Hexagon cDSP) are categorized as `OUT_OF_SCOPE_WITH_REASON` under Rule N-01 (zero closed-source blobs in TCB). Quantized CPU NEON inference (verified in SP-96) serves as the open-source, reproducible replacement.

---

### 3. Constitutional Conformance

- **Rule N-01:** Zero external crates and zero proprietary blobs in TCB.
- **Rule N-04:** Pure memory-safe Rust in userspace implementation (`userspace/init/src/hw_matrix.rs`).
