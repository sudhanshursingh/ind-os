# Ind OS Architecture Specification: Explanation Engine v1 (SP-46)

## Document Information
- **Standard**: Ind OS Development Bible v1.0.0.0 §6.3, §8.12
- **Requirement**: REQ-AI-104 (Explanation Engine v1: Action Provenance: action, rule fired, consent id, model version, input provenance)
- **Invariants**:
  - `INV-EXPL-001`: Complete Action Provenance Tuples — Every executed or denied capability proposal must produce an immutable provenance record containing: action id, category, rule id fired, consent card id, model hash / version, and input provenance source.
  - `INV-EXPL-002`: Cryptographically Bound "Why" Query Interface — The provenance engine provides a verifiable query API (`explain(action_id)`) allowing the user or auditor to trace exact causality back to model and consent inputs without ambiguous AI hallucinations.
- **Rule Set**: Rule N-01 (100% pure Rust), Rule N-04 (Zero unsafe).

---

## 1. Scope & Action Provenance Model
In Ind OS, AI actions cannot happen opaquely. The Explanation Engine (`explain`) records exact causal records for every proposal processed by the system.

A Provenance Record consists of:
1. `action_id`: Unique proposal/execution identifier.
2. `category`: Action category (File, Network, Settings, etc.).
3. `rule_fired_id`: ID of the policy rule or standing policy that evaluated the action.
4. `consent_card_id`: Optional ID of the interactive user consent card (if escalated to user).
5. `model_version`: Exact signing hash / version ID of the model producing the intent.
6. `input_provenance`: Description of the triggering source (e.g. "User Voice Prompt", "File Watcher", "Calendar Event").

---

## 2. Explanation Record Schema

```rust
pub struct ProvenanceRecord {
    pub record_id: u32,
    pub action_id: u32,
    pub category: ActionCategory,
    pub rule_fired_id: Option<u32>,
    pub consent_card_id: Option<u32>,
    pub model_version_hash: u32,
    pub input_source: &'static str,
    pub timestamp_ms: u64,
}

pub struct ExplanationReport {
    pub action_id: u32,
    pub summary: &'static str,
    pub causality_path: &'static str,
    pub verified: bool,
}
```

---

## 3. Kernel Verification Stages (177–181)
- `177_EXPLAIN_V1_INIT`: Explanation engine v1 initialized.
- `178_EXPLAIN_V1_PROVENANCE_TUPLE`: 5-tuple action provenance generation verified.
- `179_EXPLAIN_V1_RECONSTRUCT_CAUSALITY`: "Why" query causal reconstruction verified.
- `180_EXPLAIN_V1_IMMUTABLE_LOG_BINDING`: Provenance records anchored to structured audit log bus.
- `181_SP46_EXPLAIN_READY`: Sprint SP-46 Milestone reached.
