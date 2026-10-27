# `indus-ais` — Ind OS AI Subsystem Runtime Sandbox & Hardware Profile Manager

**Standards & Invariants:**
- `REQ-AI-101`: Model Runtime Sandbox (signed-model loader, isolated inference, CPU backend, memory/time limits)
- `REQ-AI-112`: Dynamic Hardware Profile Manager (`IndHardwareProfile`, RFC-INDOS-2026-001)
- `INV-AIS-001`: Signed-model loader only (cryptographic signature verification)
- `INV-AIS-002`: Bounded capability sandbox & resource quota enforcement
- `INV-AIS-003`: Deterministic telemetry-driven hardware profile tiering
- **Rule N-01:** Zero external crates
- **Rule N-04:** Zero unsafe code in new modules

## Subsystem Architecture
1. **Dynamic Hardware Profile Manager (`profile.rs`):** Discovers memory via kernel telemetry (`REQ-KRN-113`) and dynamically configures model parameters, KV cache, and memory ceiling without kernel modifications.
2. **Signed Model Loader (`loader.rs`):** Validates digital signatures and weight checksums before instantiating model tensors.
3. **Model Runtime Sandbox (`sandbox.rs`):** Constrains execution cycles, manages bounded workspace memory, and provides deterministic CPU inference.
