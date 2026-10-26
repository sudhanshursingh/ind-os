# Ind OS Architectural Specification: Settings & Consent Store (`REQ-USE-206`)
**Document Identifier:** `INDOS-SPEC-STR-01`  
**Sprint:** SP-37  
**Phase:** Phase 5 (Userspace & the Developer OS)  
**Status:** **APPROVED / ACTIVE**  
**Requirements Traceability:** `REQ-USE-206`  
**Invariants Enforced:** `INV-STR-001` (Tamper-Evident Integrity Protection), `INV-STR-002` (User-Owned Consent Authority)  
**Author:** Sudhanshu Raj Singh (`sudhanshurajsingh@users.noreply.github.com`)

---

## 1. Scope & Objective

This specification governs the architecture and security properties of the **Settings & Consent Store (`stored` / `indus-store`)** per `REQ-USE-206`.

The store is the foundational repository of user preferences, security toggles, service authorization policies, and explicit consent grants. It provides the immutable ground-truth from which the **ENF Policy Engine** (Phase 6) resolves authorization decisions.

---

## 2. Invariants

### 2.1 Invariant `INV-STR-001`: Tamper-Evident Integrity Protection
- All setting entries and consent records contain an integrity checksum/MAC computed over key, value, and timestamp.
- On reading any setting or consent decision, the store verifies its cryptographic integrity.
- Any manual corruption, unauthorized bit-flip, or tampering causes immediate failure (`StoreError::IntegrityViolation`).

### 2.2 Invariant `INV-STR-002`: User-Owned Consent Authority
- Services cannot grant themselves capabilities or modify consent policies ambiently.
- Only operations accompanied by explicit user capability authorization (`CAP_STORE_ADMIN`) can write or revoke consent records.
- Service read access for policy checks is mediated through scoped, read-only capability tokens (`CAP_STORE_READ`).

---

## 3. Data Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentState {
    Granted,
    Denied,
    PromptAlways,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsentRecord {
    pub service_id: u32,
    pub permission_id: u32,
    pub state: ConsentState,
    pub timestamp_ms: u64,
    pub checksum: u32,
}
```
