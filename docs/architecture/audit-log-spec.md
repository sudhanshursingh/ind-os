# Ind OS Architecture Specification: Cryptographic Audit Subsystem v1

**Document Identifier:** `INDOS-SPEC-AUD-01`  
**Classification:** Core Kernel Architecture / Security Spine Specification  
**Status:** Approved & Implemented (Sprint SP-20)  
**Governing Standard:** Ind OS Development Bible v1.0.0.0 (§8.9, §10, §14)  
**Invariants Enforced:** `INV-AUD-001`  

---

## 1. Executive Summary

The Ind OS Cryptographic Audit Subsystem (`Audit v1`) establishes an immutable, append-only, hash-chained security record within the IndK microkernel. Designed under the strict zero-ambient-authority mandate of Phase 3, the audit engine guarantees that every security-critical lifecycle event—including capability grants, revocations, delegations, and policy evaluation decisions—is recorded with a cryptographic tamper-evident proof.

By implementing an in-kernel, pure safe Rust zero-dependency SHA-256 engine compliant with FIPS 180-4, Ind OS satisfies Rule N-01 (zero external kernel crates) while providing mathematical guarantees against record tampering, truncation, and historical revision.

---

## 2. Normative Invariants

### `INV-AUD-001`: Cryptographic Append-Only Audit Integrity & Hash Chaining
> **Invariant Statement:** Every audit entry $E_i$ recorded in the kernel audit log must be cryptographically bound to the preceding entry $E_{i-1}$ such that $E_i.\text{prev\_hash} = E_{i-1}.\text{entry\_hash}$, where $E_0.\text{prev\_hash} = \mathbf{0}^{32}$. The digest $E_i.\text{entry\_hash}$ must equal the SHA-256 evaluation over the canonical byte serialization of $E_i$. Any modification, bit-flip, deletion, or reordering of historical records must deterministically break chain integrity and immediately localize the corrupted record index during validation.

---

## 3. Cryptographic Hash Engine (SHA-256)

Per Bible Rule N-01, the microkernel prohibits external crates or dynamic libraries. The audit subsystem incorporates a pure safe Rust implementation of the NIST FIPS 180-4 Secure Hash Standard (SHA-256) located at [kernel/src/audit/sha256.rs](file:///Users/dhan/IndOS/kernel/src/audit/sha256.rs):

- **State Representation:** 8 32-bit words ($H_0 \dots H_7$) initialized to standard fractional square roots of the first 8 primes.
- **Round Constants:** 64 32-bit constants ($K_0 \dots K_{63}$) representing the fractional cube roots of the first 64 primes.
- **Block Processing:** 512-bit (64-byte) chunk consumption with in-place big-endian word expansion.
- **Message Padding:** Strict FIPS 180-4 standard padding (`0x80` byte, zero fill, and 64-bit big-endian message length in bits).
- **Safety Guarantee:** Pure safe Rust containing zero unsafe blocks (`0`), eliminating memory safety vulnerabilities during cryptographic processing.

---

## 4. Audit Entry Schema & Canonical Serialization

Each audit record is defined as a fixed-size, statically bounded structure in [kernel/src/audit/entry.rs](file:///Users/dhan/IndOS/kernel/src/audit/entry.rs):

```rust
#[repr(u32)]
pub enum AuditEventType {
    Genesis = 0,
    CapabilityGrant = 1,
    CapabilityRevoke = 2,
    CapabilityDelegate = 3,
    PolicyAllow = 4,
    PolicyDeny = 5,
    PolicyConsentRequired = 6,
    PolicyUpdate = 7,
    ProcessTermination = 8,
}

pub struct AuditEntry {
    pub sequence: u64,
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub subject_pid: ProcessId,
    pub target_id: u32,
    pub rights: CapRights,
    pub prev_hash: [u8; 32],
    pub entry_hash: [u8; 32],
}
```

### Canonical Hash Pre-Image
The entry hash is computed across a contiguous canonical byte stream:
$$\text{PreImage}(E_i) = \text{seq} \parallel \text{time} \parallel \text{event\_type} \parallel \text{subject\_pid} \parallel \text{target\_id} \parallel \text{rights} \parallel \text{prev\_hash}$$

| Field | Type | Encoding | Byte Size |
| :--- | :--- | :--- | :--- |
| `sequence` | `u64` | Big-Endian | 8 bytes |
| `timestamp` | `u64` | Big-Endian | 8 bytes |
| `event_type` | `u32` | Big-Endian | 4 bytes |
| `subject_pid` | `u32` | Big-Endian | 4 bytes |
| `target_id` | `u32` | Big-Endian | 4 bytes |
| `rights` | `u32` | Big-Endian | 4 bytes |
| `prev_hash` | `[u8; 32]` | Raw Bytes | 32 bytes |
| **Total Pre-Image** | | | **64 bytes** |

---

## 5. Genesis Block Specification

The Genesis block anchors the cryptographic chain at kernel initialization (`Stage 49`):
- `sequence`: `0`
- `timestamp`: `0`
- `event_type`: `AuditEventType::Genesis` (`0`)
- `subject_pid`: `0`
- `target_id`: `0`
- `rights`: `CapRights::NONE` (`0`)
- `prev_hash`: `[0u8; 32]` (all zeros)
- `entry_hash`: $\text{SHA-256}(\text{PreImage}(E_0))$

---

## 6. Tamper Detection & Localization Protocol

Verification of the audit log is executed by `AuditLog::verify_integrity()`:

1. **Genesis Verification:**
   - Confirm $E_0.\text{prev\_hash} == \mathbf{0}^{32}$.
   - Confirm $\text{SHA-256}(\text{PreImage}(E_0)) == E_0.\text{entry\_hash}$.
2. **Chain Traversal ($i = 1 \dots N-1$):**
   - Check chain pointer: $E_i.\text{prev\_hash} == E_{i-1}.\text{entry\_hash}$. If mismatched, return `Err((i, "Hash chain link broken"))`.
   - Check payload integrity: $\text{SHA-256}(\text{PreImage}(E_i)) == E_i.\text{entry\_hash}$. If mismatched, return `Err((i, "Entry content hash mismatch"))`.
3. **Soundness Result:**
   - If an attacker flips even a single bit in an entry's payload or hash, either the payload hash or the downstream link in $E_{i+1}$ fails, localizing the exact record index with $O(1)$ memory overhead.

---

## 7. Bounded Static Memory Model

To prevent memory exhaustion attacks and enforce deterministic microkernel performance:
- The log is stored in fixed static memory: `[AuditEntry; MAX_AUDIT_ENTRIES]` where `MAX_AUDIT_ENTRIES = 128`.
- Total memory footprint: $128 \times 88\text{ bytes} \approx 11.2\text{ KB}$, well within L1 data cache limits.
- Zero heap allocation is performed during audit recording or verification.

---

## 8. Verification & Test Evidence

The subsystem is validated in QEMU via boot stages 49 through 52:
- `Stage 49 (49_AUDIT_LOG_INIT)`: Initializes log and validates Genesis block.
- `Stage 50 (50_AUDIT_EVENT_LOGGING)`: Appends 4 lifecycle events with strict monotonic sequence numbers.
- `Stage 51 (51_AUDIT_TAMPER_TEST)`: Executes deliberate 1-bit mutation test, confirming immediate detection and localization of corruption.
- `Stage 52 (52_AUDIT_V1_READY)`: Confirms `INV-AUD-001` verification and milestone completion.
