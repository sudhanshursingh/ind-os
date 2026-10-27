# Ind OS UI Polish & Adversarial Hardening Specification (SP-69)

## 1. Architectural Mandate
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.14, Table 19.2 (REQ-SEC-109, REQ-UX-306).
- **Target:** UI stack penetration testing, anti-tapjacking, anti-overlay, spoofed consent rejection, and universal Why-Engine explanation coverage.

---

## 2. Architectural Invariants

| ID | Description | Enforcement Mechanism |
| :--- | :--- | :--- |
| **INV-POLISH-001** | Anti-Tapjacking & Anti-Overlay Gating | Transparent or partially occluding overlays are forbidden over system dialogs; input events are discarded if z-order collision occurs |
| **INV-POLISH-002** | Spoofed Consent Rejection | Consent records without valid cryptographic token from TCB security broker are rejected with `ConsentSignatureInvalid` |
| **INV-POLISH-003** | Universal Explanation Coverage | 100% of capability decisions have registered causal audit records queryable by the Why-Engine |

---

## 3. Data Structures & API Contracts

```rust
pub enum PolishSecurityError {
    TapjackingAttemptBlocked,
    ConsentSignatureInvalid,
    UntracedActionDenied,
    TableFull,
}

pub struct AdversarialUiHarness {
    blocked_tapjacks: u32,
    rejected_faked_consents: u32,
    explanation_checks: u32,
}
```
