# Ind OS Software Bill of Materials (SBOM) - Phase 8: UX Shell + App Framework

## 1. Executive Summary
- **Target Release:** USER-READY OS V0.1 (Gate G8 Ratification).
- **Format:** CycloneDX 1.5 JSON ([docs/evidence/sbom-phase8.json](../evidence/sbom-phase8.json)).
- **Compliance Standards:**
  - Rule N-01: Zero external crates in TCB (100% in-tree Rust).
  - Rule N-04: Zero unsafe blocks in `userspace/ux` crate.

---

## 2. In-Tree Components Inventory

| Component | Scope | TCB | Safe Rust (Rule N-04) | Description |
| :--- | :--- | :--- | :--- | :--- |
| **`indk-core`** | Kernel | Yes | Controlled Unsafe in Arch HAL | ARM64 microkernel, capability engine, display & app gates |
| **`libindus`** | Userspace | No | 100% Safe | System calls, IPC wrappers, capability handles |
| **`indus-logd`** | Userspace | No | 100% Safe | System logger & Why-Engine causal explanation daemon |
| **`indus-net`** | Userspace | No | 100% Safe | Non-TCB network stack, TLS, and Leak Sentinel |
| **`indus-ux`** | Userspace | No | 100% Safe | Compositor, Shell v2, Intent UI, A11y, Core Apps, Polish |

---

## 3. Cryptographic Verification
- Component hashes verified against in-tree sources.
- Zero untracked or external crates referenced across workspace `Cargo.toml` manifests.
