# Ind OS Architectural Specification: Integration Test Suite v1 (`REQ-QA-201` Groundwork)
**Document Identifier:** `INDOS-SPEC-QA-01`  
**Sprint:** SP-39  
**Phase:** Phase 5 (Userspace & the Developer OS)  
**Status:** **APPROVED / ACTIVE**  
**Requirements Traceability:** `REQ-QA-201` Groundwork (Bible §8.11)  
**Invariants Enforced:** `INV-QA-001` (Deterministic User Journey Execution), `INV-QA-002` (Zero-Leak Soak Cycling)  
**Author:** Sudhanshu Raj Singh (`sudhanshurajsingh@users.noreply.github.com`)

---

## 1. Scope & Objective

This specification governs the architecture and execution harness of the **Whole-System Integration Test Suite v1 (`indus-testsuite`)** per `REQ-QA-201`.

The suite coordinates cross-subsystem scripted journeys validating that `init`, `libindus`, `devmgr`, `shell`, `logd`, and `store` work collaboratively as a cohesive Developer OS.

---

## 2. Invariants

### 2.1 Invariant `INV-QA-001`: Deterministic User Journey Execution
- Scripted user journeys execute end-to-end scenarios reproducing authentic developer workflows.
- Every journey verifies expected precondition, intermediate capability exchanges, and postcondition invariants with zero non-deterministic timing flakes.

### 2.2 Invariant `INV-QA-002`: Zero-Leak Soak Cycling
- The soak test harness executes repeated cycles of process spawning, capability minting, file I/O, and tear-down.
- Heap watermarks, capability table counts, and descriptor allocations must remain bounded with zero permanent resource leaks.

---

## 3. Scripted User Journeys

1. **Journey 1: System Boot & Supervision:** Validates declarative manifest loading, dependency graph topological ordering, and crash-restart backoff.
2. **Journey 2: Dynamic Peripheral & Capabilities:** Validates hotplug event broadcasting, scoped MMIO/IRQ capability delegation, and revocation.
3. **Journey 3: Developer Shell Session:** Validates command parsing, diagnostic execution, and IPC pipeline chaining.
4. **Journey 4: Diagnostics & Root Cause Analysis:** Validates log ingestion, privacy redaction filters, and `why` diagnostic root-cause explanation.
5. **Journey 5: Integrity-Protected Settings & Consent:** Validates FNV-1a checksum verification and rejection of unauthorized ambient writes.
