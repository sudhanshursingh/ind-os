# Ind OS Architecture Specification: Developer OS v0.1 Quality & Packaging

**Document ID:** `INDOS-SPEC-PKG-01`  
**Sprint:** SP-40 (Quality Sprint & Developer OS v0.1 Packaging)  
**Phase:** Phase 5 (Userspace & the Developer OS)  
**Constitutional Authority:** Ind OS Development Bible v1.0.0.0 §8.11  
**Status:** **RATIFIED**  
**Classification:** Core System Architecture Specification  

---

## 1. Scope & Objective

Sprint SP-40 represents the culmination of **Phase 5 (Userspace & the Developer OS)**, preparing Ind OS for the hard-stop Gate G5 audit and the formal declaration of **DEVELOPER OS v0.1**.

The objectives are:
1. **Quality & Stability Pass:** Audit and eliminate regressions across all userspace services (`init`, `libindus`, `devmgr`, `shell`, `logd`, `store`, `edit`, `testsuite`).
2. **Resource Leak Sweep (`INV-PKG-001`):** Complete system-wide verification of zero capability handle leakage, zero unbounded memory allocation, and zero zombie task state.
3. **Software Bill of Materials & Architecture Parity (`INV-PKG-002`):** Complete SBOM generation accounting for 100% in-tree pure-Rust components (Rule N-01), verified against the Ind OS PRD and Bible.
4. **Developer OS v0.1 Packaging:** Bundling of the microkernel ELF (`indk`), baseline service manifests, and user binaries into a reproducible release image.

---

## 2. Invariants & Architectural Rules

### 2.1 Invariant `INV-PKG-001`: Whole-System Zero-Leak Watermark
Upon completion of multi-subsystem execution and soak cycles, the system-wide resource auditor MUST verify that:
- Total active capability references match registered subsystem allocations.
- Dynamic heap high-water mark does not grow across repeat cycles.
- All temporary channels, pipes, and event slots are cleanly drained and closed.

### 2.2 Invariant `INV-PKG-002`: 100% Manifest & Capability Audit
Every userspace service deployed in Developer OS v0.1 MUST:
- Possess a declarative manifest detailing Runlevel, executable path, restart backoff, and dependency requirements.
- Strictly operate under capability-mediated authority with ZERO ambient system calls.
- Emit structured telemetry records to the unified log bus (`logd`).

---

## 3. Developer OS v0.1 Packaging Manifest

The Developer OS v0.1 bundle consists of:
- **Core Microkernel:** `indk` (ARM64 EL1 microkernel, pure Rust).
- **Userspace Baseline Suite:**
  - `init` (PID 1 service supervisor and dependency resolver)
  - `libindus` (`indus-core` runtime, collections, IPC bindings)
  - `devmgr` (device manager, VirtIO hotplug bus, capability issuer)
  - `shell` (interactive command processor with pipeline IPC)
  - `logd` (unified structured logging, privacy redaction, `why` engine)
  - `store` (integrity-protected settings & consent store)
  - `edit` (modal in-OS text editor)
  - `testsuite` (scripted whole-system integration test runner)
- **Configuration & Golden Data:**
  - `manifests/baseline.json`
  - `docs/evidence/golden-boot-*.log`

---

## 4. Verification & Validation Protocol

SP-40 execution is validated through:
- In-kernel boot stages 146 through 150 verifying leak sweeps, SBOM audit, and packaging readiness.
- Standalone packaging verification script `tools/package_developer_os.sh`.
- Automated test script `ci/verify_sp40.py`.
- Cryptographic artifact manifest `evidence/SP-40/manifest.json`.
