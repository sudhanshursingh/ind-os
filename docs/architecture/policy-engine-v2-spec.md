# Ind OS Architecture Specification: Policy Engine v2 & Standing User Policies

**Document ID:** `INDOS-SPEC-POL-02`  
**Sprint:** SP-43 (Policy Engine v2: Standing User Policies as First-Class Objects, Conflict Resolution, Dry-Run Simulation)  
**Phase:** Phase 6 (AIS Subsystem + ENF Policy Engine + Consent Broker)  
**Constitutional Authority:** Ind OS Development Bible v1.0.0.0 §6.3, §8.12 (Sprint SP-43)  
**Status:** **RATIFIED**  
**Classification:** Core System Architecture Specification  

---

## 1. Scope & Objective

Sprint SP-43 advances the Execution Normalization Framework (ENF) into **Policy Engine v2**, transforming policy rules into first-class, user-manageable standing objects:
1. **Standing User Policies as First-Class Objects (`INV-POL-001`):** User policies are inspectable, suspendable, and revocable at any time. They are represented with explicit lifecycles (`Active`, `Suspended`, `Revoked`) and unique IDs.
2. **Deterministic Conflict Resolution (`INV-POL-002`):** Sophisticated conflict resolution semantics:
   - Specificity ordering (exact subject/resource match takes precedence over wildcard matches).
   - Priority tie-breaking: explicit `Deny` rules strictly override `Allow` rules of equal or lesser priority.
   - Default-deny security model (`INV-ENF-001`) preserved at all times.
3. **Dry-Run Simulation ("What Would Happen If...") (`INV-POL-003`):** Zero-side-effect evaluation interface allowing AIS, Consent Broker, or users to simulate action proposals against the active policy set. Returns the prospective decision, which rule fired, and whether user consent would be required, without updating state or logs.

---

## 2. Invariants & Architectural Rules

### 2.1 Invariant `INV-POL-001`: Standing Policy Immutability & Revocability
Standing user policies cannot be silently mutated or bypassed by untrusted applications or AIS:
- Every standing policy has an immutable creation record and an explicit lifecycle state.
- Revoking a policy immediately transitions its state to `Revoked`, causing subsequent evaluations to skip the rule atomically.
- Users can inspect 100% of standing policies through trusted system interfaces.

### 2.2 Invariant `INV-POL-002`: Deterministic Conflict Resolution
When multiple standing policies match the same subject, resource, and rights request:
- Explicit `Deny` rules strictly override `Allow` or `RequireConsent` rules of equal or lesser priority.
- If an exact match conflicts with a wildcard match, the exact match takes precedence.
- If no matching rule permits the action, the default-deny policy rejects the request with `Denied`.

### 2.3 Invariant `INV-POL-003`: Side-Effect-Free Dry-Run Simulation
The dry-run simulation mode (`simulate_dry_run`):
- MUST evaluate identical precedence and rule-matching logic as `evaluate`.
- MUST NOT allocate persistent capability tokens, mutate consent state, or trigger audit log writes.
- Returns comprehensive diagnostic output (`would_allow`, `firing_rule_id`, `needs_consent`, `conflict_count`).

---

## 3. Data Structures & Subsystem Architecture

### 3.1 Policy Lifecycle State
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyLifecycleState {
    Active,
    Suspended,
    Revoked,
}
```

### 3.2 Standing Policy Object
```rust
pub struct StandingPolicy {
    pub policy_id: u32,
    pub name: &'static str,
    pub rule: PolicyRule,
    pub state: PolicyLifecycleState,
}
```

### 3.3 Dry-Run Simulation Report
```rust
#[derive(Debug, Clone, Copy)]
pub struct DryRunReport {
    pub prospective_decision: PolicyDecision,
    pub firing_rule_id: Option<u32>,
    pub needs_consent: bool,
    pub conflicts_detected: u32,
}
```

---

## 4. Verification & Acceptance Protocol

The implementation of SP-43 is validated through:
1. Subsystem implementation within [`kernel/src/policy/engine.rs`](file:///Users/dhan/IndOS/kernel/src/policy/engine.rs) and [`kernel/src/userspace/policy_v2.rs`](file:///Users/dhan/IndOS/kernel/src/userspace/policy_v2.rs).
2. Userspace standing policy bindings in [`userspace/ais/src/policy.rs`](file:///Users/dhan/IndOS/userspace/ais/src/policy.rs).
3. Kernel boot verification stages 162–166 in [`kernel/src/main.rs`](file:///Users/dhan/IndOS/kernel/src/main.rs).
4. QEMU execution log [`docs/evidence/golden-boot-sp43.log`](file:///Users/dhan/IndOS/docs/evidence/golden-boot-sp43.log).
5. Validation report [`docs/evidence/SP-43-validation.md`](file:///Users/dhan/IndOS/docs/evidence/SP-43-validation.md).
6. Automated CI verifier [`ci/verify_sp43.py`](file:///Users/dhan/IndOS/ci/verify_sp43.py).
