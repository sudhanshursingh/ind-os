# Ind OS Contributing Guidelines & Engineering Discipline

**Governing Authority:** `Ind_OS_Development_Bible_v1.0.0.0.md`  
**Constitution:** Section 5 (Rules N-01 through N-13)

---

## 1. Commit Message Convention (Rule N-10 & CI Enforcement)

Every commit message MUST reference a stable Requirement ID (`REQ-*`), Decision ID (`DEC-*`), Default ID (`DEF-*`), or Pending Question ID (`PEN-*`).

### Format:
```text
<type>(<scope>): <summary> [REQ-XXX-NNN | DEC-NNN | DEF-NNN | PEN-NNN]

<detailed rationale explaining what was built and why>
- Ref: REQ-XXX-NNN
- Status: [REQUIREMENT | IMPLEMENTED | TESTED | EVIDENCE: <ref>]
```

### Valid Types:
- `feat`: Implementation of a requirement
- `fix`: Bug fix with reproduction test case
- `docs`: Specification, architecture, or decision record update
- `test`: Evidence pack or verification harness update
- `ci`: CI pipeline and test runner changes
- `chore`: Toolchain, formatting, or tree scaffolding

---

## 2. The Non-Negotiable Rules (Quick Reference)

1. **Rule N-01 (From scratch):** Copying code from Linux, AOSP, or BSD into `IndK` or `IndUS` is strictly prohibited.
2. **Rule N-02 (Evidence):** Every technical claim must carry an evidence label (`[REQUIREMENT]`, `[IMPLEMENTED]`, `[TESTED]`, `[EVIDENCE: <ref>]`, `[HYPOTHESIS]`, `[FAILED: <ref>]`).
3. **Rule N-03 (No assumptions):** If an architectural question is open, escalate via `[PEN-NNN]`. Do not guess.
4. **Rule N-04 (Security enforced):** AIS is advisory only. Privileged actions require kernel capability tokens.
5. **Rule N-09 (SBOM from day zero):** Every dependency change must update `docs/engineering/licensing-and-sbom.md` in the same commit.
6. **Rule N-10 (Tests with code):** No code is merged without passing tests and CI green.
