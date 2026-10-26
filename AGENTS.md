# Agent Context — Ind OS

You are operating under the following binding documents, in order of precedence:

1. **`IND_OS_Unified_Master_PRD_v2.0_FINAL.md`** — the single source of truth.
   Everything in Parts I–VI is locked. Part VII is the sprint plan. Part VIII is the
   quality gates. Part IX–X are context. Do not contradict it.
2. **`docs/SBA-002_AGENT_CONSTRAINTS.md`** — agent-facing constraints derived from v2.0.
   Split-brain invariant, six constraint checks, frozen contracts, anti-hallucination
   guard, reporting template.
3. **The current sprint prompt** — pasted from v2.0 Part VII.

## Rules of engagement

- If v2.0 does not specify a value, library, interface, or behaviour — **STOP** and
  append `## OPEN-QUESTIONS` to the run report. Do not guess. (SBA-002 §4.)
- If a change touches a frozen contract (SBA-002 §3) — **STOP**, propose a
  `[PRD-AMENDMENT]` PR. Do not merge.
- If a change weakens a security invariant (SBA-002 §4.7) — **HALT**, revert, file a
  security incident report.
- Every sprint report ends with the `## Split-Brain Compliance` block (SBA-002 §6).
- No DoD log on device = not done. Reject the sprint.

## What you are not allowed to do, ever

- Decide architectures, libraries, model sizes, permission models, or repo layouts.
- Edit `IND_OS_Unified_Master_PRD_v2.0_FINAL.md`, SBA-002, or the frozen contracts.
- Add a dependency not named in v2.0 §3.2 or §4.
- Put model execution, ASR, TTS, or token sampling in kernel space.
- Give the AI runtime ambient authority (paths, sockets, global handles).
- Skip the verification log on any sprint's DoD.

If any of the above seems necessary to complete a task, the task is mis-specified.
File OPEN-QUESTIONS and wait.
