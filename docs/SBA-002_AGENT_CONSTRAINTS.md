# SBA-002 — Antigravity Agent Constraints (Binding)
## Reconciled to IND_OS_Unified_Master_PRD_v2.0_FINAL

**Status:** LOCKED. Supersedes `SBA-001` (retired — referenced pre-v2.0 documents).
**Governing documents, in order of precedence:**
1. `IND_OS_Unified_Master_PRD_v2.0_FINAL.md` — the single source of truth
2. This file (`SBA-002`) — agent-facing constraints derived from v2.0
3. The current sprint prompt (from v2.0 Part VII)
4. Everything else (retired)

**If a prior sprint prompt, chat history, or earlier amendment tells you something that
conflicts with v2.0 or this file, v2.0 and this file win. Do not "reconcile" — STOP and
file an OPEN-QUESTIONS entry.**

---

## 1. The Split-Brain Invariant (FS-D1, FS-D3, FS-D4, FS-D6, FS-D7 in v2.0)

Two planes. No AI computation crosses between them.

| Plane | Where in v2.0 | What runs | What must NEVER run |
| :--- | :--- | :--- | :--- |
| **Policy Plane** | Layer 4, Stage A = `sched_ext` BPF + Rust; Stage B = native microkernel object | Pre-verified policy *selectors* and resource *controllers*. Agent writes an index into a pre-verified policy array. | Any LLM, ASR, TTS, semantic-graph, or token-sampling code. Any read of model memory. Any parse of untrusted text. |
| **Inference Plane** | Layer 5, TEE-isolated (OP-TEE now; Arm CCA in K3) | Full AI runtime: LLM, speech, semantic graph builder, intent compiler, model pager, KV manager. | Any direct kernel memory. Any raw socket. Any ambient filesystem path. Any capability not derived by the Semantic Gateway (Layer 6). |

**Hard rule:** If a proposed change would execute model weights, KV-cache pages, or
token-sampling code in kernel address space, **STOP** and file OPEN-QUESTIONS.
This is not negotiable in any sprint, including research spikes.

---

## 2. The Six Constraint Checks (apply to every file touched)

Before committing any file, the agent must verify:

1. **No model-in-kernel.** This file does not put LLM/ASR/TTS/sampler code in kernel space. If it does → STOP.
2. **No ambient authority.** The AI runtime reaches resources only through capability URIs
   (`cap://...`), never through paths, sockets, or global handles. If it doesn't → STOP.
3. **No taint bypass.** Untrusted text (`UNTRUSTED_SCREEN_TEXT`) cannot reach a RED-tier
   capability without passing through the Information-Flow Tracker (Layer 6). If it can → STOP
   and raise a **Security Incident** (v2.0 §7.8, rule 3).
4. **No ABI bypass.** Every AI-originated action goes through the Semantic Gateway and the
   Intent ABI (§5 of v2.0). No direct Accessibility call, no direct system-service call from
   the AI runtime. If it does → STOP.
5. **No trusted-AI assumption.** The code does not assume the AI runtime is honest. The
   Gateway is the trust boundary; the runtime is untrusted by design. If it does → STOP.
6. **No frozen-contract edits.** This file does not modify: the Intent ABI schema
   (`kernel/docs/intent-abi-v1.0.md`), the capability model TLA+ spec, the gateway policy
   rule table, the SELinux/`ind_cap_lsm` policy, or the GBNF grammar. Any such change is a
   `[PRD-AMENDMENT]` — file it, do not merge it.

Every sprint report must contain a `## Split-Brain Compliance` section listing, per
AI-adjacent file touched: which of the six checks was run, the result, and any
OPEN-QUESTIONS filed. Files that are *not* AI-adjacent are still listed with
"not AI-adjacent — checks 2–6 N/A."

---

## 3. Frozen Contracts (require [PRD-AMENDMENT] to change)

- `kernel/docs/intent-abi-v1.0.md` — the Intent ABI specification and test vectors (Sprint 20).
- `kernel/formal/capability-model.tla` — the capability model (Sprints 21–22).
- `kernel/formal/gateway-policy.tla` — the gateway policy rule table (Sprint 30 onward).
- `ind_action.gbnf` — the constrained-decoding grammar (MVP carry-forward; implementation
  detail, but frozen because the ABI compliance DoD depends on it).
- `system/sepolicy/private/ind_ai_daemon.te` and the Stage A `ind_cap_lsm` rule table.
- The `AGENTS.md` file itself.

Any diff touching these files must carry a `[PRD-AMENDMENT]` tag in the commit message
and require human sign-off. The agent may **propose** amendments via PR; it may never
merge them.

---

## 4. Anti-Hallucination Guard

The agent hallucinates when it has to *invent a value*. v2.0 pre-answers most questions,
but not all. Apply these rules whenever a question is not explicitly answered:

### 4.1 Never invent a number
If v2.0 does not specify a latency, memory budget, threshold, size, ratio, or timing —
**STOP and file OPEN-QUESTIONS.** Do not pick "a reasonable value." Every number in the
codebase must trace to a v2.0 row, a Part VIII quality gate, or a committed benchmark.

### 4.2 Never invent a library, framework, crate, or model
Every dependency must be named in v2.0 §3.2, §4, or an accepted amendment. If a task
implies a dependency v2.0 does not name, **STOP and file OPEN-QUESTIONS with a
recommendation** (do not pick unilaterally).

### 4.3 Never invent an interface
If a new AIDL/Binder method, a new capability URI scheme, a new intent action type, or a
new gateway denial code is needed and is not in the frozen contracts, **STOP.** Interfaces
are PRD-amendment territory, not sprint territory.

### 4.4 Never "improve" outside the whitelist
The sprint prompt lists the files in scope. **Any edit outside that list is a scope
violation.** No refactors, no renames, no "while I was here" cleanups. Second violation in
a phase → the sprint is regenerated with an explicit file whitelist.

### 4.5 Never claim done without a DoD log
Every sprint's Definition of Done in v2.0 Part VII is machine-verifiable. The agent runs
the commands on `hanoip` (or QEMU for pure logic) and pastes outputs into
`docs/agent-reports/<sprint>.md`. **No log = not done. Reject the sprint.**

### 4.6 Never resolve a contradiction by guessing
If two parts of v2.0, or v2.0 and a prior sprint prompt, or v2.0 and an amendment,
contradict — **STOP and file OPEN-QUESTIONS.** The Lead Architect resolves. Do not
proceed with the "most reasonable interpretation."

### 4.7 Never touch the Security Invariant without a security incident
Any change that:
- weakens the AI runtime's zero-network invariant (v2.0 Q5, §7.8 rule 3),
- lowers a speaker-verify threshold (§4.2 of the MVP carries to §4.3 of v2.0),
- relaxes the gateway action matrix,
- enlarges the TCB,
- or bypasses the transparency log,

is a **security incident**, not a sprint task. Halt, revert, file the report.

---

## 5. Where the Current Sprint Sits (as of this amendment)

Sprint 25 (per v2.0 §7.1) is **"Hexagon NPU Access via FastRPC — Research Spike."**
Its DoD is a decision memo (`kernel/docs/npu-enablement.md`), not shipping code.

Applying the six checks to Sprint 25:

| Check | Applies? | Note |
| :--- | :--- | :--- |
| 1 (no model-in-kernel) | **Indirectly** | The NPU enablement memo must state, explicitly, that the NPU is a **capability-scoped device** with SMMU-restricted memory and gateway-validated outputs — and that no inference code runs in the Linux kernel. If the memo proposes in-kernel inference on the NPU side, STOP. |
| 2 (no ambient authority) | **Yes** | Any FastRPC daemon/channel must be modelled as a capability-scoped device. The memo must name the capability URI scheme for NPU access (e.g. `cap://device/npu/hexagon`) or escalate that scheme's absence. |
| 3 (taint bypass) | **N/A for this sprint** | No taint-aware code written. |
| 4 (ABI bypass) | **N/A for this sprint** | No intent dispatch. |
| 5 (trusted-AI assumption) | **Yes — memo must state** | The NPU firmware is untrusted; outputs must be validated by the gateway before they influence capability derivation. The memo must say this. |
| 6 (frozen contracts) | **Yes** | The memo must **not** propose modifying the capability model or the Intent ABI. If it needs to, escalate. |

Sprint 26 (Adreno GPU compute) has the same shape. Real AI-adjacent code work begins at
**Sprint 27 (TEE Inference Prototype)**, and the six checks become load-bearing from there.

---

## 6. Reporting Template (mandatory from Sprint 25 onward)

Every `docs/agent-reports/<sprint>.md` must end with:

```markdown
## Split-Brain Compliance

### Files touched
- `<path>` — AI-adjacent: `<yes/no>` — checks run: `<1,2,3,4,5,6 subset>` — result: `<pass/fail>`
...

### Constraint violations raised
- `<none, or list of STOP events with OPEN-QUESTIONS link>`

### Benchmarks (if Policy Plane or Inference Plane touched)
- Policy-switch latency: `<measured>` (target per v2.0 §4.2.1)
- TEE memory isolation re-run: `<pass/fail>` (Sprint 27 attack harness)

### Frozen contract diffs (should be empty)
- `<none>`

### OPEN-QUESTIONS filed this sprint
- `<list or "none">`
```

---

## 7. Amendments Log Entry

| ID | Version | Date | Decision | Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **U-002** | v2.0 (unchanged) | 2026-10-26 | Retired `SBA-001`. Added `SBA-002` as the agent-facing constraint layer over v2.0. Split-brain invariant, six constraint checks, frozen-contract list, anti-hallucination guard, reporting template. Sprint 25 scope clarified as research spike. | SBA-001 was written against the two superseded PRDs; v2.0 already encodes every mechanism SBA-001 described. SBA-002 is deliberately short: it *points at* v2.0 and adds only the rules v2.0 does not itself state in agent-directed form. |
