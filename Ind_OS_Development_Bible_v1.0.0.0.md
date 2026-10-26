# IND OS — THE DEVELOPMENT BIBLE

**Document ID:** INDOS-BIBLE-001
**Version:** 1.0.0.0
**Date:** 2026-10-26
**Status:** RATIFIED — PROJECT START AUTHORIZED (Sprint SP-00 may begin)
**Supersedes:** Nothing. Extends INDOS-PRD-001 (v0.1) and INDOS-PRD-002 (v1.0).
**Audience:** Founder, all engineers (kernel, userspace, AIS, security, hardware),
all executing agents, independent reviewers, external auditors.
**Primary target device (first hardware milestone):** Motorola Moto G60 4G (`hanoip`, Snapdragon SDM732G)
**Primary development target (first ~60% of the programme):** ARM64 QEMU virt platform

---

## Versioning scheme for THIS document (binding)

`MAJOR.MINOR.PATCH.REVISION`

- **MAJOR** — change to the constitution (Section 5), the gates (Section 19), or any founder-level directive (DEC items).
- **MINOR** — phase/sprint structure changes, new requirements (REQ IDs), risk register changes.
- **PATCH** — clarifications, wording fixes, no semantic change.
- **REVISION** — editorial/formatting only.

Every change goes through Section 21 change control. There is no silent edit. Ever.

---

## 0. How to use this document (READ FIRST — binding for all agents and humans)

1.  This document is the **single source of truth (SSOT)** for Ind OS development.
    If any instruction, ticket, code comment, chat message, or earlier PRD
    conflicts with this document, **this document wins**. Deviations are allowed
    ONLY through the change-control process in Section 21.
2.  **Never assume. Never hallucinate.** Every undecided item is marked
    `[PEN-NNN]` with the exact question. When an executing agent reaches a
    `[PEN-NNN]` item, it MUST stop and ask the founder. It MUST NOT pick an
    answer silently. **Silence is not approval.**
3.  Provisional defaults that engineering may proceed on **until the founder
    overrides them** are marked `[DEF-NNN]`. Defaults are not decisions; each
    default has a named sunset review gate.
4.  Every requirement carries a stable ID (`REQ-*`). Code, tests, commits, and
    sprint reviews MUST reference requirement IDs. A commit that cannot name the
    requirement it implements is out of process (CI enforces this — Rule N-10).
5.  **Every sprint ends with a validation review** (Section 8.4 — pass/fail
    only, no partial credit). Every phase ends with a **hard gate** (Section 19).
    A failed validation or gate triggers the stop/revise protocol — never silent
    continuation.
6.  Sections 0–7 and the Non-Negotiable Rules (Section 5) are frozen except by
    founder-signed change request.
7.  **PEN-013 (Section 7.3) MUST be resolved by the founder before Sprint SP-00
    begins.** No Phase 0 sprint starts on an unresolved PEN-013. This is the
    single largest open question in the programme, larger than crypto or GPU.

---

## 1. Relationship to prior documents — and an explicit divergence record

### 1.1 Lineage

- **INDOS-PRD-001 (v0.1)** — the original PRD: Android-foundation positioning,
  evidence discipline (D-08), feasibility gates (§13), recovery-before-flashing
  (§7.5), no schedule promises without evidence (D-09), separation of engineering
  truth from human value (§19). All of these remain in force where not
  contradicted here.
- **INDOS-PRD-002 (v1.0)** — the from-scratch build-ready PRD: kernel from
  scratch (DEC-001), hybrid microkernel (DEF-001), Rust (DEF-002), the
  AIS–ENF separation, phases/sprints/gates G0–G10, the PEN/DEF register.

### 1.2 🔴 RECORDED DIVERGENCE (documentation integrity — Rule N-08 applies)

In the audit document accompanying the PRD-002 review, **Option A was the
Android foundation** and **Option C was "write your own kernel,"** which the
audit advised *against* for a first product. PRD-002 stated "Option A — a genuinely
new kernel written from scratch," creating a labeling collision that an
independent reader could misread as the founder choosing the audit's
recommended path when in fact the founder chose its opposite.

**This Bible resolves the collision permanently:**

> **DEC-001 (restated):** Ind OS will use a **from-scratch kernel written from
> scratch** — the path the audit labeled Option C and recommended *against*
> for the first product. The founder has reviewed the audit's reasoning
> (schedule risk, driver famine, ecosystem risk — now registered as risks R-01,
> R-02, R-15 in Section 23) and overrides it. The founder's reasons are recorded
> in `docs/DECISIONS/DEC-001-founder-reasoning.md` (to be authored at SP-00) and
> the choice is subject to re-examination at Gate G0. This is a deliberate,
> informed founder decision — not an oversight.

From this document forward, the labels "Option A/B/C" from the audit are
deprecated. The only valid reference is **DEC-001: from-scratch path**.

### 1.3 Consequence stated plainly (no spin)

A from-scratch kernel has **no drivers** for the Moto G60's Qualcomm SoC
(SDM732G), display, modem, Wi-Fi, GPU, audio, or power management. PRD-001's
hardware-dependency matrix now becomes a *build-or-port-or-replace* matrix,
addressed in Phase 9 (hardware enablement) and risks R-01/R-02.

---

## 2. Product thesis and goal

### 2.1 The goal — stated in one sentence first

**Goal (Phase 1–9):** Boot a from-scratch operating system — Ind OS — from a
cold start, first on QEMU, then on the Moto G60 `hanoip`, with a working
userspace, a working security spine, and an advisory AI subsystem — every
claim backed by evidence at every gate.

**Goal (beyond):** Only after the OS boots and is proven do we build the
application layers and ecosystem on top. Sequence is law: **OS first, ecosystem
second.** No app-framework sprint may start before Gate G5 (a living userspace),
and no ecosystem investment may proceed before PEN-013 is resolved (Section 7.3).

### 2.2 Working positioning

*A personal computing platform, built from the kernel up, in which intelligence
is a first-class operating-system capability and the user is the final
authority over everything the system does.*

### 2.3 What "maximum AI integration" means concretely (design intents)

- **AI is an OS subsystem, not an app.** It lives beside the scheduler and
  memory manager, with defined, enforceable privileges — and it is *deniable*:
  any AI recommendation can be refused, audited, or disabled by the user.
- **The kernel is AI-aware:** scheduling, memory, power, and I/O policies take
  structured, advisory signals from the intelligence subsystem.
- **Security is enforced below the AI.** The AI can *propose*; the kernel's
  capability system *disposes*. No model output ever becomes authority (N-04).
- **The system explains itself.** Every consequential action carries a
  human-readable, plain-language rationale, and is reversible unless physically
  impossible.
- **The OS is inspectable by its own intelligence:** a diagnostic layer answers
  "why did the system do X?" with citations to actual logs, configuration, and
  decision records.

---

## 3. Definitions and terminology

| Term | Meaning |
|---|---|
| **Ind OS** | The complete operating system built under this Bible. |
| **IndK (Ind Kernel)** | The from-scratch kernel. Codename until founder names it `[PEN-003]`. |
| **IndUS (Ind Userspace)** | The from-scratch userspace: init, core services, shell, system libraries. |
| **AIS** | Adaptive Intelligence Subsystem (Section 6.4). Advisory only. |
| **ENF** | Enforcement Layer: kernel capability system, policy engine, consent broker, audit log. Never bypassable by AIS. |
| **TCB** | Trusted Computing Base: kernel core + capability system + policy engine + consent broker + audit. Everything else — including every model — is untrusted by definition. |
| **Host** | The development machine (Linux/macOS). The host is not part of Ind OS. |
| **Artifact** | Any build output: binaries, images, docs, test results. All artifacts are hashed and logged. |
| **Gate** | A pass/fail phase milestone (Section 19). No partial pass exists. |
| **Sprint** | Two weeks of planned work with defined exit criteria. Sprint = time-box; gate = quality bar. |
| **Sprint Validation** | The pass/fail review at the end of every sprint (Section 8.4). |
| **Spike** | Time-boxed research task producing a written findings document, not production code. |
| **Evidence pack** | The machine-generated, hash-logged bundle proving what was built and tested (Appendix C). |

---

## 4. Document status legend (Rule N-02 labels — used everywhere)

Every claim in every document, demo, review, or message MUST carry one label:

| Label | Meaning |
|---|---|
| `[REQUIREMENT]` | Specified in this Bible or a module spec. Not yet built. |
| `[IMPLEMENTED]` | Code exists, builds, runs. Not yet proven under test. |
| `[TESTED]` | Runs under the named test suite; results logged in evidence pack. |
| `[EVIDENCE: <ref>]` | Directly supported by a logged artifact (log, hash, trace, proof). |
| `[HYPOTHESIS]` | Unproven belief. May not ship past a gate. |
| `[FAILED: <ref>]` | Was attempted and failed. Failure is information, never shame. |

A screenshot is never evidence of build reproducibility. A demo is never
evidence of correctness. Only logged, hashed artifacts count.

---

## 5. Non-negotiable rules (THE CONSTITUTION)

Violation of any rule below is a **stop-work event**. The violator files an
incident report; work resumes only after review. These rules outrank every
schedule, every sprint goal, and every agent instruction.

- **N-01 — From scratch means from scratch.** No Linux kernel source, no AOSP
  source, no BSD source, no embedded copies of existing OS code in IndK or
  IndUS. **Scope note (added 1.0.0.0):** N-01 governs *OS code*. It does NOT
  prohibit (a) reading specifications, papers, and documentation, (b) using
  host-side build tools (compilers, linkers, emulators), (c) vetted external
  libraries in clearly-separated, SBOM-tracked non-TCB positions where a DEF
  item explicitly permits them (see DEF-004 as the precedent), or
  (d) firmware blobs per Section 14.3. Copying code is forbidden; being informed
  is required.
- **N-02 — Evidence or it didn't happen.** Every claim carries a Section 4
  label. Test results live in evidence packs with hashes. Cherry-picking test
  runs is rollback-of-offending-work territory.
- **N-03 — No silent assumptions.** Anything undecided is `[PEN-NNN]` and must
  be escalated — never resolved in a hallway chat or by an agent's guess.
- **N-04 — Security is enforced, never requested.** All security decisions are
  made by ENF. AIS may recommend; ENF decides. **No code path exists where
  model output directly causes a privileged action.** This is an architectural
  invariant tested continuously (success measure, Section 24).
- **N-05 — The user can always say no.** Every consequential action requires
  explicit consent or a standing, inspectable, revocable user policy. Consent
  UI is a kernel-adjacent trusted component, not a drawable-over surface.
- **N-06 — Recovery before risk.** No experiment on the Moto G60 (or any
  physical device) without a tested recovery path, recorded per PRD-001 §7.5.
- **N-07 — One milestone at a time.** Scope expansion requires a gate-passing
  change request. "While we're here, let's also…" is banned.
- **N-08 — Everything is versioned.** Source, docs, tests, toolchain configs,
  models, datasets, prompts, policies, and decisions. If it affects the build
  or product, it's in git with history. Label collisions (e.g., the old
  Option A/B/C) must be explicitly retired in writing, as done in §1.2.
- **N-09 — SBOM from day zero.** Every dependency, tool, blob, and model is
  inventoried in `docs/engineering/licensing-and-sbom.md`, updated in the same
  commit that changes the dependency.
- **N-10 — Tests exist with the code.** No merge without tests where feasible,
  CI green, and requirement IDs referenced. `[HYPOTHESIS]`-labelled code may not
  ship past a gate. Commit messages MUST reference `REQ-*` or `DEC/DEF/PEN-NNN`
  IDs; CI enforces the format.
- **N-11 — No flashing without gate clearance.** Physical-device writes occur
  only at the gates that name them (Section 19), and only after recovery proof.
- **N-12 — Founder's word resolves `[PEN-NNN]`.** Only the founder (or a
  delegate named in writing per the Section 21.4 founder-unavailability
  protocol) may resolve a pending item. Agent answers to pending questions are
  invalid even if technically correct.
- **N-13 — Honesty over momentum.** A failed gate is information, not shame.
  Hiding failure, redefining a gate to pass it, or cherry-picking test runs is
  grounds for rollback of the offending work. **This Bible applies N-13 to
  itself:** §7.4 records a corrected timeline estimate that supersedes the
  internally inconsistent "~78 sprints (~3 years) — realistically 5–10 years"
  statement in PRD-002 §8. Both numbers could not be true; one is now the plan
  of record, the other retired.

---

## 6. Architecture vision

### 6.1 Layered model

```text
┌─────────────────────────────────────────────────────────────┐
│  UX Shell: intent surface, explanations, consent, language   │  [Phase 8]
├─────────────────────────────────────────────────────────────┤
│  Application layer: Ind-native apps + capability manifests   │  [Phase 8;
│  (Android-compat strategy per PEN-013 — resolved at SP-00)   │   ecosystem later]
├─────────────────────────────────────────────────────────────┤
│  System services: storage, network, identity, update,      │  [Phase 5, 7]
│  telemetry (opt-in, local-first), device policy              │
├─────────────────────────────────────────────────────────────┤
│  AIS (Adaptive Intelligence Subsystem)                       │  [Phases 4–6]
│  - intent broker, model runtime, memory/graph store,          │
│  - proactive services, explanation engine                    │
│  ★ ADVISORY ONLY — holds no privilege of its own           │
├─────────────────────────────────────────────────────────────┤
│  Enforcement boundary (ENF)                                  │  [Phase 3–4]
│  - capability system, policy engine, consent broker,         │
│  - audit log (append-only, hash-chained), sandboxing       │
├─────────────────────────────────────────────────────────────┤
│  IndUS core userspace                                        │  [Phase 5]
│  - init, service manager, IPC libraries, user-mode drivers   │
├─────────────────────────────────────────────────────────────┤
│  IndK (from-scratch kernel)                                  │  [Phases 1–4]
│  - scheduler, VM/MMU, IPC primitives, VFS, driver framework│
│  - syscall surface, AIS-facing telemetry hooks (read-only)   │
├─────────────────────────────────────────────────────────────┤
│  Bootloader / firmware (device-specific; see §14)            │
├─────────────────────────────────────────────────────────────┤
│  Hardware: QEMU virt (dev) → Moto G60 hanoip (milestone)    │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Kernel architecture choice `[DEF-001]`

**Provisional default: hybrid microkernel** — a small privileged core
(scheduling, memory, IPC, capabilities, interrupts) with conventional in-kernel
drivers only where hardware reality demands (interrupt controller, timers,
early console). Everything else, including most drivers and the AIS model
runtime, runs **unprivileged in user space**.

- Rationale: capability-based security (N-04) is structurally cleanest with a
  small TCB; a from-scratch monolith would make formal verification of the
  security core infeasible; a pure microkernel would make first boot on
  Qualcomm hardware dramatically harder. Hybrid is the honest middle.
- **Sunset review:** end of Phase 2 / Gate G3. If evidence shows the hybrid
  boundary is wrong, founder decides via change request. Do not relitigate
  mid-phase.

**Provisional default `[DEF-002]`: implementation language Rust** for kernel
and system services, with a fully specified `unsafe` boundary per crate.
Assembly only for arch entry/exit, context switch, and early boot. C permitted
only for freestanding boot shims Rust cannot yet express on target. Memory
safety is a security requirement, not a preference. Sunset review at G2.

### 6.3 The AIS–ENF separation (the most important diagram in this document)

```text
   ┌────────────┐  advisory   ┌──────────────────┐
   │  AIS       │ ───────────▶│  Policy Engine    │
   │ (models,   │  structured │  (rules, user      │
   │  intent,   │  proposals  │   standing policy)│
   │  prediction)│            └────────┬─────────┘
   └─────┬──────┘                     │ grant/deny
         │ telemetry (read-only)      ▼
         │                    ┌──────────────┐     ┌────────────┐
         └──────────────────▶ │  Capability  │◀───▶│  Audit log │
            (kernel emits     │  enforcement │     │ (append-   │
             events; AIS      │  (kernel)    │     │  only)     │
             cannot write)    └──────────────┘     └────────────┘
                                     │
                                     ▼
                              privileged action
                                     │
                     ┌───────────────┴───────────────┐
                     ▼                               ▼
            explicit consent path          standing-policy path
        (Consent Broker, trusted UI)   (user-editable, revocable)
```

**Laws encoded here (requirements, not hopes):**

- `REQ-AI-001` AIS runs with zero privileges of its own. It holds capabilities
  like any user process, each individually revocable.
- `REQ-AI-002` Every privileged action flows through the capability check.
  There is no "AIS service account with root."
- `REQ-AI-003` All AIS proposals are structured, typed, schema-validated, and
  logged *before* execution. Denied proposals are logged too.
- `REQ-AI-004` The user can inspect, edit, pause, and wipe AIS state
  (conversation memory, learned patterns, model cache) from a UI AIS itself
  cannot draw over or intercept.
- `REQ-AI-005` Every action AIS causes carries an explanation record (model
  version, input provenance, policy rules fired, consent record id).
- `REQ-SEC-001` The TCB = kernel core + capability system + policy engine +
  consent broker + audit. Everything else is untrusted by definition,
  including every model.

### 6.4 What "maximum AI integration" means, layer by layer

| Layer | AI capability | Enforced by |
|---|---|---|
| Scheduler | workload classification; latency/energy hints | kernel uses hints only within hard fairness bounds |
| Memory | app-usage prefetch, compression policy hints | kernel decides; OOM behavior never model-controlled |
| Power | usage-pattern-aware suspend/deferral proposals | user policy caps what may be deferred |
| I/O / storage | predictive prefetch; anomaly detection on access patterns | anomaly → alert, never auto-quarantine without policy |
| Network | per-app traffic classification; on-device leak detection | firewall rules enforced by kernel; AI only proposes rule changes |
| Security | behavioral anomaly detection; scam/phishing analysis of content shown to user | detection result is advisory to user + policy engine |
| UX | intent surface ("do what I mean"), proactive suggestions, explanation engine | consent broker for anything consequential |
| System health | crash prediction, log summarization, self-healing *proposals* | healing executes only via approved, rollbackable playbooks |
| Development (host) | AI-assisted code review, test generation, proof assistance for formal specs | human review; generated code subject to the same rules |

Explicit **non-uses** (banned by design): AI in the boot chain; AI modifying its
own enforcement-relevant code paths at runtime; AI holding signing keys; AI
making irreversible decisions; AI replacing the consent broker.

---

## 7. Decision registers

### 7.1 Resolved by founder directive (recorded)

- **DEC-001** Kernel and userspace written from scratch (the audit's "Option C").
  Date: 2026-10-26. Restated in §1.2 with divergence record and reasoning file.
- **DEC-002** AI integration is a core OS objective, at maximum feasible depth,
  bounded by N-04/N-05. Date: 2026-10-26.
- **DEC-003** Multi-year horizon accepted; no calendar commitments made to any
  external party. Date: 2026-10-26.
- **DEC-004 (1.0.0.0)** Cryptography posture: **a vetted, audited external TLS
  library (rustls or equivalent) is the default** for the TLS layer, SBOM-tracked,
  treated as a vetted external dependency in a non-TCB position. A from-scratch
  TLS 1.3 implementation is NOT the default; it requires explicit founder
  sign-off as an exception. Rationale recorded in DEF-004 below and risk R-05.
  Date: 2026-10-26.

### 7.2 Provisional defaults (proceed unless founder overrides; each has a named review gate)

- **DEF-001** Hybrid microkernel architecture. Review at G3.
- **DEF-002** Rust for TCB; asm only at arch boundary. Review at G2.
- **DEF-003** 2-of-3 threshold release signing. Review at G7.
- **DEF-004** **TLS via vetted external library (rustls or equivalent)**,
  SBOM-tracked, pinned to an audited version, compiled from source by our
  toolchain, with its `unsafe`/crypto posture documented in the threat model.
  *Supersedes PRD-002's provisional default of from-scratch TLS 1.3, which
  contradicted the document's own risk register ("crypto from scratch introduces
  exploitable flaws — Critical") and created a Heartbleed-class default. A TLS
  library is not OS code; using one does not violate N-01 any more than using
  the Rust compiler does.* From-scratch TLS remains possible only as a
  founder-signed exception (PEN-006 closed by DEC-004 unless the founder
  reopens it). Review at G7.

### 7.3 Pending questions — AGENTS MUST ASK, NOT ASSUME

Ordered by blocking severity. PEN-013 blocks Sprint SP-00.

- **PEN-013 (BLOCKS SP-00)** **Android application compatibility strategy.**
  The founding positioning targets "the next billion users," who depend on
  roughly 30 Android apps (WhatsApp, PhonePe, YouTube, etc.) that will have no
  Ind-native version for years. A from-scratch kernel + from-scratch userspace
  + new app framework means **zero existing apps run**. Options (founder must
  pick one, with reasoning recorded):
  - (a) Android apps in a compatibility VM / runtime layer (Wine-for-Android
    class — massive scope; feasibility spike required; would be its own
    programme track).
  - (b) Android runtime reimplementation from scratch (Alien Dalvik / Waydroid
    class — years of work; legal review of clean-room constraints required).
  - (c) **Zero app compatibility**; Ind-native apps only; first users are
    developers and enthusiasts, not the "next billion" (positioning statement
    must be revised accordingly).
  - (d) Abandon from-scratch; return to an Android foundation (reverses DEC-001).
  Until resolved: Phase 0 may proceed ONLY on toolchain/repo/spec work (SP-00 to
  SP-03 are PEN-013-independent); Phase 1 kernel sprints require PEN-013
  resolved, because the answer may alter the syscall surface and app framework
  design.
- **PEN-001** Does "from scratch" exclude using *specifications or headers*
  from other OS projects (ELF format, DTB spec), or only code? *(Default until
  answered: specs/formats allowed, code forbidden — reflected in N-01 scope
  note.)*
- **PEN-002** Rust toolchain licensing for kernel use — accept Rust's standard
  licensing, or legal review before SP-01?
- **PEN-003** Name the kernel and OS beyond codenames (IndK/IndUS are internal).
- **PEN-004** GPU strategy on Moto G60: (a) software rendering, (b)
  reverse-engineered/open-source driver effort, (c) out of scope for first
  hardware milestone.
- **PEN-005** If `hanoip` secure boot rejects our kernel image entirely, which
  fallback: dev-board-first programme, or documented partial chain?
- **PEN-006** Cryptography posture — **provisionally CLOSED by DEC-004** (vetted
  external TLS library is the default; from-scratch is the exception requiring
  founder sign-off). Founder may reopen at G7 review.
- **PEN-007** Signing-key hardware: HSM purchase, threshold keyholders, and the
  legal entity holding them.
- **PEN-008a** AIS model strategy — training track: will Ind OS self-train any
  models from scratch? If yes, what compute budget and data governance?
- **PEN-008b** AIS model strategy — adaptation track: which open-weights models
  may be adapted in-house, under which licenses, and what vetting pipeline?
- **PEN-008c** AIS model strategy — hybrid posture: how do (a) and (b) combine
  per capability (intent, sentinel, explanation), and which capabilities get
  which track? *(Split from PRD-002 PEN-008 because the three tracks have
  different resource plans and gate dependencies.)*
- **PEN-009** On-device vs. hybrid inference for the intent surface: what may
  leave the device, under which consent, in which phases?
- **PEN-010** Hardware budget for a reference ARM64 dev board (and which board),
  strongly recommended before Moto G60 work begins.
- **PEN-011** Team: may we state assumed minimum staffing per phase for
  planning, or is staffing a founder-only input at each gate?
- **PEN-012** Repository visibility: public from day one, private until G5, or
  public at G9?

---

*…continues in Part 2 (Section 8: programme structure, phases and sprints).*
