# Ind OS Enforcement Layer (ENF) Policy Engine v1 Specification (SP-19)

**Document ID:** `INDOS-SPEC-ENF-01`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§6.3, §8.9, §10, §14), `docs/architecture/phase-3-security-spine-roadmap.md` (§3)  
**TCB Subsystems:** `kernel/src/policy/`, `kernel/src/capability/`, `kernel/src/task/`  
**Review Status:** RATIFIED (Phase 3 Security Spine Baseline)  

---

## 1. Architectural Scope & Objectives

The Enforcement Layer (`ENF`) Policy Engine is the central policy decision component of Ind OS. It governs access to all privileged kernel operations and capabilities.

In strict compliance with **Bible §6.3 (The AIS–ENF Separation)**:
1. **Separation of Policy and Mechanism:** Capabilities represent unforgeable access tokens (mechanisms); the Policy Engine represents standing, user-configurable, declarative rules (policy).
2. **Dual-Check Access Enforcement:** Possession of a capability token is necessary but not sufficient. Every privileged access check is cross-validated against the Policy Engine (`check_capability_access()`).
3. **AIS Untrusted Boundary (`REQ-AI-001`, `REQ-AI-002`):** The Adaptive Intelligence Subsystem runs with zero ambient authority. It can only propose actions. Standing policy or user consent determines whether actions execute.
4. **Declarative Rule Language:** Rules are written in a human-readable, auditable syntax that maps directly to kernel rule evaluation.
5. **Versioned Artifacts:** Policy rules carry explicit versioning and priority tags. Rulesets can be loaded, verified, and audited as versioned artifacts.

---

## 2. Machine Invariants

1. **`INV-ENF-001` (Default-Deny Security Model):**
   In the absence of an explicit, active `PolicyAction::Allow` rule matching the requesting subject, target resource, and requested rights, the Policy Engine returns `PolicyDecision::Denied`. Ambient or implicit authority is mathematically impossible:
   $$\forall (S, R, \text{rights}) : \left( \nexists r \in \text{Rules} : \text{matches}(r, S, R, \text{rights}) \land r.\text{action} = \text{Allow} \right) \implies \text{evaluate}(S, R, \text{rights}) = \text{Denied}$$

2. **`INV-ENF-002` (Deterministic Precedence & Versioned Rule Resolution):**
   Rules evaluate with fixed, deterministic precedence:
   - Explicit `Deny` rules with priority $\ge$ matching `Allow` or `RequireConsent` rules strictly override them.
   - Higher numeric priority takes precedence over lower numeric priority.
   - All rules declare explicit schema versions and rule IDs for tamper-evident provenance.

3. **`REQ-SEC-001` (Dual-Check Capability Supremacy):**
   A kernel capability invocation succeeds if and only if:
   $$\text{is\_valid}(C) \quad \land \quad \text{rights}(C) \supseteq \text{requested} \quad \land \quad \text{evaluate}(\text{Process}(P), C.\text{resource}, \text{requested}) = \text{Allowed}$$

---

## 3. Declarative Grammar & Engine Architecture

### 3.1 Declarative Rule Grammar

The Policy Engine parses human-readable declaration statements:
```text
<ACTION> sub:<SUBJECT> res:<RESOURCE> rights:<RIGHTS> prio:<PRIO> id:<ID> v:<VERSION>
```
- `<ACTION>`: `ALLOW`, `DENY`, or `CONSENT`
- `<SUBJECT>`: `any`, `pid:<u32>`, or `role:<u32>` (1=Core, 2=Driver, 3=AIS, 4=App)
- `<RESOURCE>`: `any`, `ep:<u32>`, `obj:<idx>`, or `sys:<num>`
- `<RIGHTS>`: Pipe-delimited combination of `READ`, `WRITE`, `GRANT`, `CALL`, or `ALL`
- `<PRIO>`: Numeric priority (0–65535, default 10)
- `<ID>`: Unique rule identifier
- `<VERSION>`: Version number of ruleset

Example Declarations:
```text
ALLOW sub:pid:1 res:ep:101 rights:READ|WRITE prio:10 id:1 v:1
ALLOW sub:pid:2 res:ep:101 rights:READ prio:10 id:2 v:1
DENY sub:pid:2 res:ep:101 rights:WRITE prio:20 id:3 v:1
CONSENT sub:role:3 res:any rights:ALL prio:30 id:4 v:1
```

### 3.2 Evaluation Flow

```text
                  Incoming Access Request (Subject, Resource, Rights)
                                          │
                                          ▼
                         Scan Active PolicyRule Array
                                          │
                  ┌───────────────────────┴───────────────────────┐
                  ▼                                               ▼
         Matching Deny Rules?                           Matching Allow / Consent?
                  │                                               │
                  └───────────────────────┬───────────────────────┘
                                          ▼
                      Deny.priority >= Allow/Consent.priority?
                                    ┌─────┴─────┐
                               YES  │           │ NO
                                    ▼           ▼
                           DENIED (EPERM)   Consent.prio > Allow.prio?
                                                      ┌─────┴─────┐
                                                 YES  │           │ NO
                                                      ▼           ▼
                                                 NEEDS_CONSENT  ALLOWED
                                                      │           │
                                       No matching rules? ────────┘
                                                      │
                                                      ▼
                                            DEFAULT-DENY (EPERM)
```

---

## 4. TCB Safety & Resource Bounds

- **Pre-Allocated Static Bounds:** Maximum 64 policy rules (`MAX_POLICY_RULES = 64`).
- **Zero Allocations in Evaluation Path:** Zero dynamic heap allocation (`alloc`) in `evaluate()` or `check_capability_access()`.
- **Zero Unsafe Blocks:** 100% pure safe Rust across `kernel/src/policy/`.
