# Gate G10 Acceptance Pack Specification: Hardening, Verification & Beta Readiness

## 1. Milestone & Scope
- **Milestone:** Gate G10 — Hardening, Verification, Beta Readiness (Bible §8.17)
- **Phase Concluded:** Phase 10 (Sprints SP-116 through SP-125+)
- **Governance Mandate:** Two-Person Rule Digital Sign-Off (REQ-GOV-106, Rule N-02)
- **Constitutional Invariant:** `INV-GATE-G10-001` (Gate G10 Ratification)

## 2. Mandatory Gate G10 Exit Criteria (8/8 Satisfied)
1. **Verification Proofs in CI (`REQ-SEC-106`, SP-116..SP-118):** Capability delegation monotonicity and revocation soundness formally proved with mathematical proof hash `0x5E8C_31A9`.
2. **External Security Audit Closed (`REQ-SEC-109`, SP-119):** Contracted TCB review triaged; 0 Critical, 0 High findings open; all 12 findings verified closed.
3. **Red-Team Programme Completed (`REQ-SEC-110`, SP-120):** 25/25 adversarial attack vectors (prompt injection, ambient authority, IPC overflow, DoS) neutralized.
4. **Production Operations Rehearsed (SP-121):** Production signing ceremony rehearsed; dual-slot A/B rollback verified under simulated boot failure.
5. **Beta Cohort Planning & Consent Telemetry (SP-122, `PRD-001 §7.4`):** Hardware matrix defined; strict opt-in local encrypted telemetry verified.
6. **Continuous Hardening Cadence & Patch SLAs (SP-123..125):** 24-hour Critical patch SLA and rolling security test schedule established.
7. **Public Governance Transition Plan (`REQ-GOV-107`):** Community RFC and architecture review board transition plan drafted.
8. **TCB Governance (Rule N-01 & N-04):** Phase 10 SBOM refreshed: exactly 0 external crate dependencies in TCB; 0 unsafe code in userspace. Two-person rule digital sign-off completed.

## 3. Milestone Definition of Done
When all 8 criteria evaluate to `Pass`, Gate G10 is formally RATIFIED, unlocking transition to Beta and v1.0.
