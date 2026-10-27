# Ind OS Specification: Proactive Engine UI & Aggressiveness Dial (SP-65)

- **Sprint Identifier:** SP-65
- **Phase:** Phase 8 (UX Shell + App Framework)
- **Target Gate:** Gate G8 (USER-READY OS V0.1)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.14, Table 19.2, REQ-AI-106, Rule N-01, Rule N-04
- **Sign-off:** Sudhanshu Raj Singh <sudhanshurajsingh@users.noreply.github.com>

---

## 1. Architectural Objectives

1. **User-Controlled Aggressiveness Dial (REQ-AI-106):**
   - Four discrete aggressiveness modes:
     - `Off` (Threshold $\ge 100$, no proactive interruptions)
     - `Conservative` (Threshold $\ge 90$)
     - `Balanced` (Threshold $\ge 70$)
     - `Proactive` (Threshold $\ge 50$)
   - Suggestions below the active dial threshold are filtered prior to presentation (`INV-PROACTIVE-001`).
2. **Deterministic Suggestion Ranking:**
   - Suggestions are ranked in descending order of confidence score (`INV-PROACTIVE-002`).
3. **Auditable Feedback Controls:**
   - Every suggestion supports Dismiss, Snooze, and Explain actions. The Explain action produces a deterministic Why-Engine link (`INV-PROACTIVE-003`).

---

## 2. Invariants & Security Guarantees

| Invariant ID | Requirement | Architectural Invariant |
| :--- | :--- | :--- |
| `INV-PROACTIVE-001` | REQ-AI-106 | Aggressiveness dial filters proactive suggestions by confidence threshold. |
| `INV-PROACTIVE-002` | REQ-AI-106 | Suggestion queue maintains deterministic priority ranking by confidence score. |
| `INV-PROACTIVE-003` | REQ-AI-106 | Dismiss, Snooze, and Explain feedback actions deterministically update suggestion state. |

---

## 3. Data Structures & API Contract

```rust
pub enum AggressivenessDial {
    Off,          // 100% threshold
    Conservative, // 90% threshold
    Balanced,     // 70% threshold
    Proactive,    // 50% threshold
}

pub struct ProactiveSuggestion {
    pub id: u32,
    pub title: &'static str,
    pub body: &'static str,
    pub confidence_score: u8, // 0..=100
    pub is_dismissed: bool,
    pub is_snoozed: bool,
}

pub struct ProactiveUiEngine {
    // Ranking queue & dial state
}
```
