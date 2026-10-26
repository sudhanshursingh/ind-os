# Ind OS Whole-System Integration Test Suite v1 (`testsuite`)
**Sprint:** SP-39  
**Requirements Traceability:** `REQ-QA-201` Groundwork (Bible §8.11)  
**Invariants:** `INV-QA-001` (Deterministic User Journey Execution), `INV-QA-002` (Zero-Leak Soak Cycling)  
**Standard:** `#![no_std]` Rust, Zero external crates (Rule N-01), Zero unsafe code (Rule N-04)

## Architecture Overview
The integration test suite (`testsuite`) coordinates whole-system verification across Phase 5:
1. Scripted User Journeys 1–5 validating cross-subsystem orchestration.
2. Soak test harness performing repeated capability allocation/release cycles without resource exhaustion.
3. Clean-checkout reproduction timing validation.
