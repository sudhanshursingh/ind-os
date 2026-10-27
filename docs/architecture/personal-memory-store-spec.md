# Ind OS Architecture Specification: Personal Memory Store (SP-45)

## Document Information
- **Standard**: Ind OS Development Bible v1.0.0.0 §6.3, §8.12
- **Requirement**: REQ-AI-103 (Personal Memory Store: Encrypted, Scoped, User-Editable, ENF-Governed Retention)
- **Invariants**:
  - `INV-MEM-001`: Scoped, Encrypted & User-Editable Personal Memory — Memory items are strictly partitioned by domain scopes (Ephemeral, Task, LongTerm), user-inspectable, user-editable, and stored encrypted with isolated credentials.
  - `INV-MEM-002`: ENF-Enforced Retention & Tombstoning — Data retention and expiration are strictly executed by the system enforcement boundary (ENF), not dependent on model prompt compliance. Expired records are cryptographically wiped and marked tombstoned.
- **Rule Set**: Rule N-01 (100% pure Rust), Rule N-04 (Zero unsafe).

---

## 1. System Architecture & Memory Scopes

The Personal Memory Store allows local AI models to retrieve relevant context without creating an ambient, permanent surveillance log.

```
       +---------------------------------------------+
       |             Userspace / AIS                 |
       |  +---------------------------------------+  |
       |  | PersonalMemoryStore                   |  |
       |  |  - Scopes: Ephemeral, Task, LongTerm  |  |
       |  |  - Inspect & Edit Interfaces          |  |
       |  +---------------------------------------+  |
       +----------------------+----------------------+
                              | IPC
       +----------------------v----------------------+
       |         ENF / Kernel Enforcement            |
       |  +---------------------------------------+  |
       |  | RetentionEngine                       |  |
       |  |  - Hard TTL Clock Sweeps              |  |
       |  |  - Secure Cryptographic Zeroization   |  |
       |  |  - Tombstone & Audit Trail            |  |
       |  +---------------------------------------+  |
       +---------------------------------------------+
```

---

## 2. Memory Item Schema

```rust
pub enum MemoryScope {
    EphemeralSession, // Auto-purged upon session end or max 1 hour
    TaskScoped,       // Retained for current project workflow
    LongTermPreference, // Explicit user preferences (e.g. dark mode, editor style)
}

pub enum MemoryStatus {
    Active,
    Edited,
    Tombstoned,
}

pub struct MemoryItem {
    pub item_id: u32,
    pub scope: MemoryScope,
    pub status: MemoryStatus,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub key: &'static str,
    pub value_bytes: [u8; 64],
    pub value_len: usize,
}
```

---

## 3. Kernel Verification Stages (172–176)
- `172_MEMORY_STORE_INIT`: Memory store subsystem initialized.
- `173_MEMORY_STORE_SCOPED_ENCRYPTION`: Scoped memory partitions verified.
- `174_MEMORY_STORE_USER_EDITABLE`: User inspection, editing, and tombstoning verified.
- `175_MEMORY_STORE_ENF_RETENTION`: ENF retention sweep zeroization verified.
- `176_SP45_MEMORY_READY`: Sprint SP-45 Milestone reached.
