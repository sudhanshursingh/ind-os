# Ind OS Architectural Specification: Unified Log Bus & Diagnostics (`REQ-USE-204`)
**Document Identifier:** `INDOS-SPEC-LOG-01`  
**Sprint:** SP-36  
**Phase:** Phase 5 (Userspace & the Developer OS)  
**Status:** **APPROVED / ACTIVE**  
**Requirements Traceability:** `REQ-USE-204`  
**Invariants Enforced:** `INV-LOG-001` (Privacy Redaction Guarantee), `INV-LOG-002` (Explainability & Causal Root Cause Reconstruction)  
**Author:** Sudhanshu Raj Singh (`sudhanshurajsingh@users.noreply.github.com`)

---

## 1. Scope & Objective

This specification governs the design of the **Unified Log Bus (`logd` / `indus-logd`)**, privacy-preserving redaction filters, and the **`why` diagnostic explanation engine** (`REQ-USE-204`).

Under the Ind OS constitutional architecture:
1. All userspace daemons and kernel subsystems submit telemetry and log events to a unified, structured bus.
2. Sensitive cryptographic keys, authentication tokens, and user credentials must be deterministically masked before storage (`INV-LOG-001`).
3. The `why` command enables developers and users to query the causal chain behind any failure, crash, or permission denial, serving as the architectural precursor to Phase 6's explanation engine (`INV-LOG-002`).

---

## 2. Invariants

### 2.1 Invariant `INV-LOG-001`: Privacy Redaction Guarantee
- Every log message ingested into the bus passes through pure Rust pattern redaction filters.
- Patterns matching capability tokens (`token:0x...`), cryptographic secrets (`key:...`), or sensitive passwords are sanitized into `[REDACTED:TOKEN]` or `[REDACTED:KEY]`.
- Raw unredacted secrets must never enter persistent storage or terminal displays.

### 2.2 Invariant `INV-LOG-002`: Explainability & Causal Root Cause Reconstruction
- When a service terminates abnormally, faults, or experiences capability denial, a structured diagnostic record (`DiagnosticCausalChain`) is linked to the event ID.
- The `why` command queries this graph and reconstructs:
  - Triggering event (e.g. `OutOfMemory`, `AccessDenied`, `SignalCrash`)
  - Subsystem context (e.g. `devmgr` attempted MMIO out-of-bounds `0x0a000200 + 1024B`)
  - Action taken by supervisor (e.g. `Supervisor scheduled restart in 40ms with backoff`)
  - Plain-language human explanation.

---

## 3. Log Bus Schema

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    SecurityAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogEntry<'a> {
    pub timestamp_ms: u64,
    pub level: LogLevel,
    pub source: &'a str,
    pub event_id: u32,
    pub message: &'a str,
}
```
