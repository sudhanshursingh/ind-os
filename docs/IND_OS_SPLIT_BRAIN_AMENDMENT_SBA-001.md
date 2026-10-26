# IND OS — SPLIT-BRAIN ARCHITECTURE AMENDMENT (SBA-001)
## Binding Agent Instruction — Effective Immediately

**Status:** LOCKED. Supersedes any conflicting instruction in prior sprint prompts.
**Applies to:** All sprints from Sprint 25 onward, and retroactively to any AI-adjacent code merged after this amendment.
**Author:** Lead Architect (Sudhanshu Raj Singh)
**Basis:** From-Scratch PRD v1.0, FS-D1 through FS-D12, plus the split-brain synthesis of AKTS (policy-plane switching), FlexServe/TZ-LLM (TEE inference), Agentic-seL4 (capability containment), and LUMOS (semantic state).

---

## 1. THE PRIME DIRECTIVE

Ind OS is a **split-brain architecture**. There are exactly two places AI-adjacent computation may live, and they have different jobs:

| Plane | Where | What runs there | What must NEVER run there |
| :--- | :--- | :--- | :--- |
| **Policy Plane** | Kernel space (Stage A: `sched_ext` BPF + Rust policy engine; Stage B: native microkernel object) | **Pre-verified policy selectors and resource controllers.** An AI agent may choose *which* verified policy to apply by writing an index. It may NOT execute model weights, tensor math, or token sampling here. | Any LLM, ASR, TTS, or semantic-graph code. Any code that reads model memory. Any code that parses untrusted text. |
| **Inference Plane** | TEE-protected userspace (Stage A: OP-TEE on Moto G60; Stage B: microkernel userspace in TEE domain) | **The full AI runtime:** LLM, speech pipeline, semantic graph builder, intent compiler. It is TEE-isolated. It holds only capability-mediated access to inputs and the Semantic Gateway. | Any direct kernel memory access. Any raw socket. Any ambient filesystem path. Any capability it did not receive from the Semantic Gateway. |

**The invariant that follows:** *No model weights, KV-cache pages, or token-sampling code may execute in kernel address space.* If a proposed change would put model execution in the kernel, STOP and file an `OPEN-QUESTIONS` entry. Do not implement it.

---

## 2. LOCKED MECHANISMS (Do not substitute)

### 2.1 Policy Plane (Kernel)
- **Stage A implementation:** `sched_ext` BPF program + Rust policy engine, following the AKTS pattern: the agent emits an integer index selecting from a pre-verified array of policies. Policy switch must complete in < 1 µs (measured, logged).
- **Stage B implementation:** native microkernel object, capability-typed, with the same index-selection semantics.
- **Constraint:** Every policy in the array must be independently verifiable and statically checked. The agent cannot emit arbitrary code.

### 2.2 Inference Plane (TEE)
- **Stage A implementation:** OP-TEE (or equivalent TEE) on Moto G60, with the AI runtime as a TA (Trusted Application).
- **Stage B implementation:** Arm CCA Realm (where hardware supports it), or TEE domain on the microkernel.
- **Constraint:** Model weights and KV cache must be unmapped from normal-world SMMU. The normal-world kernel must not be able to read them via `/proc/<pid>/mem`, DMA, or any SMMU bypass.
- **Performance target:** TTFT within 2× of a normal-world baseline (FlexServe achieved 10× speedup over naive TEE; TZ-LLM achieved 90.9% TTFT reduction — use these as the design envelope, not as a guarantee).

### 2.3 Semantic Gateway (Capability Boundary)
- **Location:** userspace, outside the TEE, outside the kernel. It is the *only* path from the Inference Plane to the world.
- **Policy engine:** hardcoded, Rust, formally specified in TLA+/Alloy. Any change to the policy engine is a `[BIBLE-AMENDMENT]`.
- **Intent ABI:** frozen as of this amendment. Schema version `1.0`. Any change requires human approval and a version bump.
- **Information-flow labels:** every input to the AI carries a taint label (`USER_VOICE`, `UNTRUSTED_SCREEN_TEXT`, `BANKING_SMS`, `SYSTEM_STATE`). The Gateway enforces that `UNTRUSTED_SCREEN_TEXT` can never invoke a capability with `RED` rights. This is a hard invariant, not a heuristic.

### 2.4 Capability Model
- **Every** AI-adjacent resource access goes through a capability.
- Capabilities are kernel-derived, unforgeable, attenuable, revocable.
- The AI runtime holds a **task-scoped** capability space. When the task ends, capabilities are revoked atomically.
- No ambient authority: the AI cannot enumerate the capability space, cannot "remember" a capability across tasks, cannot derive a capability with rights greater than its parent.

### 2.5 Semantic State Exporter
- Kernel-maintained (Stage B) or LSM-adjacent (Stage A), read-only capability.
- Produces a LUMOS-style structured graph (stable IDs, roles, bounds, action affordances).
- The AI runtime consumes this graph; it does **not** scrape the screen, does **not** use Android Accessibility, does **not** take screenshots.
- The graph is produced *below* the AI, so the AI cannot lie about what it sees.

---

## 3. CONSTRAINTS ON CURRENT SPRINT WORK (Sprint 25+)

This amendment is **additive**. It does not cancel the current sprint's tasks. It overlays constraints on any AI-adjacent work within those tasks.

**For every file the agent touches, it must check:**

1. **Does this code path execute model weights, tensor math, or token sampling in kernel space?** If yes → STOP. Move it to the Inference Plane or file `OPEN-QUESTIONS`.
2. **Does this code path give the AI runtime ambient authority (a path, a socket, a global handle) instead of a capability?** If yes → STOP. Replace with capability mediation or file `OPEN-QUESTIONS`.
3. **Does this code path allow untrusted text (screen content, SMS, notification) to reach a `RED`-capability action without passing through the Information-Flow Tracker?** If yes → STOP. This is a security incident under Bible Rule 7.
4. **Does this code path bypass the Intent ABI for an AI-originated action?** If yes → STOP. All AI actions go through the Gateway.
5. **Does this code path assume the AI runtime is trusted?** If yes → STOP. The AI runtime is untrusted by design. The Gateway is the only trust boundary.
6. **Does this code path change the Intent ABI schema, the capability model, or the policy engine?** If yes → it is a `[BIBLE-AMENDMENT]`. File it; do not merge it.

**Reporting requirement:** Every sprint report from Sprint 25 onward must include a section titled `## Split-Brain Compliance` with:
- A list of AI-adjacent files touched.
- For each, which of the 6 constraints above was checked.
- Any `OPEN-QUESTIONS` filed.
- The output of the policy-switch latency benchmark (if the Policy Plane was touched).
- The output of the TEE memory-isolation test (if the Inference Plane was touched).

---

## 4. NEW DELIVERABLES TO ADD TO THE ROADMAP

These are additive to the current roadmap. They do not replace existing sprint tasks; they are parallel workstreams that the agent must begin scaffolding.

| Deliverable | Phase | DoD |
| :--- | :--- | :--- |
| **Intent ABI v1.0 specification** | Immediate (Sprint 25–26) | Schema, semantics, test vectors, published in `docs/intent-abi/`. Frozen under `[BIBLE-AMENDMENT]`. |
| **Capability model formal spec (TLA+ or Alloy)** | Sprint 26–27 | Model-checked: derivation, attenuation, revocation, no-amplification. Proof file in `docs/formal/`. |
| **Policy Plane prototype (`sched_ext` + Rust)** | Sprint 26–28 | Sub-µs policy switch, measured; pre-verified policy array; benchmark log in `docs/agent-reports/`. |
| **TEE Inference Plane prototype (OP-TEE on hanoip)** | Sprint 27–30 | Model loads and infers in TA; normal-world `/proc/<pid>/mem` and DMA cannot read weights; TTFT within 2× of normal-world baseline. |
| **Semantic State Exporter prototype** | Sprint 27–29 | Produces LUMOS-style graph from running UI; stable IDs; action affordances; no screenshot path. |
| **Information-Flow Tracker** | Sprint 28–30 | Taint labels propagate; `UNTRUSTED_SCREEN_TEXT` → `RED` capability is blocked; 100% test coverage on the attack corpus. |
| **Policy engine formal spec** | Sprint 29–31 | TLA+/Alloy spec of the Semantic Gateway policy engine; model-checked against the Intent ABI. |
| **Measured boot + transparency log** | Sprint 30–33 | Boot measurements into TPM/TrustZone counter; attestation capability; public log entry format. |

---

## 5. ESCALATION PROTOCOL

If the agent encounters any of the following, it must **STOP** and file an `OPEN-QUESTIONS` entry. It must not guess, not "temporarily" bypass a constraint, not "fix it later."

- A current sprint task requires AI computation in kernel space.
- A current sprint task requires ambient authority for the AI runtime.
- A current sprint task requires changing the Intent ABI, the capability model, or the policy engine.
- The `sched_ext` policy switch exceeds 1 µs.
- The TEE inference prototype cannot keep model weights out of normal-world memory.
- A test demonstrates that `UNTRUSTED_SCREEN_TEXT` can reach a `RED` capability.
- Any change would weaken the measured-boot chain.

**Every escalation is logged in `docs/agent-reports/escalations.md` with:** timestamp, sprint, file, constraint violated, proposed resolution, and whether human sign-off is required.

---

## 6. WHAT THIS AMENDMENT DOES NOT CHANGE

- The 5 Pillars remain binding.
- The Green/Amber/Red matrix remains binding (now enforced as capability attenuation, not as a Java verifier).
- The 6 GB RAM budget on Moto G60 remains binding (now interpreted as: model weights live in TEE-protected memory, not in normal-world RSS).
- The voice cascade (wake-word → speaker verify → ASR → SLM) remains binding.
- The GBNF/constrained-decoding mechanism remains an implementation detail of the Inference Plane.
- The Antigravity Operating Protocol (§6 of the Bible) remains binding.
- The commit discipline, model-asset handling, and security zero-tolerance rules remain binding.

---

## 7. AMENDMENTS LOG ENTRY (append to §11 of the Bible)

| ID | Version | Date | Decision | Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **A-002** | v1.0.1 → v1.1.0 | 2026-10-26 | **Split-Brain Architecture adopted.** Kernel = Policy Plane (verified selectors only). TEE = Inference Plane (full AI runtime). Capability microkernel = enforcement boundary. Intent ABI frozen at v1.0. Information-flow labels mandatory. No model weights in kernel space, ever. | Synthesises AKTS (sub-µs policy switching), FlexServe/TZ-LLM (TEE inference viability), Agentic-seL4 (capability containment), and LUMOS (semantic state) into a single enforceable architecture. Resolves the "AI in kernel vs. AI outside kernel" debate: the answer is both, in different planes, with a verified boundary. |

---

*End of amendment. Antigravity may proceed under SBA-001. Any conflict with prior instructions resolves in favour of this amendment.*
