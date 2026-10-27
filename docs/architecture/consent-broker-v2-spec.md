# Ind OS Architecture Specification: Consent Broker v2 & Plain-Language UX (SP-44)

## Document Information
- **Standard**: Ind OS Development Bible v1.0.0.0 §6.3, §8.12
- **Requirement**: REQ-UX-301 (Plain-Language Consent UX), REQ-AI-102 (Typed Proposal Presentation)
- **Invariants**:
  - `INV-CONSENT-001`: Plain-Language Narrative Generation — Every capability request from AIS or services must be translated into human-readable, non-technical plain language before presentation.
  - `INV-CONSENT-002`: Immutable Audit Binding — Every user decision (Grant, Deny, Temporary, Permanent) must be permanently logged to the kernel audit trail prior to capability token dispensation or rejection.
- **Rule Set**: Rule N-01 (100% pure Rust), Rule N-04 (Zero unsafe).

---

## 1. Scope & Architectural Goals
In Ind OS, the AI Subsystem (AIS) cannot acquire ambient authority. When an intent cannot be satisfied under existing standing policies (evaluated in SP-43), the proposal is escalated to the Consent Broker v2.

The Consent Broker:
1. Receives typed proposals (`TypedProposal`) from AIS.
2. Synthesizes a structured `ConsentCard` featuring a clear, unambiguous **Plain-Language Narrative** explaining:
   - What action is proposed.
   - Which resource is accessed.
   - Why the action is being performed.
   - The security impact (risk level).
3. Holds the execution in pending state awaiting user interactive decision (`GrantOnce`, `GrantAlways`, `DenyOnce`, `DenyAlways`).
4. Logs the decision immutably to `logd` / kernel audit log with cryptographic hash binding.
5. Injects the resulting capability token or returns an access denial.

---

## 2. Consent Card Data Structure

```rust
pub enum ConsentRiskTier {
    Low,       // Read-only public metadata
    Medium,    // Personal data read, network access
    High,      // System settings, persistent writes
    Critical,  // Hardware changes, security policy modifications
}

pub enum UserConsentDecision {
    Pending,
    GrantOnce,
    GrantAlways,
    DenyOnce,
    DenyAlways,
}

pub struct ConsentCard {
    pub card_id: u32,
    pub proposal_id: u32,
    pub risk_tier: ConsentRiskTier,
    pub title: &'static str,
    pub narrative: &'static str,
    pub resource_summary: &'static str,
    pub decision: UserConsentDecision,
}
```

---

## 3. Kernel Verification Stages (167–171)
- `167_CONSENT_V2_INIT`: Consent broker v2 initialized.
- `168_CONSENT_V2_PLAIN_NARRATIVE`: Plain-language narrative generation validated.
- `169_CONSENT_V2_PROPOSAL_BINDING`: AIS proposal successfully converted into interactive consent card.
- `170_CONSENT_V2_AUDIT_TRAIL`: User decision recorded into immutable log bus before actiondispatch.
- `171_SP44_CONSENT_READY`: Sprint SP-44 Milestone reached.
