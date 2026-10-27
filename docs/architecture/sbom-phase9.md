# Ind OS Software Bill of Materials (SBOM) - Phase 9: Moto G60 Bring-Up

## 1. Executive Summary
- **Target Release:** MOTO G60 BRING-UP / HARDWARE V0.1 (Gate G9 Ratification).
- **Format:** CycloneDX 1.5 JSON ([docs/evidence/sbom-phase9.json](../evidence/sbom-phase9.json)).
- **Compliance Standards:**
  - Rule N-01: Zero external crates in TCB (100% in-tree Rust across kernel and userspace).
  - Rule N-04: Zero unsafe blocks in newly authored userspace modules (`userspace/init`).
  - Rule N-06 / PRD-001 §7.5: Hardware verification with non-destructive recovery.

---

## 2. In-Tree Components Inventory

| Component | Scope | TCB | Safe Rust (Rule N-04) | Description |
| :--- | :--- | :--- | :--- | :--- |
| **`aarch64-indk`** | Kernel | Yes | Controlled Unsafe in Arch HAL | ARM64 microkernel, Qualcomm SM7150 HAL, capability engine, display & touch drivers |
| **`libindus`** | Userspace | No | 100% Safe | System calls, IPC wrappers, capability handles |
| **`indus-init`** | Userspace | No | 100% Safe | Root supervisor, dependency graph, hardware coordinator, soak engine, cold-boot verifier, G9 pack |
| **`indus-logd`** | Userspace | No | 100% Safe | System logger & Why-Engine causal explanation daemon |
| **`indus-store`** | Userspace | No | 100% Safe | Settings and consent store with tamper-evident checksums |
| **`indus-ux`** | Userspace | No | 100% Safe | Compositor, Shell v2, Intent UI, A11y, on-device physical display pipeline |

---

## 3. Cryptographic Verification
- Component hashes verified against in-tree sources.
- **TCB External Dependencies Count:** Exactly 0 external crates.
- **Userspace Dependencies Count:** Exactly 0 external crates.
