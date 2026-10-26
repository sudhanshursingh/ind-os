# Sprint SP-32 Validation & Acceptance Report
**Sprint:** SP-32 (Userspace Init: Service Manifests, Dependency Graph, Crash-Restart with Backoff, Structured Boot Logs)  
**Phase:** Phase 5 (Userspace & the Developer OS)  
**Date:** 2026-10-27  
**Status:** **PASS / RATIFIED**  
**Requirements Traceability:** `REQ-USE-201`  
**Invariants Verified:** `INV-INIT-001` (Deterministic Dependency Ordering), `INV-INIT-002` (Crash-Restart Containment & Exponential Backoff)  
**Auditor:** Sudhanshu Raj Singh (`sudhanshurajsingh@users.noreply.github.com`)

---

## 1. Executive Summary

Sprint SP-32 marks the commencement of **Phase 5 (Userspace & the Developer OS)**. This sprint stands up the root userspace supervisor (**PID 1: `init`** / `indus-init`) providing declarative service orchestration, strict dependency resolution, and resilient fault isolation:
1. **Declarative Service Manifests (`REQ-USE-201`):** Manifests declare executable binaries, runlevels (`CoreSecurity`, `StorageDrivers`, `SystemServices`, `InteractiveShell`), restart policies (`Always`, `OnFailure`, `Never`), capability grants, and dependency prerequisites.
2. **Topological Dependency Graph (`INV-INIT-001`):** Resolves startup ordering using in-degree elimination (Kahn's algorithm). Cycles (e.g. $A \to B \to A$) are deterministically trapped and rejected with `DependencyCycleDetected`.
3. **Fault-Tolerant Crash-Restart with Exponential Backoff (`INV-INIT-002`):** Unscheduled process terminations trigger bounded supervisor backoff ($20\text{ ms} \to 40\text{ ms} \to 80\text{ ms}$) up to `max_retries`, ensuring cascading faults do not destabilize the system.
4. **Structured Boot Stage Mirroring:** Process state transitions are recorded into structured diagnostic logs and mirrored into the Kernel Telemetry ring buffer (`REQ-KRN-113`).
5. **Standalone Workspace Package:** Created [`userspace/init/`](file:///Users/dhan/IndOS/userspace/init/) (`indus-init`) compiling as an independent `#![no_std]` binary and library within the root Cargo workspace.

---

## 2. Invariant & Requirement Verification Matrix

| Requirement / Invariant | Governing Section | Description | Verification Method | Status |
|---|---|---|---|---|
| **`REQ-USE-201`** | Bible §8.11 | Declarative service manifests across 4 runlevels | Kernel Stage 106 & Manifest validation | **PASSED** |
| **`INV-INIT-001`** | Bible §8.11 / Spec §3 | Deterministic dependency DAG ordering & cycle detection | Kernel Stage 107; cycle rejected with `DependencyCycleDetected` | **PASSED** |
| **`INV-INIT-002`** | Bible §8.11 / Spec §4 | Service crash containment & exponential backoff | Kernel Stage 108; $20\text{ ms} \to 40\text{ ms}$ backoff verified | **PASSED** |
| **`REQ-KRN-113`** | Bible §9.3 | Structured boot stage mirroring to telemetry ring buffer | Kernel Stage 109; transition records captured & mirrored | **PASSED** |
| **Rule N-01** | Bible Rule N-01 | Zero external dependencies (pure Rust) | Static cargo tree & manifest check | **PASSED** |
| **Rule N-04** | Bible Rule N-04 | Zero unsafe code in new modules | Static AST scanner (0 unsafe blocks in `userspace/init` & `kernel/src/userspace`) | **PASSED** |
| **Rule N-10** | Bible Rule N-10 | Signed-off-by Git commit protocol | DCO compliant | **PASSED** |

---

## 3. Kernel Boot Verification (Stages 106–110)

From `docs/evidence/golden-boot-sp32.log`:
```
[STAGE:106_INIT_SERVICE_MANIFESTS] Declarative service manifests, runlevels, and capability grants verified
[INFO ] init: Service manifests validated: 4 baseline services loaded across 4 runlevels (REQ-USE-201 PASSED)
[STAGE:107_INIT_DEPENDENCY_GRAPH] DAG topological sorting and cycle detection verified
[INFO ] init: Dependency graph verified: Topological ordering resolved [auditd -> vfs-indfs -> devmgr -> shell]
[INFO ] init: Cycle detection verified: A <-> B cycle rejected with DependencyCycleDetected (INV-INIT-001 PASSED)
[STAGE:108_INIT_CRASH_RESTART] Service crash supervision, fault containment, and exponential backoff verified
[INFO ] init: Crash-restart verified: devmgr recovered with exponential backoff (20ms -> 40ms) (INV-INIT-002 PASSED)
[STAGE:109_INIT_STRUCTURED_LOG_MIRROR] Structured boot logging and telemetry event mirroring verified
[INFO ] init: Structured boot logging verified: 3 transition records captured and mirrored to telemetry (REQ-USE-201 PASSED)
[STAGE:110_SP32_INIT_READY] Sprint SP-32 Userspace Init (PID 1) Milestone REACHED
```

---

## 4. Conclusion & DoD Sign-Off

Sprint SP-32 satisfies 100% of its Definition of Done (DoD) criteria. Phase 5 is underway towards the **Developer OS v0.1** milestone at Gate G5.
