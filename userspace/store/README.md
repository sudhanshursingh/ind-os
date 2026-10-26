# Ind OS Settings & Consent Store (`stored`)
**Sprint:** SP-37  
**Requirements Traceability:** `REQ-USE-206`  
**Invariants:** `INV-STR-001` (Tamper-Evident Integrity Protection), `INV-STR-002` (User-Owned Consent Authority)  
**Standard:** `#![no_std]` Rust, Zero external crates (Rule N-01), Zero unsafe code (Rule N-04)

## Architecture Overview
The settings and consent store daemon (`stored`) provides:
1. Integrity-protected persistence for system configurations and security toggles.
2. Tamper-evident checksum validation rejecting unauthorized mutations (`INV-STR-001`).
3. User-owned consent authority preventing ambient privilege escalation (`INV-STR-002`).
4. Ground-truth evaluation engine queried by the ENF Policy Engine.
