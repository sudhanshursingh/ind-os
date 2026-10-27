# Ind OS Architecture Specification: AIS Model Runtime Sandbox & Hardware Profile Manager

**Document ID:** `INDOS-SPEC-AIS-01`  
**Sprint:** SP-41 (Model Runtime Sandbox & Dynamic Hardware Profile Manager)  
**Phase:** Phase 6 (AIS Subsystem + ENF Policy Engine + Consent Broker)  
**Constitutional Authority:** Ind OS Development Bible v1.0.0.0 §8.12, §9.7 (REQ-AI-101, REQ-AI-112, RFC-INDOS-2026-001)  
**Status:** **RATIFIED**  
**Classification:** Core System Architecture Specification  

---

## 1. Scope & Objective

Phase 6 initiates the AI-native subsystem of Ind OS. The model runtime sandbox is the foundational layer ensuring that all on-device intelligence executes inside the Phase 3 capability and enforcement spine:
1. **Model Runtime Sandbox (`REQ-AI-101`):** An isolated, unprivileged execution context for AI inference models with zero ambient authority, strictly enforced compute limits, and bounded memory quotas.
2. **Signed-Model Loader Only (`REQ-AI-101`, `REQ-AI-109`):** Mandatory cryptographic integrity verification ensuring only authenticated, signed model weights can be loaded into the sandbox. Unsigned, corrupt, or tampered models are rejected deterministically.
3. **Dynamic Hardware Profile Manager (`REQ-AI-112`, `IndHardwareProfile`):** Telemetry-driven inference tiering operating in userspace (`userspace/ais/`). Discovers total physical RAM via read-only kernel telemetry (`REQ-KRN-113`) and selects the appropriate model tier (`Budget` $\le 6$ GB, `Standard` 6–12 GB, or `Flagship` $\ge 12$ GB) to eliminate memory strain on constrained devices while maximizing flagship performance.

---

## 2. Invariants & Architectural Rules

### 2.1 Invariant `INV-AIS-001`: Signed-Model Loader Only
The model loader MUST verify the cryptographic signature and manifest digest of the model artifact prior to parsing or allocating tensor buffers:
- Unsigned binaries are rejected with `SignatureRequired`.
- Tampered or corrupted model weights are rejected with `IntegrityMismatch`.
- Model loading requires an explicit capability token with `CAP_AIS_LOAD` rights.

### 2.2 Invariant `INV-AIS-002`: Bounded Capability Sandbox & Quota Enforcement
The inference sandbox MUST run as an unprivileged process with strictly bounded resources:
- Zero ambient syscalls (no direct hardware, socket, or disk access).
- CPU backend inference with bounded execution time slice (tick limits).
- Memory allocation bounded by tier quota (e.g., $\le 1250$ MB RAM on $\le 6$ GB tier).
- Exceeding compute or memory budget triggers safe preemption or truncation, never a kernel fault.

### 2.3 Invariant `INV-AIS-003`: Deterministic Hardware Profile Tiering
`IndHardwareProfile` queries the read-only kernel telemetry ring (`REQ-KRN-113`) at startup and deterministically configures:
- **Tier Budget ($\le 6$ GB RAM):** Model `Q4_K_M` 1.5B, 2048 context length, 128 MB KV-cache, RAM limit 1250 MB.
- **Tier Standard ($> 6$ GB and $< 12$ GB RAM):** Model `Q4_K_M` 3B, 4096 context length, 256 MB KV-cache, RAM limit 3000 MB.
- **Tier Flagship ($\ge 12$ GB RAM):** Model `Q8_0` 7B/8B, 8192 context length, 512 MB KV-cache, RAM limit 6000 MB, accelerated compute.

---

## 3. Data Structures & Subsystem Architecture

### 3.1 Hardware Profile Descriptor
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareTier {
    Budget,   // <= 6 GB
    Standard, // 6 GB - 12 GB
    Flagship, // >= 12 GB
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationType {
    Q4_K_M,
    Q8_0,
}

#[derive(Debug, Clone, Copy)]
pub struct InferenceConfig {
    pub tier: HardwareTier,
    pub quantization: QuantizationType,
    pub parameter_billions_x10: u32, // e.g. 15 for 1.5B
    pub context_tokens: u32,
    pub kv_cache_mb: u32,
    pub ram_limit_mb: u32,
}
```

### 3.2 Signed Model Manifest
```rust
pub struct ModelManifest {
    pub model_id: u32,
    pub name: &'static str,
    pub version: u32,
    pub weights_hash: u32,
    pub signature_token: u32,
    pub required_ram_mb: u32,
}
```

---

## 4. Verification & Acceptance Protocol

The implementation of SP-41 is validated through:
1. Standalone `#![no_std]` crate [`userspace/ais/`](file:///Users/dhan/IndOS/userspace/ais/) (`indus-ais`).
2. Kernel boot verification stages 152–156 in [`kernel/src/main.rs`](file:///Users/dhan/IndOS/kernel/src/main.rs).
3. QEMU execution log [`docs/evidence/golden-boot-sp41.log`](file:///Users/dhan/IndOS/docs/evidence/golden-boot-sp41.log).
4. Validation report [`docs/evidence/SP-41-validation.md`](file:///Users/dhan/IndOS/docs/evidence/SP-41-validation.md).
5. Automated CI verifier [`ci/verify_sp41.py`](file:///Users/dhan/IndOS/ci/verify_sp41.py).
