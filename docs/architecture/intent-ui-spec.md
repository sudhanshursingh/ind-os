# Ind OS Specification: Intent Surface UI & Indic IME Engine (SP-62)

- **Sprint Identifier:** SP-62
- **Phase:** Phase 8 (UX Shell + App Framework)
- **Target Gate:** Gate G8 (USER-READY OS V0.1)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.14, Table 19.2, REQ-AI-104, REQ-AI-105, REQ-AI-108, Rule N-01, Rule N-04
- **Sign-off:** Sudhanshu Raj Singh <sudhanshurajsingh@users.noreply.github.com>

---

## 1. Architectural Objectives

1. **Intent Surface Lifecycle (REQ-AI-104, REQ-AI-105):**
   - Implements strict 5-stage intent progression:
     $$\text{Input} \longrightarrow \text{Proposal Card} \longrightarrow \text{User Consent} \longrightarrow \text{Sandbox Execution} \longrightarrow \text{Explanation Receipt}$$
   - Any attempt to execute without explicit user consent is unconditionally rejected (`INV-INTENT-UI-001`).
2. **Explanation Receipt Auditability (REQ-AI-105):**
   - Upon execution completion, an explanation receipt is generated and immutably logged into the Why Engine (`INV-INTENT-UI-002`).
3. **Indic IME Engine (REQ-AI-108):**
   - Core phonetic transliteration and font layout architecture for Indian languages (Devanagari, Tamil, Telugu, Kannada, Bengali).
   - Zero-heap allocation; glyph mapping performed within bounded stack buffers (`INV-INTENT-UI-003`).

---

## 2. Invariants & Security Guarantees

| Invariant ID | Requirement | Architectural Invariant |
| :--- | :--- | :--- |
| `INV-INTENT-UI-001` | REQ-AI-104 | State machine enforces mandatory consent prior to capability dispatch. |
| `INV-INTENT-UI-002` | REQ-AI-105 | Execution produces auditable explanation receipt with deterministic token. |
| `INV-INTENT-UI-003` | REQ-AI-108 | Indic IME transliteration performs bounded glyph substitution with zero heap allocations. |

---

## 3. Data Structures & API Contract

```rust
pub enum IntentState {
    Drafting,
    ProposalGenerated,
    ConsentGranted,
    ConsentDenied,
    Executed,
    ReceiptGenerated,
}

pub struct ProposalCard {
    pub intent_id: u32,
    pub summary: &'static str,
    pub required_cap: u64,
    pub risk_level: u8,
}

pub struct ExplanationReceipt {
    pub intent_id: u32,
    pub action_summary: &'static str,
    pub audit_hash: u32,
}

pub struct IndicImeEngine {
    // Zero-heap transliteration buffers
}
```
