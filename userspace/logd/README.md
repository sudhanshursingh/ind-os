# Ind OS Unified Log Bus & Diagnostics (`indus-logd`)
**Sprint:** SP-36  
**Requirements Traceability:** `REQ-USE-204`  
**Invariants:** `INV-LOG-001` (Privacy Redaction Guarantee), `INV-LOG-002` (Explainability & Causal Root Cause Reconstruction)  
**Standard:** `#![no_std]` Rust, Zero external crates (Rule N-01), Zero unsafe code (Rule N-04)

## Architecture Overview
The logging and diagnostics daemon (`logd`) provides:
1. Unified structured log ingestion bus across all subsystems (`kernel`, `init`, `devmgr`, `vfs`, `shell`).
2. Redaction filter engine enforcing privacy and zero cleartext leak of capability tokens or cryptographic keys.
3. Diagnostic causal explanation engine implementing the `why` query interface.
