# IND OS — THE DEVELOPMENT BIBLE

**Document ID:** INDOS-BIBLE-001
**Version:** 1.1.0.0
**Date:** 2026-10-27
**Status:** RATIFIED — GATE G10 AUDIT PASSED & RATIFIED; PHASE 11 (PRIMARY APPLICATIONS) AND PHASE 12 (BETA & v1.0 STABLE) ROADMAP ACTIVATED.
**Supersedes:** Nothing. Extends INDOS-PRD-001 (v0.1), INDOS-PRD-002 (v1.0), and absorbs in full INDOS-UX-PRD-001 (v1.0.0.0) and INDOS-APPS-PRD-001 (v1.0.0.0).
**Audience:** Founder, all engineers (kernel, userspace, AIS, security, hardware, applications), all executing agents, independent reviewers, external auditors.
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
- **DEF-UX-001** Motion curve family: `cubic-bezier(0.16, 1, 0.3, 1)` (ease-out-expo class) for enter, standard ease for exit, 220 ms default duration. Review at G8.
- **DEF-UX-002** Accent family: indigo-violet (`#7C6CFF`) on deep charcoal (`#0E0F14`) base (tokens in Appendix A). Review at G8.
- **DEF-UX-003** Grid unit: 260 px / 12 px gutters / 24 px screen margins. Review at G8 against real-device ergonomics.
- **DEF-APP-001** Notification grouping: by app by default; user-configurable to per-sender. Review at G11.
- **DEF-APP-002** Share sheet target order: Most-recently-used first; alphabetical fallback. Review at G11.
- **DEF-APP-003** Camera default capture format: JPEG (highest compatibility); user-configurable to PNG. Review at G11.
- **DEF-APP-004** Browser default engine & media pipeline: WebKit-class vendored engine with ARM64 NEON software decoders (dav1d/libvpx), Wayland-to-uxd graphics bridge, and ClearKey/sandboxed Widevine dual-track DRM (PEN-APP-008 resolved). Review at G11.
- **DEF-APP-005** Weather default data source: user API key with bundled coarse model fallback (PEN-APP-002 pending). Review at G11.
- **DEF-APP-006** Clipboard sensitive-content timeout: 60 seconds auto-clear for OTPs/passwords. Review at G11.
- **DEF-APP-007** Gallery trash retention: 30 days local soft-delete before permanent cryptographic key destruction. Review at G11.
- **DEF-APP-008** Music library root: `/user/media/music/`. Scan on first launch; user-configurable roots. Review at G11.

### 7.3 Pending questions — AGENTS MUST ASK, NOT ASSUME

Ordered by blocking severity. PEN-013 blocks Sprint SP-00; PEN-APP-008 is resolved for Sprint SP-154.

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
- **PEN-UX-001** Haptic hardware reality on hanoip: does the FocalTech/PMU path expose a controllable haptic driver, or is feedback deferred to visual tick / audio click? *(Default: visual tick; haptic is a stretch goal.)*
- **PEN-UX-002** Mascot final name & art direction — codename Gaja stands until founder approves; is elephant motif finalized?
- **PEN-UX-003** Theme support at launch: dark-only (glassmorphism optimized) vs dual dark/light themes?
- **PEN-UX-004** Wallpaper / personalization scope: static system gradient vs user image backgrounds?
- **PEN-UX-005** Voice input activation: push-to-talk in Intent Bar only, or hotword listening? *(Default: push-to-talk to eliminate always-listening privacy exposure.)*
- **PEN-UX-006** Third-party app chrome freedom: enforce system tokens universally, or permit custom app styling? *(Default: enforce tokens universally; relaxation is a founder decision at G8.)*
- **PEN-UX-007** Typeface licensing decision: bundled open-license grotesque for Latin + Noto Indic companions.
- **PEN-APP-001** Maps / Turn-by-Turn Navigation app: deferred in v1.0; review at G12 whether to enter v1.1 or remain browser-fallback.
- **PEN-APP-002** Weather data source resolution: (a) user API key, (b) bundled coarse model, (c) Ind-hosted proxy, (d) browser fallback. Default: (a) with (b) fallback.
- **PEN-APP-003** Music streaming integration: deferred in v1.0; review at G12 for v1.1 scope.
- **PEN-APP-004** Native Email client: deferred in v1.0; review at G12 based on SP-168 user feedback.
- **PEN-APP-005** Third-party app store / distribution model: deferred; proposal scheduled at SP-177 SDK sprint.
- **PEN-APP-006** AIS default-on policy: all AIS features off by default unless founder grants specific default-on exception with rationale. Default: all off.
- **PEN-APP-007** Biometric Face Unlock: deferred to Phase 12 pending hanoip hardware review and threat modeling.
- **PEN-APP-008 (RESOLVED — PRODUCTION SPECIFICATION FOR SP-154)** **Browser Engine & Multimedia Pipeline Architecture.**
  Resolves the browser engine selection and video/media stack into a production-grade 4-part specification:
  1. **Graphics Subsystem (`uxd` / Wayland Bridge):** Use `indus-ux` (`uxd`) with `CAP_DISPLAY_RENDER` for native Ind OS apps. For the Phase 10 Android container runtime, implement a lightweight Wayland-to-`uxd` IPC translation shim so Android video apps can pass framebuffers directly to our double-buffered display pipeline (`INV-COMP-001`).
  2. **Video Decoding Pipeline (ARM64 NEON Software Decoders):** Replace `libblkio` with **dav1d (AV1)** and **libvpx (VP9/VP8)** compiled as in-tree `#![no_std]` or safe C-wrapper Rust crates targeting **ARM64 NEON SIMD vector extensions** (`SP-96`). This guarantees smooth 1080p 60fps software video decoding without requiring closed-source GPU/DSP driver blobs in the microkernel TCB (`Rule N-01`).
  3. **Streaming Engine & Egress Governance:** Stream HLS/DASH over `indus-net` TLS 1.3 sockets (`ADR-004`). The **Network Leak Sentinel (`INV-LEAK-001`)** monitors media stream bandwidth without blocking consented video playback.
  4. **DRM & Content Protection Dual-Track:**
     - *Native Ind OS Apps:* Use **ClearKey AES-128** for open-standard, DRM-free, or self-hosted media streams.
     - *Commercial Android Apps (Phase 10 Container):* Pass Widevine DRM requests to the sandboxed Android container runtime (`LXC/Waydroid`), isolating proprietary Widevine CDM libraries completely outside the Ind OS microkernel TCB.
- **PEN-APP-009** RCS protocol support: carrier-dependent; resolution scheduled at G12.
- **PEN-APP-010** Camera beauty filters & AR lenses: deferred; default: reject to keep camera pipeline clean and deterministic.
- **PEN-APP-011** Calculator graphing mode: deferred; review at G12.
- **PEN-APP-012** Browser extensions ecosystem: deferred to Phase 13+.
- **PEN-APP-013** Calendar cloud sync: deferred to Phase 12; user-opt-in, provider-scoped, zero-PII required.
- **PEN-APP-014** Voice recorder legal notice jurisdiction: auto-detect via SIM vs explicit first-use user selection. Default: explicit user selection.
- **PEN-APP-015** Gallery on-device face clustering: deferred to v1.1; local-only opt-in.

---

---

## 8. Programme structure

### 8.1 Honest timeline (N-13 applied — supersedes PRD-002 §8)

PRD-002 presented "~78 sprints (~3 years at pure velocity) — realistically 5–10 years." Those two numbers were irreconcilable, and this Bible does not repeat the error. The corrected, internally consistent statement of record is:

> **Plan of record: 126 two-week sprints (SP-00 through SP-125) ≈ 5 years at pure velocity (no gate failures, no re-baselining, no founder-decision latency). Realistic horizon: 5–10 years** with gate re-baselining per §8.3, hardware reality per §8.5, and the founder's accepted multi-year posture (DEC-003). No calendar dates are committed externally. Duration is expressed in sprints, phases, and gates only.

Sprint counts below are the plan of record — estimates to be re-baselined at every gate. They are not promises.

### 8.2 Phase map (plan of record)

| Phase | Sprints | Count | Theme | Hard gate | Shippable intermediate |
|---|---|---|---|---|---|
| 0 | SP-00 – SP-03 | 4 | Foundation: repo, toolchain, specs, evidence harness | G0 | Reproducible empty workspace |
| 1 | SP-04 – SP-09 | 6 | Kernel bootstrap: boot to serial on QEMU | G1 | Booting kernel |
| 2 | SP-10 – SP-15 | 6 | Core kernel: memory, scheduling, exceptions, objects | G2 | Multitasking kernel |
| 3 | SP-16 – SP-23 | 8 | IPC + capabilities + audit (the security spine) | G3 | — |
| 4 | SP-24 – SP-31 | 8 | VFS, user-mode driver framework, SMP, HAL skeleton | G4 | Kernel + storage + drivers on QEMU |
| 5 | SP-32 – SP-40 | 9 | Userspace: init, services, shell, developer experience | G5 | **★ DEVELOPER OS v0.1** (Phase 0.5 milestone, §8.7) |
| 6 | SP-41 – SP-50 | 10 | AIS subsystem + ENF policy engine + consent broker | G6 | — |
| 7 | SP-51 – SP-59 | 9 | Network stack, updates, key ceremony, service completion | G7 | Connected OS with update story |
| 8 | SP-60 – SP-69 | 10 | UX shell, app framework, accessibility, intent surface | G8 | Demoable product shell |
| 9 | SP-70 – SP-109 | 40 | Moto G60 bring-up — honest re-implementation (§8.5) | G9 | **Ind OS on founder hardware** |
| 9.5 | SP-110 – SP-115 | 6 | Performance & usability pass (targets set at G4, §8.12) | G9.5 | Usable, fast OS |
| 10 | SP-116 – SP-126 | 11 | Hardening, formal verification, external audit, beta readiness | G10 | Beta candidate (Ratified) |
| 11 | SP-127 – SP-170 | 44 | Primary applications & cross-app services (12 apps + 5 system surfaces) | G11 | Feature-complete phone OS |
| 12 | SP-171 – SP-184 | 14 | Post-G11 Beta cohort, SDK, ecosystem, hardening, v1.0 stable | G12 | **★ Ind OS v1.0 STABLE RELEASE** |

**Total: 185 sprints (SP-00 – SP-184).** Formally re-baselined at Gate G10 ratification: Phase 11 (SP-127–SP-170) and Phase 12 (SP-171–SP-184) activate the complete product roadmap to v1.0 stable.

### 8.3 Re-baselining rule

At every gate, sprint counts for the remaining phases are re-estimated from evidence and re-baselined in writing (a change-log entry in this Bible, MINOR version bump). A re-baseline is not a failure; presenting a stale estimate as current IS an N-13 violation.

### 8.4 Sprint contract and sprint validation (binding, every sprint, no exceptions)

**Sprint contract (agreed at sprint start):**
1. Sprint goal tied to named REQ IDs.
2. Exit checklist — each item observable, testable, documentable (done = demonstrable + tested + documented).
3. Named risks from the register touched this sprint.
4. Every `[PEN-NNN]` touched this sprint is re-asked to the founder, not re-assumed.

**Sprint validation (pass/fail review at sprint end — the gate discipline applied at sprint granularity):**
1. Each exit-checklist item is demonstrated live or from the evidence pack — with the REQ ID, test log, and artifact hash visible.
2. Traceability check: every commit in the sprint names a live REQ/DEC/DEF/PEN ID (CI enforces; random 20-commit quarterly audit per §21).
3. Risk register delta reviewed: new risks added, mitigations statused honestly.
4. Gate-distance report: "what now stands between us and the next gate."
5. **Verdict: PASS or FAIL.** No partial credit.
   - **FAIL** → stop/revise for that workstream: written deviation report, root cause named, plan revised before new sprint work begins on that track. Independent sprints may continue (§21.3).
   - A sprint ending with <80% of its exit checklist done is automatically a FAIL verdict, even if the demonstrated items passed.

### 8.5 The QEMU → hardware cliff — stated honestly (fixes PRD-002's 9-sprint Phase 9)

PRD-002 allocated 9 sprints to Moto G60 bring-up, implicitly assuming QEMU virtio drivers port to Qualcomm hardware. **They do not port; they must be re-implemented.** Concretely:

- **Storage:** UFS on Qualcomm is not virtio-blk. A new driver, new error handling, new power-loss behavior.
- **Display:** the G60 panel is DSI + Adreno GPU with no vendor documentation — not a framebuffer. Default per PEN-004 is CPU software rendering via a simple display driver written from panel timing data.
- **Touch:** an I²C/SPI controller with vendor protocol quirks — discovered by probing, documented as found, never guessed.
- **Power/PMIC:** effectively undocumented; charging/thermal behavior is reverse-engineered or bounded conservatively.
- **Boot chain:** Qualcomm secure boot may reject an unsigned kernel entirely — Gate G9's first test (§7.3 PEN-005, SP-71).

Therefore Phase 9 is **40 sprints of planned work**, and it is explicitly classified as *re-implementation*, not *porting*. To reduce (not eliminate) this cost, **Phase 4 includes a Hardware Abstraction Layer (HAL) skeleton** (SP-31): every driver built for QEMU must route device-specific access through the HAL interface, so Phase 9 re-implements behind the interface rather than redesigns the system. The HAL is a mitigation, not a miracle: 40 sprints stands as the honest estimate.

### 8.6 Phase 0 — Foundation (SP-00 – SP-03) → Gate G0

**Goal:** a reproducible, honest workspace where a machine (or human) can execute sprints deterministically. **Precondition: PEN-013 resolved by founder before SP-00 begins (§7.3).**

- **SP-00 — Repository & governance bootstrap**
  - Monorepo layout (kernel/, userspace/, ais/, apps/, tools/, ci/, docs/, evidence/).
  - CONTRIBUTING, CODEOWNERS, commit-message spec with REQ-ID tagging (N-10), branch protection, CI skeleton.
  - Decision register created (`docs/DECISIONS/`), seeded with every DEF/PEN item from Part 1 §7, including the §1.2 divergence record.
  - `docs/DECISIONS/DEC-001-founder-reasoning.md` authored: the founder's written reasons for the from-scratch path.
  - PEN-013 resolution recorded with reasoning; ecosystem/positioning consequences noted.
  - Deliverable: empty `main` builds a no-op artifact, hash-logged.
- **SP-01 — Toolchain & environment**
  - Rust pinned version (DEF-002), `aarch64-unknown-none` cross build, QEMU pinned, linker, GDB; U-Boot vs. direct QEMU `-kernel` chosen via documented spike.
  - Host environment as code (containerfile): any engineer/agent reproduces bit-identical setups.
  - Deliverable: `make bootstrap` from clean machine → toolchain ready ≤ 30 min.
- **SP-02 — Specification skeleton**
  - Module specs for Phase 1–2 components (boot, memory, interrupts, scheduler) as `docs/architecture/*.md`: interfaces, invariants, error model. Specs precede code (N-02/N-10).
  - Deliverable: specs reviewed; per-module invariants list approved.
- **SP-03 — Evidence & test harness**
  - Test runner: QEMU headless, serial capture, golden-log comparison, hash recording into `evidence/`.
  - Deliverable: CI job that fails loudly on any deviation.

**Gate G0 exit:** clean-machine bootstrap works; CI green on empty tree; specs approved; founder (or delegate) has signed ratification of this Bible; PEN-013 resolution and DEC-001 reasoning on file.

### 8.7 Phase 1 — Kernel bootstrap (SP-04 – SP-09) → Gate G1

**Goal:** IndK boots on ARM64 QEMU virt to a serial console it owns (REQ-KRN-101).

- **SP-04** — Boot entry & exception vectors: AArch64 reset, EL transitions, vector table, panic handler with structured output.
- **SP-05** — Serial console: PL011 early console driver; leveled logging with boot-stage markers.
- **SP-06** — Memory discovery: QEMU DTB parsing, physical memory map, frame allocator, kernel address-space layout.
- **SP-07** — Kernel heap & core data structures: allocator, intrusive lists; the `unsafe`-budget file opened (tracked from here forever, Section 24).
- **SP-08** — Interrupts: GICv2/v3 driver, timer interrupts, IRQ dispatch, critical-section discipline.
- **SP-09** — Boot hardening & evidence pack: cold-boot matrix (N=20 boots, all logged), boot-time budget report, "hello from IndK" milestone.

**Gate G1 exit (PRD-001 §9.3 standard):** 20/20 cold boots pass; serial log uniquely identifies the build (git hash + timestamp + config hash); build reproduces from clean checkout; all G0 docs updated; corrupt-image rejection demonstrated.

### 8.8 Phase 2 — Core kernel (SP-10 – SP-15) → Gate G2

**Goal:** memory protection and preemptive multitasking exist; the kernel is no longer a boot script.

- **SP-10** — Virtual memory: page tables, per-process address spaces, kernel/user split, copy-on-write skeleton.
- **SP-11** — Processes & loading: process abstraction, own documented binary format + ELF-subset reader decision recorded as ADR; first user-mode "hello" process (real user mode).
- **SP-12** — Scheduler: run queues, priority classes, preemption, context switch (asm + safety wrapper), `sched_hint` API stub (advisory, documented no-op).
- **SP-13** — Exception & syscall surface: syscall ABI frozen (documented, versioned); per-process syscall permission bitmap as pre-capability scaffolding (replaced by capabilities in Phase 3).
- **SP-14** — Kernel object model: the objects Phase 3 capabilities will reference (process, endpoint, memory region, IRQ) — the object table is designed *before* security depends on it.
- **SP-15** — Stress & soak: fork-storm, memory-pressure, timer-storm tests; lock-discipline review; every `unsafe` block audited against budget.

**Gate G2 exit:** two+ isolated user processes preempt one another indefinitely under stress; address-space separation proven by test (cross-boundary write → fault, logged); syscall ABI doc signed; unsafe-budget review passed; scheduler latency distribution measured and recorded (this becomes the G4 performance baseline input).

### 8.9 Phase 3 — The security spine: IPC + capabilities + audit (SP-16 – SP-23) → Gate G3

**Goal:** REQ-KRN-104/105/108 live. This is the phase the whole OS stands on; it gets two extra sprints and the strictest gate in the programme.

- **SP-16** — IPC core: endpoints, send/recv with rights transfer, notifications, deadlines/timeouts; latency benchmark harness (G4 baseline input).
- **SP-17** — Capabilities I: unforgeable tokens, object-table integration, mint/grant/revoke, per-process namespaces.
- **SP-18** — Capabilities II: delegation with attenuation (rights shrink along delegation chains), revocation propagation, capability-leak test suite.
- **SP-19** — Policy engine v1 (ENF): declarative, human-readable rule language; rule evaluation on capability checks; default-deny; rules are versioned artifacts.
- **SP-20** — Audit v1: hash-chained append-only log of grants/denials/security events; log signing; tamper test (flip a bit → detection).
- **SP-21** — Consent Broker v1: trusted UI channel end-to-end (kernel-mediated input, isolated surface); consent record schema; the API every later service must use.
- **SP-22** — Adversarial self-review: internal red team attempts privilege escalation via IPC bugs, capability confusion, audit tampering; findings fixed or formally accepted.
- **SP-23** — Formal-spec groundwork + G3 evidence assembly (REQ-SEC-106 inputs: delegation monotonicity and revocation-soundness invariants written as checkable specs).

**Gate G3 exit (blocking):** capability invariants proven by test plus machine-checkable specs; no ambient authority anywhere (grep-level proof + runtime proof); IPC latency within budget; consent broker demonstrably un-spoofable by a malicious app (overlay/input-injection test → fails, logged); audit tamper test passes. **Failure = HARD STOP.** The security architecture is re-reviewed by the founder; Phase 4 does not begin on a broken spine.

### 8.10 Phase 4 — VFS, driver framework, SMP, HAL skeleton (SP-24 – SP-31) → Gate G4

**Goal:** the kernel hosts a real userspace with persistent storage and devices — **and** the HAL interface that makes Phase 9 a re-implementation behind an interface rather than a redesign.

- **SP-24** — VFS core: vnode layer, mount table, path resolution, file-handle capabilities (file access capability-mediated from day one — REQ-KRN-106).
- **SP-25** — tmpfs + IndFS: our journaling, checksummed persistent FS (design doc in SP-24; fsck; golden-image crash tests).
- **SP-26** — User-mode driver framework: device manager, IRQ broker, DMA/IOMMU story (SMMU on QEMU), driver protocol over IPC (REQ-KRN-107).
- **SP-27** — Reference drivers (user mode): virtio-blk, virtio-net, framebuffer, keyboard/mouse — proving the framework carries real devices without kernel privilege.
- **SP-28** — SMP: core bring-up, per-CPU run queues, lock refinement, IPIs, affinity (REQ-KRN-109).
- **SP-29** — Memory pressure & OOM: reclamation, swap decision (spike → DEF or PEN), OOM policy — deterministic, never model-controlled.
- **SP-30** — Kernel telemetry API v1: read-only ring buffer of scheduler/IPC/memory events for later AIS consumption (REQ-KRN-113); write-attempt-from-userspace denial test.
- **SP-31** — **HAL skeleton + performance baselines**: the Hardware Abstraction Layer interface (memory-mapped register access, IRQ, DMA, clocks/resets) that every driver must route through — designed with Phase 9's Qualcomm targets in mind. Includes `REQ-ARCH-101` standalone `indk-core` workspace crate decoupling (RFC-INDOS-2026-001). Simultaneously: **performance baselines measured and recorded** (boot time, IPC round-trip, syscall latency, context-switch cost, FS throughput) — these become the targets' *denominators* for the Phase 8/9.5 budgets (§8.12). DEF-001/002 sunset review held and minuted.

**Gate G4 exit:** user-mode driver suite serves storage+input+display on QEMU; FS survives randomized power-cut fuzzing (documented crash-consistency results); SMP scaling measured at 1/2/4 cores; telemetry API proven read-only; HAL interface adopted by 100% of reference drivers (grep + runtime proof); performance baseline report approved; DEF-001/002 reviews minuted.

### 8.11 Phase 5 — Userspace & the Developer OS (SP-32 – SP-40) → Gate G5

**Goal:** IndUS stands up: init, services, shell, developer experience — and the programme's **first shippable intermediate product** (§8.11 milestone).

- **SP-32** — init (REQ-USE-201): service manifests, dependency graph, crash-restart with backoff, boot stages mirrored to structured logs.
- **SP-33** — Core libraries (REQ-USE-202): alloc, collections, IPC bindings with capability-first ergonomics; `no_std`-friendly; docs with examples.
- **SP-34** — Device manager + hotplug events: capability handout per REQ-USE-203; virtio plug/unplug tests.
- **SP-35** — Shell v1: our own shell — parsing, pipelines as IPC chains; first dogfood milestone: a developer does a full work session inside Ind OS.
- **SP-36** — Logging & diagnostics (REQ-USE-204): unified log bus, redaction filters, `why` command (precursor of the explanation engine).
- **SP-37** — Settings & consent store (REQ-USE-206): integrity-protected, user-owned; the store ENF policy engine reads.
- **SP-38** — Developer experience: in-OS text editor viability spike, remote shell over serial/virtio, docs generated from source.
- **SP-39** — Integration test suite v1 (REQ-QA-201 groundwork): whole-system behavior tests — scripted user journeys, soak harness, clean-checkout reproduction procedure (timing target: ≤ 30 min).
- **SP-40** — Quality sprint: 72h soak, memory-leak sweep, docs parity audit (N-10), SBOM completeness audit (N-09); **Developer OS packaging**: installable QEMU image + docs + release notes.

**★ MILESTONE (Phase 0.5 intent): DEVELOPER OS v0.1** — declared at G5: a QEMU-bootable Ind OS with shell, editor, and text tools that a developer can genuinely use as a work environment. This is the programme's first externally demoable, fundable, community-growable artifact — roughly month 20 in the plan of record — deliberately created years before the full product. Talent, funding, and morale ride on it.

**Gate G5 exit:** team lives on Ind OS for a full working day (dogfood record, warts documented); every service has a manifest, capabilities, and logs; 72h soak with no kernel panic; settings/consent store passes tamper tests; Developer OS v0.1 image reproduces from clean checkout; founder demo session held.

### 8.12 Phase 6 — AIS + ENF policy + Consent (SP-41 – SP-50) → Gate G6

**Goal:** the AI-native heart, built *inside* the Phase 3 enforcement architecture — advisory AIS, real ENF. **Safety AND quality are both gated (REQ-AI-110).**

- **SP-41** — Model runtime sandbox (REQ-AI-101): signed-model loader only, isolated inference process with minimal capabilities, CPU backend, scheduler-enforced memory/time limits. Includes `REQ-AI-112` dynamic hardware profile manager (`IndHardwareProfile`, RFC-INDOS-2026-001).
- **SP-42** — Intent broker (REQ-AI-102): NL/system-event → typed proposal schema; schema registry; every proposal logged pre-routing (REQ-AI-003).
- **SP-43** — Policy engine v2: standing user policies as first-class objects (inspectable, revocable); conflict resolution; dry-run simulation ("what would happen if…").
- **SP-44** — Consent UX + broker v2: plain-language narratives (REQ-UX-301); AIS proposals rendered as *proposals*; user decisions recorded to audit.
- **SP-45** — Personal memory store (REQ-AI-103): encrypted, scoped, user-editable; retention enforced by ENF, not the model.
- **SP-46** — Explanation engine v1 (REQ-AI-104): every consequential action answerable with (action, rule fired, consent id, model version, input provenance).
- **SP-47** — Kernel-AI feedback loop: telemetry (SP-30) → sched/mem/power hints; provable bounds — hints cannot starve the realtime class, cannot delay consent UI, bounded influence — tested adversarially.
- **SP-48** — Anomaly sentinels v1 (REQ-AI-107): permission-use and storage-access baselines; alerts to user via trusted UI only.
- **SP-49** — **AIS intent quality sprint**: build the 200-intent evaluation corpus (target-language coverage per PEN-008c resolution); measure task-completion rate; iterate until trajectory meets the REQ-AI-110 target or an honest gap report is filed.
- **SP-50** — AIS red-team sprint: prompt injection via content, memory poisoning, confused-deputy via proposals; every failure → fix or policy; results minuted.

**Gate G6 exit (the soul of the product):** live demo — (a) AIS proposes, (b) ENF denies under standing policy, (c) user inspects, overrides once with consent, revokes the policy — all with explanation + audit trail; prompt-injection suite passes (no privileged action via injection); memory wipe provably destroys keys; **REQ-AI-110 quality target met on the frozen eval corpus**; founder signs AIS-charter addendum (the §6.4 non-uses).

### 8.13 Phase 7 — Network + updates + service completion (SP-51 – SP-59) → Gate G7

**Goal:** a connected OS with a trustworthy update story. Crypto posture per DEC-004/DEF-004 (vetted external TLS library; from-scratch only as founder-signed exception).

- **SP-51** — Network stack core (REQ-KRN-110): our Ethernet/IP/TCP/UDP subset; socket objects as capabilities; zero-copy goals documented honestly.
- **SP-52** — TLS integration: DEF-004 library integrated, pinned, SBOM-tracked, compiled from source by our toolchain; crypto posture documented in threat model; ADR records DEC-004 application.
- **SP-53** — DNS, DHCP, firewall: firewall enforced in-kernel; rule changes capability-gated.
- **SP-54** — Update service (REQ-USE-205): A/B slots, signed manifests, atomic switch, verified rollback; update policy user-controlled.
- **SP-55** — Key ceremony tooling (REQ-SEC-108): threshold signing workflow, HSM strategy per PEN-007, compromise playbook rehearsed (tabletop).
- **SP-56** — Leak sentinel (REQ-AI-107 network half): on-device traffic classification; findings → user alerts; no auto-blocking without user policy.
- **SP-57** — Interop: file sharing with other machines, remote shell over network, boot-to-network tests.
- **SP-58** — Integration suite v2: update-interruption fuzzing (power cut mid-update → boots previous slot), network fuzzing (malformed-packet storm), full-system soak with network up.
- **SP-59** — Reliability + SBOM sprint: third adversarial sprint without TCB compromise; license/SBOM refresh (N-09); G7 evidence assembly.

**Gate G7 exit:** update chain demonstrated end-to-end on QEMU with rollback; firewall denies-by-default inbound; leak sentinel catches a seeded leak in test; crypto ADR signed; SBOM refreshed; DEF-003/004 sunset reviews minuted.

### 8.14 Phase 8 — UX shell + app framework (SP-60 – SP-69) → Gate G8

**Goal:** the user-friendliness promise becomes visible as **Ind Surface v2** (ratified in `INDOS-UX-PRD-001`). App-framework design MUST respect the PEN-013 resolution (recorded at SP-00); if PEN-013 chose an Android-compat track, its feasibility spike lands here as a parallel workstream with its own budget — and does not expand scope silently (N-07).

#### 8.14.1 The Ind Surface v2 Architecture (1080 × 2460 Physical Display)
The physical display is partitioned into three immutable compositor zones (`INV-SHELL-HW-003`):

1. **Physical Zone 1: Status Bar (1080 × 64 px)**
   - Compositor-reserved system chrome (`CAP_SYSTEM_CHROME`).
   - Left: Gaja elephant state indicator glyph (States 1–5: Sleeping, Resting/Calm, Listening/Sampling, Reasoning/Pulsing, Alert/Warning), clock.
   - Right: Sovereign ENF Consent Beacon (`🛡️`), Wi-Fi RSSI (WCN3990), and PMIC Battery Fuel Gauge (`88%`, 4050mV).
2. **Physical Zone 2: Main Canvas (1080 × 2268 px)**
   - **Central Intent Bar (§6.1):** Prominent frosted glass capsule with Indic language pills (`[EN]`, `[HI]`, `[TA]`), push-to-talk (PTT) microphone trigger, and natural-language intent input (`"What would you like Ind OS to do?"`).
   - **Adaptive Bento Grid (§6.2):**
     - *Tile 1 (2×1 Ambient Card):* Date, local weather, Gaja resting mascot, and Rule N-01 Zero External Crates banner.
     - *Tile 2 (1×1 PMIC Battery Fuel Gauge):* Large SoC display, 4050mV, 6000mAh, TSENS thermal status.
     - *Tile 3 (1×1 Sovereign Security & Integrity):* Gate G10 Ratification, 0 TCB Crates, Fastboot + EDL 9008 Verified.
     - *Tile 4 (2×1 Core Native Apps Row §10):* Core native apps (`Files`, `Settings`, `Why?`, `Notes`, `Terminal`).
     - *Tile 5 (2×2 Proactive AIS Proposal Card):* `SUGGESTION` watermark (`INV-SHELL2-001`), plain-language confidence (`"I'm fairly sure (94%)"`), `[Apply 60Hz]`, `[Dismiss]`, and `[Why? Receipt]`.
   - **Trusted Consent Surface (`PlaneId::HardwareIsolated`, §8):** Un-spoofable hardware-isolated modal plane with green consent boundary, plain-language capability description, `[Grant Access]`, `[Deny]`, and `[Why?]`.
3. **Physical Zone 3: Navigation Bar (1080 × 128 px)**
   - Minimal gesture strip with 4px haptic pill, `[◀ BACK]`, `[ — HOME — ]`, and `[🛡️ CONSENT]`.

#### 8.14.2 Normative Design Token System (`INV-TOK-001`)
Shipped UI references Appendix A normative tokens only; raw hex values, arbitrary font sizes, and uncalibrated durations fail code review (`INV-TOK-001`).
- **Base Colors:** `TOK-COLOR-BASE-0` (`#0E0F14`), `TOK-COLOR-BASE-1` (`#16181F`), `TOK-COLOR-GLASS-BG` (`rgba(255,255,255,0.06)`), `TOK-COLOR-GLASS-BORDER` (`rgba(255,255,255,0.12)`).
- **Accents & Alerts:** `TOK-COLOR-ACCENT` (`#7C6CFF`), `TOK-COLOR-OK` (`#4CD9A6`), `TOK-COLOR-ALERT` (`#FF6B6B`).
- **Typography & Motion:** `TOK-TYPE-S1..H1`, `TOK-SPACE-U: 4px`, motion curves `cubic-bezier(0.16,1,0.3,1)`, duration 220 ms.

#### 8.14.3 Phase 8 Sprint Breakdown (SP-60 – SP-69)
- **SP-60** — Compositor & display server (user mode, capability-fed): damage tracking, vsync, zero-copy swapchain, input routing with focus integrity (`INV-COMP-001`).
- **SP-61** — Shell v2 layout: zone map live (64/2268/128), Status Bar + Nav Bar rendered by compositor, Gaja state machine (`INV-SHELL-HW-003`).
- **SP-62** — Intent Bar + input path: zero-heap transliteration-first IME architecture, Hindi/English/Hinglish tables loaded (`INV-INTENT-UI-003`).
- **SP-63** — App framework: plain-language capability manifests (REQ-USE-207), sandbox spawn, bento tile hosting (`INV-APP-001`).
- **SP-64** — Accessibility services (REQ-UX-304, P0): semantic tree IPC (`INV-A11Y-001`), magnification, switch access; tested with real users recruited this sprint.
- **SP-65** — Proactive engine UI (REQ-AI-106): suggestion feed, 4-mode dial, `SUGGESTION` watermark (`INV-SHELL2-001`), explainable Why-receipts.
- **SP-66** — Core apps suite: FilesApp (IndFS), SettingsApp (consent surface), WhyExplorerApp (causal timeline), NotesApp (Indic IME dogfood), TerminalApp.
- **SP-67** — Performance budget enforcement: frame-time (≤16.6ms @ p95), cold-start (≤250ms), zero-heap render fast-path (`INV-COMP-002`) enforced in CI.
- **SP-68** — UX validation sprint: external users (target 15+, mixed languages/abilities) run scripted tasks; success/error rates recorded; top-10 friction list becomes backlog.
- **SP-69** — Polish + G8 evidence assembly.

**Gate G8 exit:** a non-engineer completes the demo script unaided; accessibility tasks pass with real users; no app can draw over or fake the consent surface (adversarial UI suite); explanation engine answers questions about *any* visible action with citations; founder product review held with recorded decisions.

### 8.15 Phase 9 — Moto G60 bring-up: honest re-implementation (SP-70 – SP-109) → Gate G9

**Goal:** Ind OS boots on the founder's real device — re-implementing every driver behind the HAL, per §8.5. **Recovery-first: N-06 / PRD-001 §7.5 is law here.**

- **SP-70** — Programme preparation: reference dev board bring-up per PEN-010 resolution (strongly recommended *before* touching the phone); hardware lab setup (serial access, power measurement, recovery workstation).
- **SP-71** — Device evidence pack (N-06): exact variant, bootloader state/version, partition map, current ROM hash captured per PRD-001 §9.2 *without flashing*; known-good recovery image + verified restore procedure tested **before any experiment**.
- **SP-72** — Boot-chain proof: can a non-stock kernel image boot at all? Documented attempts, logs, fallback analysis (PEN-005). **If blocked → stop; options return to founder. No workarounds without sign-off.**
- **SP-73** — Kernel core on hardware: GIC, timers, UART, memory map from device DT (extracted from stock firmware, provenance documented); serial milestone at Gate G1 evidence standard, now on hardware.
- **SP-74 – SP-79** — **Storage:** UFS driver behind HAL (SP-74–75: bring-up, then power-loss resilience on-device); **Display:** panel bring-up with CPU software rendering per PEN-004 (SP-76–77: timings, compositor path); **Input:** touch controller driver, basic input → shell milestone (SP-78–79).
- **SP-80 – SP-85** — **Power/thermal/charging:** PMIC basics — battery readout, charging state, thermal throttling hooks (documented conservative bounds where undocumented); **USB:** basic device-mode or host-mode support per founder priority.
- **SP-86 – SP-95** — **Connectivity drivers:** Wi-Fi/BT firmware-blob integration per §14.3 policy (blob loaded by our driver, SBOM-tracked, never in TCB); **sensors** (accelerometer, light, proximity); **audio** bring-up if feasible, else honestly classified out-of-scope-with-reason.
- **SP-96 – SP-103** — **On-device AIS:** runtime ported to ARM64, CPU inference benchmarked (latency/thermal), honest feasibility report per PEN-008c/009 resolutions; **system services on device:** update chain, settings, shell — the full G5-standard userspace now running on hardware.
- **SP-104 – SP-109** — **Hardening on device + hardware matrix v1 (REQ per PRD-001 §8.3):** per-component status: works / partial / blocked / out-of-scope-with-reason; GPU, modem, camera, audio honestly classified; recovery route re-tested *after* experiments; G9 evidence assembly (N predeclared ≥ 10 reproducible cold boots; serial + display evidence; acceptance pack per PRD-001 §9.5: source revision, build logs, hashes, device IDs, boot logs, repeatability, recovery results, matrix, limitations, reviewer sign-off).

**Gate G9 exit:** reproducible cold boots of the identified build; recovery route proven *after* the experiments; hardware matrix published internally; acceptance pack complete. **This gate is the PRD-001 founding milestone — celebrate it honestly, claim nothing beyond it.**

### 8.16 Phase 9.5 — Performance & usability pass (SP-110 – SP-115) → Gate G9.5

**Goal:** from-scratch OSs are slow for years by default (Linux 1991, Haiku, Redox — the precedent is universal). This phase exists so slowness is a scheduled debt, not a surprise. **Targets are not invented here: they were defined at G4 from measured baselines (§8.10) and ratified with numbers at G5.** Plan-of-record target classes:

| Class | Examples | Source of target |
|---|---|---|
| Boot | cold boot to shell, on QEMU and on device | G4 baseline + founder-approved budget |
| Launch | app cold-start, shell-start | G4 baseline + budget |
| Interaction | frame time, input latency | G8 budgets enforced in CI |
| Voice/intent | intent → proposal card latency (device, CPU inference) | G6 SP-49 measurements |
| Memory | per-app-class memory budgets | G4/G8 measurements |

- **SP-110** — Full-system profile: every subsystem measured on QEMU and device; top-20 hotspots list.
- **SP-111 – SP-113** — Hotspot elimination sprints (kernel paths, IPC, FS, compositor), each ending with the CI perf gates re-run and evidence recorded.
- **SP-114** — Usability debt sprint: top-10 friction list from SP-68 + dogfood warts from G5 onward.
- **SP-115** — G9.5 evidence assembly.

**Gate G9.5 exit:** all ratified performance targets met `[TESTED]` on both QEMU and device — or each miss carries a founder-signed exception with a named fix phase. No silent misses.

### 8.17 Phase 10 — Hardening, verification, beta readiness (SP-116 – SP-126) → Gate G10 [RATIFIED]

**Goal:** turn the working system into something defensible. **STATUS: Gate G10 passed & ratified at SP-126.**

- **SP-116 – SP-118** — Formal verification push (REQ-SEC-106): capability-core invariants proved (delegation monotonicity, revocation soundness); proof artifacts in CI.
- **SP-119** — External security audit (REQ-SEC-109): contracted TCB review; findings triaged; 0 open criticals confirmed before any outward "secure" language.
- **SP-120** — Red-team programme (REQ-SEC-110): multi-week adversarial campaign; results and fixes documented internally.
- **SP-121** — Production ops: signing ceremonies rehearsed for real; update infrastructure staging; incident-response playbook; support/docs readiness.
- **SP-122** — Beta cohort planning: devices (QEMU + hanoip + dev boards), consent-first telemetry (opt-in, local-first — PRD-001 §7.4), user-research protocol.
- **SP-123 – SP-126** — Continuous hardening cadence: rolling security sprints, patch SLAs, cold-boot double-buffered animation sequence (2.8s budget), Phase 10 completion.

**Gate G10 exit [PASSED & RATIFIED]:** external audit closed; verification proofs in CI; update chain operated for real; beta plan approved; G11 and G12 roadmaps activated with evidence per Rule N-02.

---

### 8.18 Phase 11 — Primary Applications & Cross-App Services (SP-127 – SP-170) → Gate G11

**Document ID:** INDOS-APPS-PRD-001 (Phase 11 Activation)  
**Goal:** Deliver the 12 primary native applications and 5 system surfaces that transform Ind OS from a functional microkernel OS into a **complete daily-driver phone**. Every app strictly conforms to the Universal App Charter (§8.20).

#### 8.18.1 The App Catalog (Tiers 1, 2, and 3)
- **Tier 1 (Mandatory — Must ship in v1.0 stable):** Phone, Messages, Contacts, Camera, Gallery, Clock, Calculator, Browser, and System Surfaces (Lock Screen, Notification Center, Quick Settings, Recents, Global Search).
- **Tier 2 (Expected — Must ship or explicitly carry PEN):** Calendar, Music (local-only), Weather (offline-first), Recorder.
- **Tier 3 (Deferred with Documented PEN):** Maps/Navigation (`PEN-APP-001`, browser fallback), Banking/UPI (follows `PEN-013`), Email (`PEN-APP-004`, browser fallback), Podcasts (`PEN-APP-003`), Fitness/Health (deferred), Streaming Video (DRM out-of-scope), App Store (`PEN-APP-005`, SDK shipping at SP-177).

#### 8.18.2 Phase 11 Sprint Breakdown (44 Sprints: SP-127 – SP-170)
- **SP-127 — App Framework Extensions:** Shared back-stack manager, window management, chrome library, lifecycle hooks (`REQ-APP-COMMON-006`).
- **SP-128 — Cross-App Services:** Universal Share Sheet (`REQ-APP-COMMON-007`), Intents Registry (`REQ-APP-COMMON-008`), Clipboard with sensitive OTP redaction (`REQ-APP-COMMON-011`), Global Search foundation (`REQ-APP-COMMON-010`).
- **SP-129 — System Surfaces I:** Lock Screen (`REQ-APP-COMMON-015`) + Notification Center (`REQ-APP-COMMON-013`). Consent broker accessible on hardware plane; notifications dismissible; priority bands.
- **SP-130 — System Surfaces II:** Quick Settings (`REQ-APP-COMMON-014`) + Recents/App Switcher (`REQ-APP-COMMON-016`). Live compositor thumbnails, gesture kill.
- **SP-131 — Notification Discipline & AIS Kill Switch:** User-initiated notification rules (`REQ-APP-COMMON-005`), AIS hardware kill switch accessible ≤2 gestures from lock screen (AIS Charter §12.3).
- **SP-132 – SP-134 — Phone App:** Voice call placement/reception via modem (`REQ-APP-PHONE-001`), Emergency dialer from lock (`REQ-APP-PHONE-002`), in-call UI on hardware-isolated plane (`REQ-APP-PHONE-004`), call history (`REQ-APP-PHONE-003`), Suraksha scam-call shield (`REQ-APP-PHONE-008`), AIS proposal calling (`REQ-APP-PHONE-009`).
- **SP-135 – SP-138 — Messages App:** SMS/MMS engine (`REQ-APP-MSG-001/002`), threaded view (`REQ-APP-MSG-003`), attachments (`REQ-APP-MSG-004`), Suraksha banking OTP screen-share blackout (`REQ-APP-MSG-005`), scam SMS on-device classifier (`REQ-APP-MSG-006`), encrypted at rest (`REQ-APP-MSG-011`).
- **SP-139 – SP-140 — Contacts App:** Contact CRUD (`REQ-APP-CT-001`), `.vcf` import/export (`REQ-APP-CT-002`), caller ID integration (`REQ-APP-CT-004`), groups (`REQ-APP-CT-007`), per-query AIS consent (`REQ-APP-CT-006`).
- **SP-141 – SP-142 — Clock App:** Alarms with DND override (`REQ-APP-CLK-001/002`), multi-timers (`REQ-APP-CLK-003`), stopwatch (`REQ-APP-CLK-004`), world clock (`REQ-APP-CLK-005`), wake-up on hardware-isolated plane (`REQ-APP-CLK-006`).
- **SP-143 — Calculator App:** Standard arithmetic (`REQ-APP-CALC-001`), scientific mode (`REQ-APP-CALC-002`), history (`REQ-APP-CALC-003`), unit conversion (`REQ-APP-CALC-004`), inline AIS calculation result (`REQ-APP-CALC-005`).
- **SP-144 – SP-149 — Camera App:** Still photo capture (`REQ-APP-CAM-001`), MP4 video capture (`REQ-APP-CAM-002`), zero-latency gallery handoff (`REQ-APP-CAM-003`), status bar persistent red recording dot (`REQ-APP-CAM-004`), capture modes (`REQ-APP-CAM-005`), offline OCR/barcode scan (`REQ-APP-CAM-009`).
- **SP-150 – SP-153 — Gallery App:** Browse photos/videos (`REQ-APP-GAL-001`), playback (`REQ-APP-GAL-002`), basic deterministic editing (`REQ-APP-GAL-003`), 30-day trash (`REQ-APP-GAL-004`), share sheet integration (`REQ-APP-GAL-005`), opt-in offline content search (`REQ-APP-GAL-007`).
- **SP-154 – SP-157 — Browser App:** Standards-compliant rendering (`REQ-APP-BRW-001`), HTTPS-only enforcement (`REQ-APP-BRW-002`), tabs (`REQ-APP-BRW-003`), history/bookmarks (`REQ-APP-BRW-004`), download to Files (`REQ-APP-BRW-005`), per-site sandboxing (`REQ-APP-BRW-006`), anti-fingerprinting (`REQ-APP-BRW-007`), private browsing (`REQ-APP-BRW-008`), reader mode (`REQ-APP-BRW-010`), AIS read-only grammar summary (`REQ-APP-BRW-011`).
- **SP-158 – SP-160 — Calendar App:** Local events (`REQ-APP-CAL-001`), month/week/day/agenda views (`REQ-APP-CAL-002`), notifications (`REQ-APP-CAL-003`), `.ics` import/export (`REQ-APP-CAL-004`), AIS scheduling proposal (`REQ-APP-CAL-006`).
- **SP-161 – SP-163 — Music App:** Local playback MP3/FLAC/OGG/WAV (`REQ-APP-MUS-001`), lock-screen & notification media controls (`REQ-APP-MUS-002`), background playback (`REQ-APP-MUS-003`), audio focus arbitration (`REQ-APP-MUS-004`), metadata editing (`REQ-APP-MUS-005`).
- **SP-164 – SP-165 — Weather App:** Current conditions (`REQ-APP-WTH-001`), 24h & 7-day forecast (`REQ-APP-WTH-002`), offline-first coarse model (`REQ-APP-WTH-003`), manual pull-to-refresh (`REQ-APP-WTH-004`), per-fetch location consent (`REQ-APP-WTH-006`).
- **SP-166 — Recorder App:** Audio recording to WAV/OGG/FLAC (`REQ-APP-REC-001`), status bar persistent recording dot (`REQ-APP-REC-002`), live waveform visualization (`REQ-APP-REC-004`), opt-in on-device transcription (`REQ-APP-REC-005`), first-use jurisdiction legal notice (`REQ-APP-REC-007`).
- **SP-167 – SP-168 — Cross-App Integration & Usability Testing:** 15+ external users run frozen scripted task suites across all 12 apps. Measure task completion, friction points, a11y.
- **SP-169 – SP-170 — System Integration, Accessibility Audit & Gate G11 Pack:** 72h soak with all 12 apps active; a11y task success verified; performance budgets `REQ-PERF-401..415` green; evidence pack assembled in `evidence/G11/`.

#### 8.18.3 Gate G11 Exit Criteria (All Required, Pass/Fail Only)
1. All Tier 1 apps pass acceptance tests with `[TESTED]` evidence in their evidence pack.
2. All Tier 2 apps shipped or explicitly deferred with a documented PEN.
3. Every app conforms to all 10 items of the Universal App Charter (§8.20).
4. Accessibility task success verified with real users across screen-reader, magnification, and switch access (SP-168).
5. Offline-first invariant (`INV-APP-003`) verified: apps perform primary tasks with network revoked.
6. All app-class performance budgets (`REQ-PERF-401..415`) met in CI.
7. Independent non-engineer completes the frozen multi-app demo script unaided.
8. Battery accounting: no app exceeds declared 24h power budget.
9. Founder and steering group sign-off on G11 acceptance pack.
10. All `AppManifest` definitions public and verified against least-privilege capability matrix.

---

### 8.19 Phase 12 — Post-G11 Beta, Hardening & v1.0 Stable Release (SP-171 – SP-184) → Gate G12

**Goal:** Transform the feature-complete app layer into a bulletproof, audited, production-ready **Ind OS v1.0 Stable Release**.

#### 8.19.1 Phase 12 Sprint Breakdown (14 Sprints: SP-171 – SP-184)
- **SP-171 — Beta Cohort Onboarding:** Onboard first 50–100 diverse external users across geography, language, and abilities.
- **SP-172 — Beta Feedback Triage:** Systematic categorization of bug reports, UI friction, and performance edge cases.
- **SP-173 – SP-176 — Priority Remediation Sprints:** Two-week hardening cycles addressing top beta findings.
- **SP-177 — Developer Ecosystem & SDK:** Public release of the Ind Native SDK, app templates, documentation, and capability APIs.
- **SP-178 — Deferred-App Review:** Formal resolution of `PEN-APP-001..005` (Maps, Banking, Email, Podcasts, Store) for v1.1 roadmap.
- **SP-179 — Language Expansion:** First roadmap Indic language (Tamil or Telugu) shipped across all primary apps.
- **SP-180 — Update Infrastructure Staging:** Fleet-wide signed-OTA staging, transparency log integration, incident response drill.
- **SP-181 — Expanded Beta Cohort:** Deploy to 500+ device fleet, telemetry review, stability soak.
- **SP-182 — Second Cohort Fix Sprint:** Final defect remediation and regression pass.
- **SP-183 — v1.0 Release Candidate:** Release candidate builds, golden image hashes, signed manifests, launch documentation.
- **SP-184 — v1.0 Stable Release & Gate G12 Ratification:** Gate G12 formal audit, public launch authorization.

#### 8.19.2 Gate G12 Exit Criteria (Final Commercial Release)
1. Beta cohort NPS ≥ 40 or documented remediation path.
2. Zero open critical bugs; high-severity bugs ≤ 5 with scheduled maintenance dates.
3. Verified OTA update executed successfully on live beta fleet.
4. Ind Native SDK shipped with working third-party dogfood application.
5. All deferred-app PEN items resolved with recorded decisions.
6. Formal founder and steering group sign-off on v1.0 release.
7. Transparency log entries published and independently verifiable.
8. External security audit ratified with 0 unmitigated vulnerabilities.
9. Production images threshold-signed (`DEF-003`) and published.

---

### 8.20 The Universal App Charter & Cross-Cutting Architecture

Every application running on Ind OS is bound by the **Universal App Charter**. An app that violates any charter requirement fails review regardless of feature completeness.

#### 8.20.1 The 10 Binding Charter Commitments
1. **Minimum Capability Set (`INV-APP-001`):** Every capability in `AppManifest` must trace to a user-facing feature in the capability matrix (Appendix B). Unearned or unused capabilities are strictly removed.
2. **Offline-First Baseline (`INV-APP-003`):** The app must perform its core tasks completely offline. Network connectivity is an enhancement, never a prerequisite for local utility.
3. **Semantic Tree (`REQ-APP-COMMON-001`, `INV-A11Y-001`):** Every screen, modal, and control must publish a structured semantic tree over the accessibility IPC bus. Surfaces lacking a tree cannot hold focus.
4. **Consent-Driven Acquisition (`INV-APP-002`):** No capability is acquired silently or granted at install. Every privilege requires explicit user consent via the Consent Broker.
5. **Why-Explorer Receipts (`REQ-AI-104`):** Every consequential action produces a cryptographically signed `ProvenanceRecord` viewable in WhyExplorer.
6. **Design Tokens Only (`INV-TOK-001`):** All UI references Appendix A normative design tokens exclusively. Raw hex codes and ad-hoc dimensions fail CI review.
7. **AIS as Proposal, Never Authority:** AI actions are routed as proposals through ENF; the app cannot execute an AIS recommendation without user confirmation or explicit standing policy.
8. **Dignified Indic Localization (`REQ-APP-COMMON-I18N-001`):** First-class support for English, Hindi (Devanagari), and Hinglish transliteration at launch, with dynamic 0.85×–2.0× typography scaling.
9. **Battery Accounting (`REQ-APP-COMMON-002`):** App power consumption is reported to the accounting API; abnormal draw triggers an unsuppressible user notification in the Privacy Feed.
10. **Test Evidence Integrity:** Every feature must carry machine-checkable `[TESTED]` evidence in `evidence/` before gate submission.

#### 8.20.2 Shared UI Component Library (`userspace/lib/ind-ui`)
Apps must utilize the standardized, glassmorphic primitive set (`Card`, `List`, `Form`, `Dialog`, `Progress`, `EmptyState`, `ErrorState`). Rewriting core UI primitives in app code is prohibited (`INV-APP-UI-001`).

---

---

---

## 9. Requirements baseline and v1.0.0.0 deltas

### 9.1 Baseline statement (what exists, what this section adds)

The complete requirement catalog is anchored in three places:

- **Part 1 §6.3** — the AIS–ENF laws (`REQ-AI-001` through `REQ-AI-005`, `REQ-SEC-001`).
- **The PRD-002 §6 baseline** — `REQ-KRN-101..113`, `REQ-USE-201..207`, `REQ-SEC-101..110`, `REQ-AI-101..109`, `REQ-UX-301..305`. These remain in force unless superseded here.
- **This section (§9)** — the v1.0.0.0 deltas. New requirements added in response to Expert Review R-01, the QEMU→hardware cliff finding, and the governance gaps identified during Bible consolidation.

A requirement is only live if it appears in one of these three places. Ad-hoc REQ IDs invented in a sprint are out of process (Rule N-03).

### 9.2 REQ-AI-110 — AIS intent quality (new, in response to Review R-01 F-07)

**REQ-AI-110 (P1) — AIS intent quality target.** AIS intent interpretation must meet a documented task-completion rate on a defined evaluation corpus before Gate G6.

- **Corpus:** 200 intents, frozen at SP-49. Language coverage determined by PEN-008c resolution. Complexity distribution: 40% simple single-action ("turn on torch"), 40% multi-step ("message mom that I'll be late"), 20% adversarial/noisy ("asdf; what's the weather? also call Ravi").
- **Success criterion:** ≥ 80% task completion. Task completion = the intent produces a proposal that ENF accepts and the user does not retract within 30 seconds as "wrong."
- **Failure consequence:** Gate G6 fails. No outward claim of "user-friendly" AIS is permitted until the target is met on a subsequent gate attempt.
- **Re-baselining:** The 80% threshold may be adjusted by founder change request with new evidence — not lowered to pass a gate (Rule N-13).
- **Corpus immortality:** The frozen corpus is versioned and cannot be replaced with an easier corpus to pass. New corpora can be added; old ones remain for regression.

### 9.3 REQ-QA series — reproducibility and validation (new, in response to Review R-01 F-09)

**REQ-QA-201 (P1) — Gate reproducibility.** Every gate demo must be reproducible from a clean checkout by an independent engineer or agent in ≤ 30 minutes on the documented host environment.

- **"Independent"** means: not the engineer who wrote the demo, and not an AI agent that has been given the answer key.
- **"Documented host environment"** means: the container in `SP-01`-compliant form, or the equivalent host OS + toolchain pins in `docs/engineering/`.
- **"Reproducible"** means: the same commands produce the same observable result (demo passes, tests pass, evidence pack generated) with the same hashes modulo timestamps.
- **Failure consequence:** the gate does not pass. The demo is a party trick until it reproduces.

**REQ-QA-202 (P1) — Evidence-pack completeness.** Every gate evidence pack must include the exact command sequence used by the independent reproducer, plus the successful run log. Packs missing this are invalid.

**REQ-QA-203 (P2) — Quarterly external reproduction.** At minimum once per phase, an external party (community member, contractor, founder's delegate) reproduces a gate demo from a clean checkout. The attempt log enters the evidence pack whether successful or not.

### 9.4 REQ-PERF series — performance and usability targets (new, in response to Review R-01 F-08)

All numeric targets are set at **G4 from measured baseline**, not invented by the Bible author. The Bible defines the *framework*; the numbers are evidence-driven.

| REQ ID | Metric | Target-setting moment | Target applies at |
|---|---|---|---|
| **REQ-PERF-101 (P1)** | Cold-boot-to-interactive (QEMU) | Baseline measured SP-31; budget set at G4 | G5, G8, G9.5 |
| **REQ-PERF-102 (P1)** | App cold-launch | Baseline at SP-31; budget at G4 | G8, G9.5 |
| **REQ-PERF-103 (P1)** | Intent → proposal-card latency | Baseline at SP-49 (on QEMU); device budget at G9 | G6, G9.5 |
| **REQ-PERF-104 (P1)** | Compositor frame time (p95) | Baseline at SP-31; budget at G4 | G8, G9.5 |
| **REQ-PERF-105 (P1)** | Idle OS RAM footprint | Baseline at SP-31; budget at G4 | G5, G8, G9, G9.5 |
| **REQ-PERF-106 (P1)** | Sustained-inference thermal ceiling | Baseline at SP-49; device budget at G9 | G6, G9.5 |
| **REQ-PERF-107 (P1)** | Cold-boot-to-interactive (device) | Measured at SP-73 (first hardware boot); budget set at G9 | G9, G9.5 |

**Rule:** Phase 9.5 (§8.16) must demonstrate measurable improvement against Phase 8 and Phase 9 baselines on *every* REQ-PERF metric. A metric that has no baseline is a metric the programme cannot optimize.

### 9.5 REQ-GOV series — governance requirements (new, per Review R-01 F-10)

Governance is not an "aspiration" in this Bible — it is a requirement set with the same status as security or performance.

**REQ-GOV-101 (P0) — Two-person rule for constitutional change.** Any change to N-rules, gate definitions, or §6 requirements requires written sign-off from the founder **and** one other named steering-group member (§21.4). One-person changes to the constitution are forbidden, including by the founder alone.

**REQ-GOV-102 (P0) — Founder-unavailability protocol.** A written delegation exists (§21.4) naming a delegate who may resolve `[PEN-NNN]` items when the founder is unavailable beyond the documented threshold. The delegate may not amend this Bible, DEC items, or N-rules — only resolve pending items to unblock work.

**REQ-GOV-103 (P0) — Decision latency SLA.** Every `[PEN-NNN]` item escalated to the founder receives a decision within a documented SLA. SLA by default is 5 business days for PEN items that block a sprint; 2 business days for PEN items that block a gate. SLA misses are logged in the decision register and reviewed weekly.

**REQ-GOV-104 (P1) — Decision register is git-tracked.** Every DEC, DEF, and PEN item, plus its resolution date, reasoning, and signatory, is committed to `docs/DECISIONS/` in a structured format. No decision exists outside the register.

**REQ-GOV-105 (P1) — Quarterly traceability audit.** Once per quarter, a random sample of 20 commits is audited to confirm each names a live REQ/DEC/DEF/PEN ID that resolves against the current registers. Failures are reported at the next gate.

**REQ-GOV-106 (P1) — Gate sign-off dual-attestation.** Every gate sign-off requires both the founder (or delegate) and the phase lead for that phase. Two-person rule extends to gates.

**REQ-GOV-107 (P2) — Public governance transition plan.** By Gate G10, a written plan exists for transitioning the Intent ABI and capability model to public governance (community RFC process, review board). This satisfies Part 1 P5 "radical openness" beyond code.

### 9.6 REQ-HAL series — portability boundary (new, in response to the QEMU→hardware cliff)

The HAL is the mechanism that converts "Phase 9 is re-implementation, not porting" from a scheduling wish into an architectural fact. Without a HAL, every Phase 9 driver re-opens Phase 4 design decisions.

| REQ ID | Requirement | Applies at |
|---|---|---|
| **REQ-HAL-101 (P0)** | Generic device-model trait defined: enumeration, capability negotiation, IRQ delivery, DMA-safe memory, power states, error propagation. | G4 |
| **REQ-HAL-102 (P0)** | At least two backends: QEMU virt (first) and hanoip (second). Same driver source compiles against both. | G9 |
| **REQ-HAL-103 (P0)** | Backend-specific code isolated behind compile-time feature gates; no `#[cfg(not(...))]` sprawl in driver bodies. | G4 |
| **REQ-HAL-104 (P0)** | Conformance test suite runnable against any backend; failures block backend promotion. | G4 |
| **REQ-HAL-105 (P1)** | HAL interface stability: any breaking change to the trait requires a change request (it affects Phase 9 schedule). | G4 |

### 9.7 REQ-AI-112 — Dynamic Hardware Profile Manager (new, per RFC-INDOS-2026-001)

Dynamic inference configuration daemon in unprivileged userspace (`userspace/ais/`) to eliminate RAM strain on budget hardware (6 GB) while utilizing flagship compute (12 GB+). Scheduled for Phase 6 (Sprint SP-41).

| REQ ID | Requirement | Applies at |
|---|---|---|
| **REQ-AI-112 (P1)** | `IndHardwareProfile` daemon in `userspace/ais/` inspects physical RAM via read-only telemetry (`REQ-KRN-113`) during early initialization. Selects `Q4_K_M` 1.5B (2K context, 128 MB KV-cache, $\le 1250$ MB RAM) on $\le 6$ GB tier, or `Q8_0`/7B/8B (8K context, Vulkan acceleration) on $\ge 12$ GB tier. Zero kernel/ENF modification. | G6 |

### 9.8 REQ-ARCH-101 — Standalone `indk-core` Workspace Decoupling (new, per RFC-INDOS-2026-001)

Freestanding workspace decoupling of kernel primitives from mobile userspace assumptions, enabling `IndK` reusability for robotics, IoT, and edge compute. Scheduled for Phase 4 (Sprint SP-31).

| REQ ID | Requirement | Applies at |
|---|---|---|
| **REQ-ARCH-101 (P1)** | Restructure `/kernel` into an independent Cargo workspace crate `indk-core` (`aarch64-unknown-none`, `no_std`) compiling standalone without mobile userspace (`IndUS`) or AI subsystem (`AIS`) dependencies. Non-disruptive refactor executed in SP-31 HAL buildout. | G4 |

### 9.9 REQ-APP series — Primary Applications Requirements Catalog (new, per INDOS-APPS-PRD-001 v1.0.0.0)

Every application requirement carries a stable ID (`REQ-APP-<tier/app>-<number>`) and capability (`CAP-APP-<app>-<nnn>`). Acceptance requires verification against the Universal App Charter (§8.20) and passing the offline-first test (`INV-APP-003`).

#### 9.9.1 Common App Architecture & Framework (`REQ-APP-COMMON-*`)

- **REQ-APP-COMMON-001 (P0) — Semantic Tree Mandate:** Every screen, dialog, and surface in every app exposes a compositor-verifiable semantic tree (container, interactive, content, navigation, proposal). A missing tree fails gate review. Screen-reader users must be able to complete primary tasks unaided.
- **REQ-APP-COMMON-002 (P1) — Battery Accounting Integration:** Every app registers with the OS battery accounting API. Abnormal draw (>2× declared budget over rolling 24h) triggers an un-suppressible "High battery use" notice in the Privacy Feed.
- **REQ-APP-COMMON-003 (P1) — Local Crash Reporting:** On crash, produce a local crash report visible in WhyExplorer. User opt-in required to export/share; zero telemetry leaves device by default. No external crash SDKs.
- **REQ-APP-COMMON-004 (P1) — Signed OTA Lifecycle:** Apps update strictly via signed OS OTA (REQ-USE-205). Independent self-updates and ambient third-party app stores are prohibited in v1.0.
- **REQ-APP-COMMON-005 (P1) — Notification Center Mediation:** Apps post notifications exclusively via Notification Center service (§6.7). Direct screen intrusion is prohibited. Lock screen displays a persistent kill-switch for AIS and ambient proposals.
- **REQ-APP-COMMON-006 (P0) — App Framework Extensions:** Formalized lifecycle hooks (`on_launch`, `on_foreground`, `on_background`, `on_suspend`, `on_terminate`), window management, back-stack navigation, and standard chrome library.
- **REQ-APP-COMMON-007 (P1) — Cross-App Share Sheet:** Standard OS share target mediation (Messages, Files, Notes, Clipboard, Gallery, Browser, Music, Recorder). Every share operation is explicitly user-mediated; standing share capabilities are banned.
- **REQ-APP-COMMON-008 (P1) — Standard Intent Registry:** Apps register typed intents (`ind.intent.VIEW`, `ind.intent.SHARE`, `ind.intent.CALL`, etc.) routed via ENF capability mediation.
- **REQ-APP-COMMON-009 (P1) — Deferred App Fallback:** Documented browser and web-app fallbacks for deferred services (`docs/apps/deferred-apps.md`).
- **REQ-APP-COMMON-010 (P1) — Global Search Surface:** System-wide search indexed across installed apps, contacts, recent files, and settings. Content of messages, photos, and browser history are excluded by default (opt-in only).
- **REQ-APP-COMMON-011 (P1) — Clipboard Store & Hygiene:** Encrypted local clipboard store with 60-second sensitive content (OTP/passwords) timeout (`DEF-APP-006`).
- **REQ-APP-COMMON-012 (P1) — Capability-Mediated Clipboard Read:** Inter-app clipboard reading requires per-read user consent. Ambient clipboard snooping is physically blocked.
- **REQ-APP-COMMON-013 (P0) — Notification Center System Surface:** Top edge-swipe pull-down presenting unified notifications and persistent consent receipts.
- **REQ-APP-COMMON-014 (P0) — Quick Settings Surface:** Secondary top pull-down exposing hardware toggles (Airplane, Wi-Fi, Torch, Battery, AIS Mode, Privacy Feed shortcut).
- **REQ-APP-COMMON-015 (P0) — Lock Screen Surface:** Hardware-isolated lock surface with clock, emergency dialer shortcut, media controls, and consent card queue.
- **REQ-APP-COMMON-016 (P0) — Recents / Multitasking Carousel:** Bottom edge-swipe-up presenting interactive card carousel of active app states with swipe-to-terminate.
- **REQ-APP-COMMON-A11Y-001 (P0) — Real-User Accessibility Validation:** No app ships without passing verified task completion with real screen-reader and switch-access users recorded in its evidence pack.
- **REQ-APP-COMMON-I18N-001 (P0) — Indic Script & RTL Support:** Native typography and layout support for Hindi (Devanagari), Hinglish, English, and RTL scripts from day one.

#### 9.9.2 Tier 1 & Tier 2 Native Applications

##### Phone & Emergency (`REQ-APP-PHONE-*`)
- **REQ-APP-PHONE-001 (P0) — Voice Calls:** Cellular voice call origination and termination via Phase 9 modem HAL (`CAP-APP-PHONE-CALLS`).
- **REQ-APP-PHONE-002 (P0) — Emergency Dialer:** Emergency calling (`112`, `911`, `100`, `101`, `102`) operational from lock screen, during call, or device suspension without unlock or capability negotiation.
- **REQ-APP-PHONE-003 (P0) — Local Call History:** Chronological log with direction, duration, timestamp. Local-only; zero cloud sync.
- **REQ-APP-PHONE-004 (P0) — Hardware-Isolated In-Call UI:** In-call controls (mute, speaker, keypad, hold, end) rendered on `PlaneId::HardwareIsolated` to prevent overlay tapjacking.
- **REQ-APP-PHONE-005 (P1) — Voicemail:** Carrier standard call-in voicemail; visual voicemail deferred to v1.0.1 (carrier-dependent).
- **REQ-APP-PHONE-006 (P1) — Dual-SIM Management:** Per-call SIM selection on dual-SIM hardware; seamless single-SIM fallback.
- **REQ-APP-PHONE-007 (P1) — Favorites & Speed Dial:** One-tap rapid dialing for primary contacts.
- **REQ-APP-PHONE-008 (P1) — Suraksha Scam-Call Shield:** On-device classifier evaluates incoming numbers/metadata against offline threat patterns; displays warnings and one-tap "block & report".
- **REQ-APP-PHONE-009 (P1) — AIS Voice Intent Integration:** Voice intent "call Mom" routes to proposal card with explicit confirmation; silent dialing strictly prohibited.
- **REQ-APP-PHONE-010 (P1) — Offline Operation:** Full functionality (emergency, history, speed dial) operational without data connection.

##### Messages & SMS (`REQ-APP-MSG-*`)
- **REQ-APP-MSG-001 (P0) — SMS Send & Receive:** Standard cellular SMS compose, send, receive, and threaded conversation views.
- **REQ-APP-MSG-002 (P0) — Delivery Reliability:** On-device retry queue for transient cellular drops; carrier delivery receipt logging.
- **REQ-APP-MSG-003 (P0) — Threaded Conversations:** Address-grouped message threads with unread indicators and draft persistence.
- **REQ-APP-MSG-004 (P0) — User-Mediated Attachments:** Attachment selection strictly via OS share sheet or Gallery picker; ambient file reading banned.
- **REQ-APP-MSG-005 (P0) — OTP Interception & Screen-Share Blackout:** Banking OTP SMS intercepted by Suraksha Shield; forwarded only via explicit consent; blacked out during screen-share.
- **REQ-APP-MSG-006 (P0) — Scam SMS Shield:** Local on-device NLP classifier flags fraudulent phishing/digital-arrest SMS with high-contrast warning cards.
- **REQ-APP-MSG-007 (P1) — RCS Protocol:** Deferred to Phase 12 pending carrier-universal stack on hanoip (PEN-APP-009).
- **REQ-APP-MSG-008 (P1) — Local Message Search:** Local indexing of conversation text; banking OTPs and passwords excluded from index.
- **REQ-APP-MSG-009 (P1) — AIS Proposal Routing:** Voice intent creates draft proposal card; user must review and tap send. Silent sending prohibited.
- **REQ-APP-MSG-010 (P1) — MMS & Group Messaging:** MMS multimedia transport and SMS group conversation threading.
- **REQ-APP-MSG-011 (P1) — Device Encryption at Rest:** Complete message database encrypted with device-derived key.

##### Contacts (`REQ-APP-CT-*`)
- **REQ-APP-CT-001 (P0) — Contact CRUD:** Full management of names, numbers, emails, postal addresses, notes, and local photo links.
- **REQ-APP-CT-002 (P0) — VCF Import/Export:** Import and export standard `.vcf` files via FilesApp. Zero cloud dependency.
- **REQ-APP-CT-003 (P0) — Capability Gating:** Reading contacts requires explicit `CAP-APP-*-CONTACTS-READ` grant; contacts database owned by Contacts service.
- **REQ-APP-CT-004 (P1) — Local Caller ID:** Real-time caller identification from local database; unknown numbers prompt Suraksha lookup.
- **REQ-APP-CT-005 (P1) — Fast Local Search:** Sub-32ms local search indexed by name, phonetic transliteration, and number.
- **REQ-APP-CT-006 (P1) — AIS Consent Gate:** AIS queries require per-query user confirmation before reading contact details.
- **REQ-APP-CT-007 (P1) — Contact Groups:** User-defined groups for batched messaging.
- **REQ-APP-CT-008 (P1) — Cryptographic Deletion:** Contact deletion cryptographically shreds local record; no cloud remnants.

##### Camera (`REQ-APP-CAM-*`)
- **REQ-APP-CAM-001 (P0) — Still Image Capture:** High-resolution JPEG/PNG capture from front and rear camera sensors.
- **REQ-APP-CAM-002 (P0) — Video Recording:** H.264/MP4 hardware-accelerated video recording with audio synchronization.
- **REQ-APP-CAM-003 (P0) — Zero-Lag Gallery Ingestion:** Captured media directly written to Gallery storage pipeline upon shutter completion.
- **REQ-APP-CAM-004 (P0) — Hardware Indicator & Revocation:** Mandatory hardware/status bar indicator during active sensor use; capability revoked immediately upon backgrounding.
- **REQ-APP-CAM-005 (P1) — Essential Modes:** Photo, Video, Night, and Portrait (if depth hardware present). Beauty filters and AR lenses prohibited.
- **REQ-APP-CAM-006 (P1) — Sensor Controls:** Manual resolution, HDR, composition grid, timer, and opt-in EXIF geotagging (default: GPS stripped).
- **REQ-APP-CAM-007 (P1) — On-Device Computer Vision:** Opt-in local OCR and barcode scanning running purely on-device.
- **REQ-APP-CAM-008 (P1) — AIS Camera Intent:** Voice intent launches camera in ready state; shutter trigger requires confirmed user action.
- **REQ-APP-CAM-009 (P1) — Barcode / QR Scanner:** Offline decoding for URLs and Wi-Fi configurations presented via inspection card.

##### Gallery (`REQ-APP-GAL-*`)
- **REQ-APP-GAL-001 (P0) — Media Browser:** Chronological timeline grid, folder groupings, and high-performance filmstrip view.
- **REQ-APP-GAL-002 (P0) — Native Video Player:** Hardware-decoded playback with seek, scrub, and volume controls.
- **REQ-APP-GAL-003 (P0) — Deterministic Editor:** Crop, rotate, and basic color balancing. Save as copy by default; no destructive overwrites.
- **REQ-APP-GAL-004 (P0) — 30-Day Local Trash:** Local soft-delete retention for 30 days before irreversible key destruction (`DEF-APP-007`).
- **REQ-APP-GAL-005 (P0) — Share Sheet Dispatch:** Media sharing strictly through OS share sheet; direct network upload banned.
- **REQ-APP-GAL-006 (P1) — Local Album Management:** User-created and directory-sourced media albums.
- **REQ-APP-GAL-007 (P1) — On-Device Semantic Search (Opt-in):** Local offline embedding model indexing images by visual concepts ("car", "garden") without cloud leakage.
- **REQ-APP-GAL-008 (P1) — AIS Media Query Gate:** Natural language queries route through AIS only upon explicit per-query user consent.

##### Clock & Alarm (`REQ-APP-CLK-*`)
- **REQ-APP-CLK-001 (P0) — Multi-Alarm Engine:** Configurable recurring alarms, custom audio tones, vibration profiles, and snooze intervals.
- **REQ-APP-CLK-002 (P0) — Hard Real-Time Reliability:** Alarms fire unconditionally across Do-Not-Disturb, battery saver, and AIS restricted modes (`[TESTED]`).
- **REQ-APP-CLK-003 (P0) — Precision Timers:** Concurrent countdown timers with persistent background state.
- **REQ-APP-CLK-004 (P0) — Stopwatch:** Millisecond precision lap timing and pause/resume capability.
- **REQ-APP-CLK-005 (P0) — World Clock:** Multi-timezone display with UTC offset mapping.
- **REQ-APP-CLK-006 (P0) — Hardware-Isolated Wake Screen:** Wake-up alarm dismiss/snooze rendered on `PlaneId::HardwareIsolated` to prevent malicious overlay dismissal.
- **REQ-APP-CLK-007 (P1) — AIS Voice Alarm Setup:** Voice intent "set alarm for 6am" creates proposal card requiring tap confirmation.
- **REQ-APP-CLK-008 (P1) — Timezone & DST Transition:** Automated daylight saving and timezone transition management.

##### Calculator (`REQ-APP-CALC-*`)
- **REQ-APP-CALC-001 (P0) — Standard Arithmetic:** High-precision basic operations with operator precedence and parenthesis support.
- **REQ-APP-CALC-002 (P0) — Scientific Mode:** Trigonometric, logarithmic, exponential, and power functions.
- **REQ-APP-CALC-003 (P0) — Session History:** 20-calculation rolling history with tappable operand recall.
- **REQ-APP-CALC-004 (P1) — Unit & Currency Conversion:** Offline unit conversion with bundled currency exchange snapshots.
- **REQ-APP-CALC-005 (P1) — AIS Inline Computation:** Natural language math queries evaluated directly into Intent Bar proposal card.

##### Web Browser (`REQ-APP-BRW-*`)
- **REQ-APP-BRW-001 (P0) — Web Standards Engine & Media Pipeline:** HTML5, CSS3, ES2020+ execution via vendored engine (WebKit-class) with dav1d/libvpx ARM64 NEON software decoders, Wayland-to-uxd compositor bridge, and ClearKey/sandboxed Widevine DRM dual-track per PEN-APP-008.
- **REQ-APP-BRW-002 (P0) — Enforced HTTPS-Only:** Rejects unencrypted HTTP by default; certificate errors require non-dismissible manual bypass.
- **REQ-APP-BRW-003 (P0) — Process-Isolated Tabs:** Sandboxed multi-tab session management encrypted at rest.
- **REQ-APP-BRW-004 (P0) — Local Bookmarks & History:** Zero-sync local storage; searchable via Global Search only upon opt-in.
- **REQ-APP-BRW-005 (P0) — Sandboxed File Downloads:** Downloads dispatched directly to FilesApp with explicit destination consent.
- **REQ-APP-BRW-006 (P0) — Site Isolation & Cookie Partitioning:** Strict origin isolation with third-party tracking cookies blocked by default.
- **REQ-APP-BRW-007 (P0) — Anti-Fingerprinting Shield:** Canvas, WebGL, and user-agent randomization with blocked beacon counters.
- **REQ-APP-BRW-008 (P0) — True Private Browsing:** Ephemeral session partition leaving zero disk artifacts or cache remanence.
- **REQ-APP-BRW-009 (P1) — Integrated Password Manager:** Autofill mediated through OS-encrypted credential store.
- **REQ-APP-BRW-010 (P1) — Distraction-Free Reader Mode:** Deterministic article extraction for accessibility and readability.
- **REQ-APP-BRW-011 (P1) — AIS Read-Only Summarization:** Natural language "summarize this page" executes strictly on user command via read-only GBNF grammar parser.
- **REQ-APP-BRW-012 (P1) — Standalone Web Apps (PWA):** Installation of web applications to Bento Grid as sandboxed first-class entities.

##### Calendar (`REQ-APP-CAL-*`)
- **REQ-APP-CAL-001 (P0) — Local Event Storage:** Event scheduling with recurrence rules, location metadata, and local alarm triggers.
- **REQ-APP-CAL-002 (P0) — Multi-Scale Views:** Month, week, day, and agenda views with smooth touch scrolling.
- **REQ-APP-CAL-003 (P0) — Reliable Event Reminders:** Local notifications scheduled via system alarm daemon.
- **REQ-APP-CAL-004 (P0) — iCalendar (.ics) Support:** Local import and export of RFC 5545 `.ics` event files via FilesApp.
- **REQ-APP-CAL-005 (P1) — User-Mediated Event Sharing:** Sharing calendar invites via Messages or FilesApp.
- **REQ-APP-CAL-006 (P1) — AIS Scheduling Intent:** Voice intent "schedule meeting with Priya tomorrow at 3pm" produces structured proposal card.
- **REQ-APP-CAL-007 (P1) — Privacy-Preserving Cloud Sync:** Deferred to Phase 12; user-opt-in, provider-scoped, zero-PII cloud sync (`PEN-APP-013`).

##### Music & Audio (`REQ-APP-MUS-*`)
- **REQ-APP-MUS-001 (P0) — Local Codec Playback:** Bit-perfect playback of MP3, FLAC, OGG, and WAV from user storage (`DEF-APP-008`).
- **REQ-APP-MUS-002 (P0) — Isolated Media Controls:** Playback widget in Notification Center and lock screen on isolated plane.
- **REQ-APP-MUS-003 (P0) — Background Playback Service:** Audio output continues under screen-off state via unprivileged audio daemon.
- **REQ-APP-MUS-004 (P0) — Audio Focus Arbitration:** Strict priority pre-emption: Alarms and Phone calls preempt Music playback.
- **REQ-APP-MUS-005 (P1) — Metadata Parsing & Tagging:** ID3/Vorbis comment parser and embedded artwork extractor.
- **REQ-APP-MUS-006 (P1) — Library Search & Playlists:** Local query indexing across tracks, artists, and albums.
- **REQ-APP-MUS-007 (P1) — AIS Voice Playback:** Voice intent generates proposal card for requested playlist or artist.

##### Weather (`REQ-APP-WTH-*`)
- **REQ-APP-WTH-001 (P0) — Meteorological Conditions:** Current temperature, humidity, wind velocity, precipitation, and solar times.
- **REQ-APP-WTH-002 (P0) — Multi-Day Forecasting:** 24-hour hourly and 7-day extended forecasts.
- **REQ-APP-WTH-003 (P0) — Offline-First Presentation:** Cached data shown with explicit "Last updated" timestamp; fallback to coarse model if no network.
- **REQ-APP-WTH-004 (P1) — On-Demand Manual Refresh:** User-initiated pull-to-refresh; background polling limited to Wi-Fi charging (`DEF-APP-005`).
- **REQ-APP-WTH-005 (P1) — Multi-City Profiles:** Saved location profiles with designated primary home widget.
- **REQ-APP-WTH-006 (P1) — Coarse Location Consent:** Per-fetch or manual city selection to preserve location anonymity.
- **REQ-APP-WTH-007 (P1) — AIS Weather Queries:** Voice queries answered from cached weather state without triggering live network egress.

##### Voice Recorder (`REQ-APP-REC-*`)
- **REQ-APP-REC-001 (P0) — Bit-Perfect Audio Capture:** Microphone capture to WAV, FLAC, or OGG with pause/resume support.
- **REQ-APP-REC-002 (P0) — Mandatory Hardware Indicator:** Persistent red recording indicator in status bar; silent recording structurally impossible (`CAP-APP-REC-MIC`).
- **REQ-APP-REC-003 (P0) — Local File Management:** Timestamped recording nomenclature stored in isolated sandbox directory.
- **REQ-APP-REC-004 (P1) — Real-Time Waveform:** Hardware-synced audio visualizer and interactive scrubber.
- **REQ-APP-REC-005 (P1) — Local Offline Transcription:** On-device ASR model transcribes voice notes locally without cloud egress.
- **REQ-APP-REC-006 (P1) — AIS Intent Control:** Voice start/stop intents require explicit visual confirmation.
- **REQ-APP-REC-007 (P1) — Jurisdiction Consent Notice:** First-use legal recording compliance notice acknowledged by user (`PEN-APP-014`).

#### 9.9.3 App-Class Performance Budgets (`REQ-PERF-401..415`)

Measured on reference hardware (Moto G60 / SM7150) under production build flags (`--release`). Enforced in CI automated regression suites:

| REQ ID | Target Application | Cold-Launch (p95) | Resident Memory (RAM) | Interaction Latency (p95) |
|---|---|---|---|---|
| **REQ-PERF-401** | Phone & Emergency | $\le 200\text{ ms}$ | $\le 24\text{ MB}$ | $\le 32\text{ ms}$ |
| **REQ-PERF-402** | Messages & SMS | $\le 250\text{ ms}$ | $\le 32\text{ MB}$ | $\le 32\text{ ms}$ |
| **REQ-PERF-403** | Contacts | $\le 200\text{ ms}$ | $\le 16\text{ MB}$ | $\le 32\text{ ms}$ |
| **REQ-PERF-404** | Camera | $\le 500\text{ ms}$ | $\le 64\text{ MB}$ | $\le 100\text{ ms}$ |
| **REQ-PERF-405** | Gallery | $\le 300\text{ ms}$ | $\le 48\text{ MB}$ | $\le 32\text{ ms}$ |
| **REQ-PERF-406** | Clock & Alarm | $\le 150\text{ ms}$ | $\le 12\text{ MB}$ | $\le 16\text{ ms}$ |
| **REQ-PERF-407** | Calculator | $\le 100\text{ ms}$ | $\le 8\text{ MB}$ | $\le 16\text{ ms}$ |
| **REQ-PERF-408** | Web Browser | $\le 400\text{ ms}$ (first paint) | $\le 96\text{ MB}$ | $\le 32\text{ ms}$ |
| **REQ-PERF-409** | Calendar | $\le 250\text{ ms}$ | $\le 24\text{ MB}$ | $\le 32\text{ ms}$ |
| **REQ-PERF-410** | Music & Audio | $\le 200\text{ ms}$ | $\le 24\text{ MB}$ | $\le 16\text{ ms}$ |
| **REQ-PERF-411** | Weather | $\le 150\text{ ms}$ | $\le 16\text{ MB}$ | $\le 16\text{ ms}$ |
| **REQ-PERF-412** | Voice Recorder | $\le 200\text{ ms}$ | $\le 24\text{ MB}$ | $\le 32\text{ ms}$ |
| **REQ-PERF-413** | Notes App | $\le 150\text{ ms}$ | $\le 16\text{ MB}$ | $\le 16\text{ ms}$ |
| **REQ-PERF-414** | Files App | $\le 200\text{ ms}$ | $\le 20\text{ MB}$ | $\le 32\text{ ms}$ |
| **REQ-PERF-415** | System Shell & Surfaces | $\le 100\text{ ms}$ | $\le 32\text{ MB}$ | $\le 8.33\text{ ms}$ (120Hz) |

### 9.10 Requirement lifecycle

- **Adding a requirement** requires a change request (§21), MINOR version bump, and — if it blocks a gate — a phase-plan update in Part 2.
- **Changing a requirement** requires the same, with a documented reason. Silent requirement changes are grounds for rollback of the offending sprint (Rule N-13).
- **Retiring a requirement** requires evidence that it has been superseded; the retirement is logged in the changelog (§27.7).
- **REQ IDs are never reused.** If REQ-KRN-104 is retired, REQ-KRN-104 stays retired; a replacement gets a new number.

---

## 10. Repository and engineering conventions

### 10.1 Monorepo layout

Canonical layout is defined in §27.1. Every top-level directory has a `README.md` explaining its purpose, its ownership (CODEOWNERS entry), and its build entry point. Directories without these are non-conformant.

### 10.2 Commit protocol

- **Format:** `<type>(<scope>): <subject>` — conventional-commits style. Types: `feat`, `fix`, `docs`, `spec`, `test`, `refactor`, `perf`, `chore`, `build`, `ci`.
- **Requirement tagging:** every commit body includes at least one of: `REQ-XXX`, `DEC-NNN`, `DEF-NNN`, `PEN-NNN`, `N-NN`. CI rejects commits that name none.
- **Signed-off-by:** every commit is signed-off (`git commit --signoff`), indicating DCO compliance.
- **One logical change per commit.** Mixed commits (spec + code + doc) are permitted only when the change is genuinely indivisible (e.g., spec and its test vectors).
- **No force-push to `main`.** Ever. Feature branches are rebased; `main` is merge-only.

### 10.3 Branching model

- `main` — always green, always passing all gates that have been passed.
- `sprint/sp-XXX` — one branch per sprint, cut from `main`, merged at sprint validation PASS.
- `hotfix/*` — for security fixes; require N-14 two-person approval if they touch TCB.
- `phase/kN` — long-lived branches for phase gates (used when a phase has destabilizing work that must not enter `main` until the gate).
- **Feature branches are forbidden.** Work happens on the sprint branch, period. This prevents "shadow work" that skips validation.

### 10.4 Code review

- **Every merge to `main` requires ≥ 1 review** from a different engineer or agent.
- **TCB changes require 2 reviews**, one of which must be a named TCB-owner (a designated senior engineer per subsystem).
- **Spec changes require the spec owner's review.** Specs are contracts; spec owners are named in `docs/architecture/OWNERS.md`.
- **AI agents reviewing AI-generated code:** permitted, but the agent reviewer's output must be visibly signed and cannot substitute for a human reviewer on TCB changes.

### 10.5 Agent-generated code policy

- **All agent-generated code is subject to the same rules as human code** — same tests, same reviews, same commit protocol, same evidence pack.
- **Agents cannot approve their own PRs.**
- **Agents cannot push directly to `main`,** ever, even for "trivial" changes.
- **Agents that violate N-rules produce an incident report** — the incident is attributed to the human supervisor of that agent run, not "to the AI."
- **Agent prompts are versioned** in `tools/agent/prompts/`. Prompt changes that change behaviour of AI-generated code require review.

---

## 11. Threat model and explicit non-guarantees

### 11.1 Attacker classes (in scope)

| Class | Capability | Defence |
|---|---|---|
| **Malicious app (sandboxed)** | Runs with its manifest-declared capabilities | Capability isolation + consent broker + audit |
| **Malicious app (privileged exploit)** | Achieves kernel memory corruption | Capability model limits blast radius; TCB small; formal verification target |
| **Prompt-injection attacker** | Controls content the AI reads (SMS, screen text, notifications) | Taint tracking + dual-context isolation (Part 1 §6.3) + policy engine below the AI |
| **Network attacker** | MITM on update, DNS, TLS | Signed updates (DEF-003), vetted TLS (DEF-004), pinned certs where applicable |
| **Compromised AI runtime** | Achieves code execution in the AI process | TEE isolation (Part 1 §6.3); capability model limits what AI process can do regardless |
| **Compromised kernel** | Achieves kernel privilege | TEE-protected AI memory (where hardware supports); measured boot detects | 
| **Physical attacker (device stolen)** | Device in hand, unlocked or locked | Encryption at rest (REQ-SEC-107) where hardware supports; user PIN/biometric |
| **Supply-chain attacker (model)** | Malicious model update | Signed models (REQ-AI-101, REQ-AI-109), capability-inheritance rule |
| **Supply-chain attacker (code)** | Malicious dependency | SBOM + pinned versions + reproducible builds (REQ-QA-201, REQ-QA-202) |

### 11.2 Trust boundaries

- **TCB boundary:** kernel core, capability system, policy engine, consent broker, audit. Anything outside is untrusted.
- **AIS boundary:** AIS runs as a user process with capabilities; it is not trusted with any authority it did not receive from ENF.
- **TEE boundary:** where hardware supports it, AI weights and prompts live in TEE-protected memory; the rich OS cannot read them.
- **Consent boundary:** the consent surface is rendered on a non-compositable plane; no app or AIS surface can draw over it or inject input.

### 11.3 Attack surface inventory (maintained as a living document)

Living document at `docs/architecture/attack-surface.md`, updated every sprint that adds a syscall, capability type, or service entry point. Reviewers check that new surface is either (a) behind capability mediation or (b) explicitly accepted with rationale.

### 11.4 Explicit non-guarantees

These are stated so no external claim can drift into implying otherwise:

- **No defence against a physically-compromised hardware supply chain.** A tampered chip can be a backdoor.
- **No defence against a fully-compromised vendor firmware** (modem, PMIC where firmware is closed). The blob policy (§14.3) is about honest labelling, not about trusting blobs.
- **No defence against the user deliberately installing a capability-granting malicious app.** The model is: consent-based capability grants + audit, not prevention.
- **No claim that the TEE is unbreakable.** TEEs have been broken in the past. The claim is: a normal-world kernel compromise cannot read AI memory (per Sprint 27 attack suite standard).
- **No claim that "zero ambient authority" means "zero privilege."** The kernel has privilege. The claim is that the kernel's privilege is bounded and formally verified where possible, and that user processes (including AIS) have no ambient authority.

---

## 12. AIS charter (binding on AIS implementation)

### 12.1 Positive scope — what AIS may do

- Propose actions to ENF via typed intents.
- Read its own memory store (scoped).
- Read kernel telemetry (read-only, per REQ-KRN-113).
- Produce explanations for user queries about past actions (per REQ-AI-104).
- Suggest proactive actions ranked by user-controlled aggressiveness.
- Flag anomalies via trusted-UI alerts.

### 12.2 Negative scope — what AIS may never do (the §6.4 non-uses, formalized)

1. **Boot-chain participation.** AIS is not loaded, called, or consulted before the kernel reaches a documented "AIS-ready" state.
2. **Modify its own enforcement-relevant code or configuration.** AIS cannot write to the policy engine, the capability table, the consent broker configuration, or the audit log.
3. **Hold signing keys.** AIS never has access to update signing keys, model signing keys, or user data encryption keys.
4. **Make irreversible decisions.** AIS cannot directly perform actions that cannot be undone (factory reset, permanent deletion, payment finalization, contact-block-forever). Such actions must pass through consent and (where applicable) human-only gates.
5. **Replace the consent broker.** AIS cannot draw the consent surface, intercept input on it, or produce what appears to be a consent prompt.
6. **Communicate externally without a capability.** No raw network. Even read-only network (e.g., fetching a URL for the user) is mediated by a separate network service the user has consented to.
7. **Produce deceptive UI.** AIS-suggested content is visually and structurally distinct from system facts.
8. **Train on user data without consent.** Any AIS learning from user behaviour requires explicit consent and is revocable.

### 12.3 AIS kill switch

A dedicated, always-available hardware or gesture path exists to fully disable AIS without a reboot. After activation, AIS produces no suggestions, reads no memory, emits no proposals, and stores nothing new. Disabling is logged; re-enabling requires explicit user action. Requirement: **AIS kill switch usable in ≤ 2 gestures from the lock screen.**

### 12.4 AIS review cadence

Every gate from G6 onward re-audits AIS against this charter. Every N-rule violation involving AIS triggers an incident report and a charter review. AIS charter changes require N-14 two-person sign-off.

---

## 13. Data handling, privacy, and consent

### 13.1 Data classification

- **Public** — user's own files meant for sharing; no special handling.
- **Private** — user's non-sensitive content (notes, calendar, contacts). Encrypted at rest per REQ-SEC-107.
- **Sensitive** — banking data, OTPs, medical, location history, voice biometrics. Encrypted at rest, never in AIS-accessible memory without a capability, never logged in plaintext.
- **Secret** — keys (user encryption keys, device keys, signing keys). Never in AIS address space, never logged, TEE-protected where hardware supports.

Data that is *unclassified* by this taxonomy is out of compliance and blocks gate pass.

### 13.2 Data residency

- **On-device by default.** All user data lives on-device unless the user explicitly invokes a feature that requires off-device processing.
- **Off-device processing requires a capability** granted by the user's explicit consent, per-action.
- **Cloud bridge (if built)** is opt-in, isolated, PII-scrubbed by a local-only pre-processor, and the cloud provider is user-chosen (bring-your-own-key pattern). No default cloud.

### 13.3 Retention policy

- **User data:** retained until the user deletes it. Deletion is immediate and provably cryptographic (key-destruction per REQ-AI-103 memory wipe standard).
- **Logs:** system logs retained ≥ 30 days for diagnostic purposes; user can export or delete at any time.
- **Audit log:** immutable; retained for the device's lifetime; user can export for external auditors (privacy through transparency, not secrecy).
- **Crash dumps:** opt-in; if disabled, no crash data leaves the device.

### 13.4 Right to delete

Every data store must expose a delete API. Delete = key destruction where encryption is used, physical zeroing where not. Verification: after delete, no reachable memory or disk region contains the deleted data. This is tested per sprint that touches a data store; failures block merge.

### 13.5 Transparency log

From G5 onward, every signed release (OS, model, update) publishes a transparency log entry: artifact hash, signer identity, timestamp, changelog. The log is append-only, hash-chained, and independently verifiable. Any user can verify their running system against the log from a second device. This is the technical basis for the "user-verifiable integrity" positioning — no gatekeeper. (Requirement: **REQ-SEC-111 new — transparency-log integration**, P1, applies at G5.)

---

## 14. Firmware and hardware enablement strategy

### 14.1 Platform priority

1. **QEMU virt ARM64** — primary dev target for ~60% of the programme (Phases 0–8).
2. **Reference ARM64 dev board** — per PEN-010 resolution; strongly recommended before Moto G60 work (Phase 9). Provides a hardware sanity check independent of phone-specific complexity.
3. **Moto G60 (`hanoip`)** — Tier-0 milestone device; Phase 9.

### 14.2 The QEMU→hardware cliff (restated from Part 2 §8.5 for §14 context)

QEMU virtio drivers are not the same as Qualcomm drivers. Every Phase 9 driver is re-implemented against the same HAL interface, not compiled. Phase 9's 40 sprints reflect this reality. The HAL (§9.6) is the mitigation that keeps driver redesign out of Phase 9.

### 14.3 Firmware blobs policy (matches Part 1 N-01 reference to §14.3)

**What is allowed:**

- Loading closed firmware binaries (modem, Wi-Fi, PMIC, some DSP) **as opaque data** by our driver. The blob runs on the DSP/co-processor; our driver does not link against it.
- Including blobs in the SBOM with provenance, hash, license, and redistribution status.
- Using blobs during development on hardware we own.

**What is not allowed:**

- Any blob in the TCB. The TCB is what we compile from our source; blobs are inputs to non-TCB processes.
- Redistribution of blobs beyond the licensing terms — licensing review (per REQ-LIC-201, §15.5) determines whether a blob may ship in a public build.
- Making security claims that assume a blob is trustworthy. The threat model (§11.4) explicitly excludes defence against compromised vendor firmware.
- **Any blob in the boot chain** that has not been through legal review. Modem firmware that cannot lawfully ship blocks the "cellular" feature (not the boot).
- Silent blob updates. Every blob update is an SBOM change and a build change; CI catches them.

**Rationale:** closed firmware is a hardware reality we cannot eliminate without a decade of reverse engineering. The policy is: use it, track it, isolate it, be honest about it. Do not pretend the OS is open-source end-to-end when the modem firmware is not.

### 14.4 Hardware matrix expectation

Per gate G9, the matrix has a row per component (SoC, storage, display, touch, USB, Wi-Fi, BT, audio, sensors, modem, camera, GPU, PMIC, thermal). Each row is classified:

- **Works** — meets its functional test with `[TESTED]` label + evidence.
- **Partial** — works with documented limitations (e.g., "Wi-Fi works at 2.4GHz only").
- **Blocked** — cannot work due to hardware/legal/technical constraint, with reason.
- **Out of scope** — explicitly excluded with rationale.

No component may be "probably works" at G9. Ambiguity is out of process.

### 14.5 Boot chain strategy and fallbacks

Qualcomm secure boot may reject an unsigned or self-signed kernel image. This is the single largest unknown for Phase 9. Approach:

1. **Reconnaissance (SP-71–SP-72):** document what the bootloader accepts.
2. **PEN-005 decision (founder):** if unsigned boot is blocked, options are (a) dev-board-first, (b) documented partial chain, (c) abandon `hanoip` as target. Founder decides; work does not improvise.
3. **No workarounds without sign-off.** Unlocking methods, exploit tools, or "trust me it works" paths that bypass the honest investigation are Rule N-11 violations.

---

## 15. Licensing, SBOM, and third-party dependency policy

### 15.1 Inbound licensing

- **Permitted inbound:** Apache 2.0, MIT, BSD-2/3, ISC, MPL-2.0, GPLv2-compatible (for kernel), LGPL (with care), CC-BY for docs.
- **Forbidden inbound:** AGPL (unless a specific founder-approved exception), SSPL, non-commercial licenses, "source-available" licenses that restrict use, licenses with field-of-use restrictions.
- **Review process:** any new dependency triggers a licensing review (recorded in SBOM) before merge. `docs/engineering/licensing-and-sbom.md` is the register.

### 15.2 Outbound licensing

- **Userspace + AIS + Apps:** Apache 2.0.
- **Kernel (IndK):** GPLv2-compatible, chosen specifically for Linux-kernel-driver compatibility should any ever be consulted (per N-01, code is not copied — this is about license compatibility).
- **Documentation and specs:** CC-BY-4.0.
- **The Intent ABI (eventually):** permissive (Apache 2.0 or CC0) so third parties can implement it without friction.
- **No license change** without N-14 two-person approval.

### 15.3 SBOM maintenance

- **Format:** CycloneDX or SPDX — one, not both, chosen at SP-01.
- **Cadence:** regenerated on every release; updated on every dependency change.
- **Completeness:** every dependency (direct + transitive + firmware blob + model artifact) appears. Missing entries are CI failures.
- **Verification:** before each gate, a random 10% of the SBOM is spot-checked against actual build inputs.

### 15.4 Third-party code criteria (for anything outside the TCB)

A dependency may be used if ALL of:

- Actively maintained (commit in the last 12 months, or explicitly archived-with-plan).
- Security history reviewed (known CVEs listed in SBOM).
- License permitted (§15.1).
- SBOM-tracked (§15.3).
- Not in the TCB (TCB code is ours; §15.5 covers the sole exception).

### 15.5 Firmware/blob licensing review

Every firmware blob triggers a written legal review before it may be (a) used in a build, (b) shipped publicly. The review is stored in `docs/engineering/blob-licenses/`. A blob that cannot lawfully ship publicly blocks the corresponding feature from shipping (it does not block development).

---

## 16. Performance and usability budget framework

### 16.1 Baseline definition (established at G4, per REQ-PERF framework in §9.4)

Baseline measurements at G4 include: cold-boot-to-shell, IPC round-trip, syscall latency, context-switch cost, FS throughput, memory footprint at idle, app cold-launch time, compositor frame time p95, and (from SP-49) intent-to-card latency. All are recorded in the G4 evidence pack with methodology documented.

### 16.2 Target classes (filled at G4 and G9 with numbers)

- **QEMU target class:** tighter numbers, since QEMU is a deterministic dev platform.
- **Device target class:** looser numbers, since hardware is thermal- and power-limited.
- **Per-metric targets** set at G4 (QEMU) and G9 (device), by the founder, from baselines.

### 16.3 Regression gates in CI

Every merge to `main` runs a micro-benchmark subset. If any metric regresses > 5% against the current baseline, the merge is blocked unless the PR includes an accepted rationale (performance trade-offs are allowed with written justification, never silently).

### 16.4 Measurement methodology

- **Hardware:** QEMU version pinned; host CPU class documented; measurements taken on identical host for regression comparisons.
- **Thermal:** device measurements at thermal steady-state after 5-minute warmup.
- **Battery:** measured with the harness from SP-24, method documented.
- **Repeatability:** every published number reports mean, p50, p95, p99, and N. A number without these is `[HYPOTHESIS]`, not `[TESTED]`.

---

## 17. Evidence pack and validation specification

### 17.1 Evidence pack contents

Per gate, per §27.3. Each sprint also produces a mini-pack for sprint validation.

### 17.2 Sprint validation procedure (restated from Part 2 §8.4 for §17 cross-reference)

- Live demo or evidence-pack inspection of every exit-checklist item.
- REQ-ID traceability check (CI + random 20-commit quarterly audit).
- Risk-register delta review.
- Gate-distance report.
- PASS/FAIL verdict.

### 17.3 Reproducibility requirements

Per REQ-QA-201/202 (§9.3). Gate evidence pack must include the exact command sequence an independent reproducer runs, and the successful run log from that reproducer.

### 17.4 Traceability

Every artifact in the evidence pack names its REQ IDs. Every test log names its evidence-pack entry. Every commit in the gate's sprint range names a REQ/DEC/DEF/PEN ID. The chain of custody from requirement to artifact is verifiable end-to-end.

### 17.5 Tamper evidence

Evidence pack contains a hash manifest (`manifest.json`) covering every artifact. The manifest is signed (per DEF-003 threshold scheme from G7 onward; GPG or equivalent for earlier gates). Any tamper invalidates the pack.

---

## 18. Release, versioning, and long-term maintenance

### 18.1 Versioning for the OS

- **Format:** `MAJOR.MINOR.PATCH-CODENAME`, e.g. `0.1.0-qemu` (Developer OS), `0.5.0-hanoip` (device milestone), `1.0.0-monsoon` (v1.0).
- **MAJOR:** incompatible changes to syscall ABI, Intent ABI, or capability model.
- **MINOR:** new features, new supported devices.
- **PATCH:** bugfixes, security fixes, no feature changes.
- **Codename:** one per major release, chosen by founder at the release gate.

### 18.2 Release channels

- **`nightly`** — from `main`, no stability promise.
- **`dev`** — from phase branches at phase-gate passes.
- **`beta`** — post-G10 initial; gated cohort.
- **`stable`** — post-G11; broad availability.

Users opt in per channel. Downgrade permitted between channels with data-compatibility checks.

### 18.3 Patch SLAs (defined at G10 with evidence)

- **Critical security patches:** target 30 days from public disclosure for `stable`; shorter for actively-exploited CVEs.
- **High:** 90 days.
- **Medium:** next scheduled release.
- **Low:** at discretion, documented.

SLAs are targets, not promises. Misses are published to the transparency log.

### 18.4 Support windows

- **Stable** releases receive patches for ≥ 3 years from release.
- **Beta** releases receive patches for ≥ 6 months.
- **Dev** releases receive patches only while that phase is active.
- **Device support** per device, published at gate-pass for that device.

### 18.5 Deprecation

Deprecation of any interface requires: (a) ≥ 2 minor versions of overlap, (b) migration path documented, (c) transparency-log entry, (d) user-facing notice where relevant.

---

## 19. Consolidated gates table (G0–G12, including G9.5)

### 19.1 Gate discipline (restated)

- **Pass/fail only.** No partial pass. A gate either produces a complete evidence pack meeting all exit criteria, or it does not pass.
- **Evidence-based.** Every gate pass requires a signed evidence pack (§27.3).
- **Two-person sign-off.** Per REQ-GOV-106, founder (or delegate) + phase lead.
- **DEC-001 re-examination at each gate.** The founder's decision to build from-scratch (Part 1 §1.2) is re-examined at every gate. The re-examination is minuted even when the answer is "continue." If any gate shows the from-scratch path cannot deliver the properties it promises, the founder re-opens the Android-foundation option — not "later," but at that gate.

### 19.2 The table

| Gate | After phase | After sprint | Pass criteria (abbreviated — full text in Part 2 §8, matched here) | Fail action | Evidence pack | DEC-001 re-exam |
|---|---|---|---|---|---|---|
| **G0** | 0 | SP-03 | Clean-machine bootstrap; CI green; specs signed; **PEN-013 resolved**; PEN-012 resolved; PRD ratification signed; DEC-001 reasoning file on record | No kernel work begins | `evidence/G0/` | **Required (first)** |
| **G1** | 1 | SP-09 | 20/20 cold boots; build uniquely identified; reproducible from clean checkout; corrupt-image rejected | Fix boot path; no memory work | `evidence/G1/` | Required |
| **G2** | 2 | SP-15 | Two+ isolated processes preempt under stress; address-space separation proven; syscall ABI frozen; unsafe-budget review passed; DEF-001/002 sunset reviews held | Kernel core redesign, not patch-on-patch | `evidence/G2/` | Required |
| **G3** | 3 | SP-23 | Capability invariants proven by test + machine-checkable specs; **no ambient authority (grep + runtime)**; IPC latency within budget; **consent broker un-spoofable**; audit tamper test passes | **HARD STOP.** Whole security architecture re-reviewed by founder. Phase 4 does not begin on a broken spine. | `evidence/G3/` | Required |
| **G4** | 4 | SP-31 | User-mode drivers serve storage/input/display; FS survives power-cut fuzzing; SMP scaling measured; telemetry API read-only proven; **HAL adopted by 100% of reference drivers**; **performance baselines recorded**; DEF-001/002 sunset minuted | Revert drivers to framework; re-prove | `evidence/G4/` | Required |
| **G5** | 5 | SP-40 | Team lives on Ind OS a full working day (dogfood record); 72h soak; consent store tamper-passing; **Developer OS v0.1 image reproduces from clean checkout** (REQ-QA-201); founder demo | Stability backlog before AIS | `evidence/G5/` | Required |
| **G6** | 6 | SP-50 | Live demo of propose/deny/inspect/consent/revoke with explanation + audit; prompt-injection suite passes; memory wipe destroys keys; **REQ-AI-110 rate ≥ 80% on frozen corpus**; AIS-charter addendum signed | AIS frozen; ENF review | `evidence/G6/` | Required |
| **G7** | 7 | SP-59 | Update chain E2E with rollback; firewall default-deny inbound; leak sentinel catches seeded leak; **crypto ADR signed (DEF-004 = vetted external library)**; SBOM refreshed; DEF-003/004 reviews minuted | No network-dependent features | `evidence/G7/` | Required |
| **G8** | 8 | SP-69 | Non-engineer completes demo script unaided; accessibility tasks pass with real users; consent surface un-spoofable; explanation engine answers any visible action; founder product review | UX work blocked at shell | `evidence/G8/` | Required |
| **G9** | 9 | SP-109 | Reproducible cold boots (≥10) of identified build on **device**; recovery proven **after** experiments; hardware matrix published; acceptance pack complete per PRD-001 §9.5; independent reviewer sign-off | PRD-001 stop/revise: device strategy back to founder | `evidence/G9/` | **Required (major)** |
| **G9.5** | 9.5 | SP-115 | Every REQ-PERF target met `[TESTED]` on QEMU + device — OR founder-signed exception with named fix phase; CI perf gates block regressions | Product stays slow; no Phase 10 start | `evidence/G9.5/` | Required |
| **G10** | 10 | SP-125+ | External audit closed; verification proofs in CI; update chain operated for real; beta plan approved; docs validated by independent engineer | No beta, no outward claims | `evidence/G10/` | **Required (major)** |
| **G11** | 11 | SP-170 | All 12 Tier 1 & Tier 2 apps pass acceptance tests; Universal App Charter verified across all 10 items; real-user a11y task success recorded; offline-first test (`INV-APP-003`) passed; REQ-PERF-401..415 budgets met in CI; battery accounting clean; cross-app demo completes | Phase 12 blocked; offending app rebuilt | `evidence/G11/` | Required |
| **G12** | 12 | SP-184 | Beta cohort NPS ≥ 40; zero critical bugs (high ≤ 5); live update chain operated on fleet; SDK shipped with 3rd-party demo; deferred-app decisions recorded; founder + steering sign-off; transparency log published; external audit pass; v1.0 signed images published | v1.0 release blocked; Phase 12.5 remediation | `evidence/G12/` | **Required (major)** |
| **G13+** | Post-v1.0 | Defined at G12 | Defined at G12 with evidence. Post-v1.0 roadmap governed per Rule N-02. | — | — | — |

### 19.3 DEC-001 re-examination protocol

At each gate, the founder (or delegate) writes a one-paragraph minuted decision:

- **Continue** — from-scratch path is delivering the promised properties.
- **Adjust** — a specific change (e.g., DEF-001 review, HAL scope, phase re-baseline) that keeps the path viable.
- **Re-open Android-foundation option** — the from-scratch path has failed a gate criterion that cannot be addressed without a foundational change.

The minute is committed to `docs/DECISIONS/gate-GN-dec001-review.md`. This is the founder's explicit, on-the-record re-examination per Part 1 §1.2.

### 19.4 Gate evidence pack required sections

Every gate pack contains (per §27.3):

1. `manifest.json` — hashes of all artifacts.
2. Build log + build config + git commit hash.
3. Test logs from every DoD command.
4. Independent reproduction log (REQ-QA-201).
5. Docs diff vs. prior gate.
6. Deviation report (if any sprint in the phase < 80% complete).
7. DEC-001 re-examination minute.
8. Founder + phase-lead signature.
9. Threshold signature (from G7 onward, per DEF-003).

### 19.5 Gate failure procedures

- **Immediate action:** HALT the offending workstream; file the deviation report; the founder and phase lead review the failure within 5 business days.
- **Remediation plan:** written, with named owner, revised exit criteria (never lowered — rewritten to address the root cause), and re-attempt gate within ≤ 2 sprints.
- **Cascading failures:** if a gate fails twice in a row, the founder re-examines the phase plan and DEC-001. Two failures is data, not noise.
- **Silent gate pass is forbidden.** A gate that "kind of" passed is a failed gate that has not been documented.

---

## 20. Deviation reporting

### 20.1 Sprint deviation

- Any sprint ending with < 80% of its exit checklist done is automatically a FAIL (§8.4 of Part 2).
- The sprint report includes a deviation section: what was planned, what was delivered, why the gap, what changes for next sprint.
- The deviation is logged in `docs/agent-reports/deviations/` with sprint ID, date, and a link to the sprint report.

### 20.2 Phase deviation

- At every gate, if any sprint in the phase had a < 80% completion, or if any exit criterion was renegotiated downward, a phase deviation report is mandatory.
- The phase deviation report becomes part of the gate evidence pack.
- Repeated phase deviations (> 2 in the same phase) trigger a phase re-baseline.

### 20.3 Deviation report template

```
## Deviation Report — [Sprint or Gate ID]

**Planned exit criteria:** [list]
**Delivered exit criteria:** [list with % complete]
**Gap:** [what wasn't done and why]
**Root cause:** [systemic issue, not "ran out of time"]
**Corrective action:** [named owner, what changes]
**Impact on next gate:** [delays? scope change? risk?]
**Founder aware:** [yes/no, date]
```

### 20.4 Escalation chain

Sprint FAIL → phase lead → founder. Gate FAIL → founder → external reviewer (if contracted) → public disclosure (per transparency-log policy).

---

## 21. Change control

### 21.1 Change request types

- **CR-EDITORIAL** — typo, formatting. No semantic change. Approver: documentation lead. Bible REVISION bump.
- **CR-CLARIFICATION** — wording that resolves ambiguity without changing meaning. Approver: founder or delegate. Bible PATCH bump.
- **CR-SEMANTIC** — changes to requirements, phases, sprint counts, or DEF items. Approver: founder + N-14 second signature. Bible MINOR bump.
- **CR-CONSTITUTIONAL** — changes to N-rules, gate definitions, §6 requirements, DEC items. Approver: founder + N-14 second signature. Bible MAJOR bump.
- **CR-SECURITY** — any change touching TCB, security invariants, or the threat model. Approver: founder + TCB-owner + N-14 second signature. Also triggers security-incident review if it weakens an invariant.

Every CR is a PR against this Bible, titled `[CR-<type>] <summary>`, referencing the affected section and the rationale. CRs are merged by the approver; the approver cannot be the author.

### 21.2 Approval matrix

| Change touches | Required approvals |
|---|---|
| Documentation only | Documentation lead |
| Requirements (§6, §9) | Founder + N-14 second |
| Phase/sprint structure (Part 2) | Founder + phase lead + N-14 second |
| DEF items (Part 1 §7.2) | Founder + the DEF's named sunset-reviewer |
| PEN resolutions (Part 1 §7.3) | Founder (or delegate per §21.4) |
| DEC items (Part 1 §7.1) | Founder only (DEC items are founder directives) |
| N-rules (Part 1 §5) | Founder + N-14 second — never below two |
| Gates (§19) | Founder + N-14 second |
| TCB/security invariants (§11, §17) | Founder + TCB-owner + N-14 second |
| This Bible's version scheme (§0) | Founder only, MAJOR bump |

### 21.3 Agent-initiated changes

- An AI agent **may draft** a CR; it **may never merge** one.
- An agent finding a rule ambiguous files a CR-CLARIFICATION; it does not resolve the ambiguity in code.
- An agent that detects a rule violation in existing code halts the workstream and files an incident report. It does not "fix it silently."
- Agent-drafted CRs are subject to the same approval matrix as human-authored ones.

### 21.4 Founder-unavailability protocol

**Purpose:** prevent a single-person bottleneck from freezing a 126-sprint programme when the founder is unavailable (illness, travel, personal time) beyond a defined threshold.

**Threshold:** If the founder is unavailable for more than 7 consecutive business days, the delegation activates.

**Delegate:** A named delegate (documented in `docs/DECISIONS/founder-delegate.md`, updated on any change) is authorized to:

- **Resolve `[PEN-NNN]` items** that block a sprint, within the scope the founder has pre-authorized. The delegate's resolution is recorded in the decision register with `[DELEGATE-RESOLUTION]` tag.
- **Resolve `[PEN-NNN]` items** that block a gate, with the constraint that no gate may be passed without a founder's retrospective sign-off within 10 business days of the founder's return.
- **Sign off on CR-CLARIFICATION and CR-EDITORIAL** changes.
- **Halt workstreams** for safety reasons, even without founder authority.

**Delegate may NOT:**

- Amend N-rules, DEC items, or gate definitions.
- Resolve PEN items that the founder has explicitly reserved (e.g., PEN-013 unless pre-authorized).
- Sign a gate pass without founder retrospective.
- Change the from-scratch decision (DEC-001).

**N-14 rule for the delegate:** any delegate resolution that would normally require N-14 two-person rule (e.g., a TCB change) additionally requires a second steering-group member's signature.

**Resumption:** on the founder's return, all delegate resolutions are reviewed within 10 business days. Resolutions the founder revokes are reversed within the next sprint (work already merged is reverted; the reversal is logged).

**Second delegate:** at least one backup delegate exists, named in the same file. If both primary delegate and founder are unavailable, **work pauses at the last gate pass**; no gate is passed without the founder. This is a safety property, not a bottleneck — the programme is designed to pause rather than drift.

### 21.5 This Bible's version-bump discipline (restated from Part 1 §0)

- `MAJOR` — constitutional change.
- `MINOR` — phase/sprint/requirement change.
- `PATCH` — clarification.
- `REVISION` — editorial.

Every change goes through §21.1. No silent edits. Ever. The current version (1.0.0.0) is recorded in this file's header. Any file claiming to be this Bible without matching changelog entries (§27.7) is not authoritative.

---

## 22. Decision register maintenance

### 22.1 Location and format

`docs/DECISIONS/` contains:

- `DECISIONS.md` — running log of DEC, DEF, and PEN items with current state.
- `DEC-<NNN>-<slug>.md` — one file per DEC item with full reasoning (referenced from `DECISIONS.md`).
- `PEN-<NNN>-<slug>.md` — one file per PEN item with the question, context, options, and resolution (or `PENDING` state).
- `gate-GN-dec001-review.md` — per-gate re-examination minute (§19.3).
- `founder-delegate.md` — delegate naming per §21.4.

### 22.2 Lifecycle

- **DEC** — created by founder directive. Never retired; superseded DEC items are marked `[SUPERSEDED BY DEC-NNN]`.
- **DEF** — created by founder decision with a sunset gate. At the sunset gate, the DEF becomes `[CONFIRMED]` (moved to DEC) or `[OVERRIDDEN]` (replaced by new DEF or DEC).
- **PEN** — created by founder, engineering, or agent escalation. Resolved by founder (or delegate per §21.4). A resolved PEN becomes `[RESOLVED: <date> — <decision>]` and the resolution may spawn DEC or DEF.

### 22.3 PEN-013 resolution (the pivotal item)

Recorded in `docs/DECISIONS/PEN-013-android-app-compatibility.md` with:

- Chosen option (a/b/c/d from Part 1 §7.3).
- Founder reasoning (written, not verbal).
- Impact on positioning statement, target users, and app-framework design (Part 2 §8.14).
- Impact on risk R-02 (ecosystem risk) — updated in §23.
- Reference from DEC-001 reasoning file (Part 1 §1.2) if the choice is to abandon from-scratch (PEN-013 option d).

### 22.4 Decision latency SLA (per REQ-GOV-103)

- **PEN blocking a sprint:** 5 business days.
- **PEN blocking a gate:** 2 business days.
- **Misses logged** in `DECISIONS.md` with a `[LATENCY-MISS]` tag; reviewed weekly by founder.
- **Delegate activation** (§21.4) is triggered by SLA miss on a gate-blocking PEN + founder unavailable > 7 days.

---

## 23. Full risk register

### 23.1 Assessment methodology

- **Likelihood:** Low / Medium / High — based on current evidence, not vibes. Updated at every gate.
- **Impact:** Low / Medium / High / Critical — Critical means programme-ending or promise-breaking.
- **R-NN ID:** permanent. Retired risks are marked `[RETIRED: reason]`; IDs never reused.
- **Review cadence:** every sprint's validation reviews the register delta; every gate reviews the full register.

### 23.2 The register

| ID | Risk | Likelihood | Impact | Mitigation / Gate | Notes |
|---|---|---|---|---|---|
| **R-01** | **Driver famine** — from-scratch kernel + Qualcomm hardware requires re-implementing every driver; Phase 9 may take far longer than 40 sprints | High | High | HAL (DEF-005) reduces redesign; dev-board intermediate (§14.1); honest re-baselining at every gate (Part 2 §8.3) | **Re-examined at every gate** per Part 1 §1.2 |
| **R-02** | **Ecosystem risk** — no Android apps run on from-scratch OS; users cannot adopt | High | High | **PEN-013 resolution required before SP-00**; positioning statement revised per PEN-013 outcome; Developer OS as ecosystem seed | **Blocks SP-00** |
| **R-03** | **Agent architectural drift** — AI agents invent architecture despite locked decisions | Medium | Critical | §10.5 agent code policy; Part 2 §8.4 sprint validation; N-03 escalation discipline; agent prompts versioned | Mitigated by discipline, not eliminated |
| **R-04** | **Founder decision latency** blocks sprints | Medium | Medium | REQ-GOV-103 SLA; §21.4 delegate protocol; batched decisions; DEF items keep work moving | Weekly latency review |
| **R-05** | **Crypto from scratch introduces exploitable flaws** | Low (mitigated) | Critical | **DEF-004 revised: vetted external library default**; from-scratch requires founder sign-off + external crypto audit | Mitigated by DEC-004 |
| **R-06** | **Consent fatigue** — users rubber-stamp prompts, defeating consent model | High | High | REQ-UX-301/302 (≤ 3 interactions revoke, one-question policy); user testing at G8; UI distinguishes AIS proposals from system facts | Social risk, not just technical |
| **R-07** | **Rust toolchain gaps** for freestanding kernels | Medium | Medium | SP-01 spike; asm shims; unsafe budget honestly reported; DEF-002 sunset at G2 | Precedent: Redox, theseus |
| **R-08** | **Talent acquisition** — rare engineers needed (kernel + TEE + formal + AI + mobile hardware) | High | High | PEN-011 (staffing transparency); documentation-first culture; Developer OS as recruiting tool | Competes with Meta/Google/Apple |
| **R-09** | **Update/key compromise** — signing keys stolen or update channel hijacked | Low | Critical | DEF-003 threshold signing (2-of-3); PEN-007 HSM strategy; SP-51 tabletop rehearsal; transparency log detects | Once compromised, unrecoverable without hardware reset |
| **R-10** | **Community/legal perception of "from scratch"** claims — audits find copied code or overbroad claims | Medium | Medium | N-01 scope note (§5); SBOM transparency; legal review of firmware blobs (§15.5); reproducible builds | Reputation risk |
| **R-11** | **TEE isolation insufficient** — hanoip TEE is compromised via side channel | Low | Critical | Defence-in-depth: capability model limits blast radius even if TEE fails; formal verification target at G10 | Cannot fully mitigate on this hardware |
| **R-12** | **Boot-chain rejection** — hanoip secure boot refuses unsigned kernel | Medium | Critical | SP-71 recon → PEN-005 founder decision → dev-board fallback possible | Gate G9's first test |
| **R-13** | **Performance debt** — from-scratch OSs are slow for years by default | High | High | Phase 9.5 performance pass (§8.16); REQ-PERF framework; baseline-at-G4 discipline; CI regression gates | Precedent: Linux 1991, Haiku, Redox |
| **R-14** | **Scope creep** — "while we're here" additions invalidate sprint estimates | High | Medium | N-07 one-milestone rule; §20 deviation reports; CR-SEMANTIC approval required | Cultural risk |
| **R-15** | **Schedule risk** — from-scratch path takes 6–10 years, versus audit-recommended 1–2 years | High | High (founder-dependent) | Honest accounting (§8.1); Developer OS intermediate at G5 (month ~20); founder accepts in DEC-001 | **Re-examined at every gate** |
| **R-16** | **HAL interface insufficient** — Phase 4 HAL cannot accommodate Qualcomm device quirks without redesign | Medium | High | DEF-005 sunset at G4 and G9; HAL designed with Phase 9 targets in mind (SP-31); conformance suite catches gaps | Mitigated but not eliminated |
| **R-17** | **Microkernel IPC performance overrun** — hybrid microkernel slower than required for voice-path latency | Medium | Medium | DEF-001 sunset review at G3/G4; benchmark harnesses continuously; fallback: selective in-kernel fast-path for critical IPC | Mitigated by benchmarking early |
| **R-18** | **AIS quality insufficient** — 80% task completion not achievable on current models | Medium | High | REQ-AI-110 hard gate at G6; SP-49 iteration budget; PEN-008c guides model strategy | Gate-blocking if unmet |
| **R-19** | **Single-founder bottleneck** | High | Medium | §21.4 delegate protocol; N-14 two-person rule; steering group from G5 | Foundational for long programme |
| **R-20** | **Banking app compatibility** — no from-scratch UPI apps | High (if PEN-013=c) | High (product positioning) | PEN-013 resolution shapes strategy; if PEN-013=(a/b), feasibility spike in Phase 8 | Tied to R-02 |
| **R-21** | **Firmware blob licensing** — modem firmware cannot ship publicly | Medium | Medium (blocks cellular in public builds) | §15.5 blob licensing review; documented per-blob; feature shipped-or-not tracked in hardware matrix | Shipping legal risk |
| **R-22** | **Regulatory risk** — on-device AI may face jurisdiction-specific rules (EU AI Act, India DPDP) | Medium | Medium | Legal review at G10; consent architecture exceeds GDPR baseline; on-device-only defaults reduce exposure | Multi-jurisdictional |
| **R-23** | **Trust in transparency log** — the log itself must be verifiable independently | Medium | High | External audit at G10; public verifier tool at SP-69; hash-chain design reviewed at G5 | Trust depends on verifiability |
| **R-24** | **Dependency supply chain attack** — malicious package through SBOM-tracked dependency | Medium | High | Pinned versions; reproducible builds (REQ-QA-201); SBOM audits; vendoring for critical deps | Standard supply-chain risk |
| **R-25** | **Documentation drift** — Parts 1/2/3 and this Bible drift out of sync with actual state | Medium | Medium | N-08 everything-versioned; gate checks doc-update in evidence pack; changelog discipline (§27.7) | Hidden risk for new contributors |
| **R-26** | **App quality insufficient** — apps feel like features rather than polished products | Medium | High | Per-app user testing at SP-168; hard gates at G11; founder product review | G11 |
| **R-27** | **Browser engine decision delays** — PEN-APP-008 unresolved stalls Browser app | High | High | Resolve at SP-127 before Browser sprints; WebKit default proposed | G11 |
| **R-28** | **Camera hardware out-of-scope** — camera app cannot ship on target hardware | Medium | High | Phase 9 matrix honesty; ship with explicit "unavailable" status; re-entry when hardware driver ready | G11 |
| **R-29** | **Phone app reliability** — call drops or emergency dialer failure | Low | Critical | REQ-APP-PHONE-002 emergency dialer is P0 hard-tested; 1000-call reliability stress suite | G11 |
| **R-30** | **AIS quality drift per app** — some apps' intent handling falls below 80% threshold | Medium | Medium | Per-app AIS corpus extension at SP-127; AIS features failing quality are omitted from app | G11 |
| **R-31** | **Consent fatigue from new apps** — 12 apps' capability prompts overwhelm users | High | High | One-prompt-per-app first-launch policy; batch prompts where safe; REQ-APP-COMMON-A11Y-001 consent UX review | G11 |
| **R-32** | **Cross-app integration bugs** — share sheet, intents, deep links fail at edge cases | High | Medium | Comprehensive cross-app test suite (SP-167–168); integration fuzzing in CI | G11 |
| **R-33** | **Deferred apps create perception of incomplete OS** | High | Medium | Deferred-apps doc published pre-release; browser fallbacks honest and verified; consistent user messaging | G11 |
| **R-34** | **Language pack translation quality** — Indic translations feel unnatural or machine-translated | Medium | High | Native-speaker linguistic review per language pack; user testing with native speakers | G11 |
| **R-35** | **Weather data source failure** — user API key model too difficult for non-technical users | Medium | Medium | PEN-APP-002 decision at SP-127; bundled coarse model fallback (b) for users without API keys | G11 |
| **R-36** | **App evidence-pack quality** — apps pass feature tests but fail a11y, perf, or offline criteria | Medium | High | Full Universal App Charter test at G11; zero partial passes permitted | G11 |
| **R-37** | **Beta cohort insufficiently diverse** — feedback skewed toward technical enthusiasts | High | Medium | Cohort design in SP-171 mandates diversity across geography, language, age, and physical ability | G12 |
| **R-38** | **Update infrastructure not ready at G12** | Medium | High | Update chain operated in real during Phase 12; not just rehearsed in simulation | G12 |

### 23.3 Risk review cadence

- **Sprint:** sprint validation reports register delta (new risks, updated likelihood/impact, retired risks).
- **Gate:** full register review; risk-based adjustment to next phase plan if any risk is Critical + High likelihood.
- **Quarterly:** external risk review by founder + delegate + any external reviewer engaged.
- **Major breach:** if any Critical risk becomes realized (e.g., R-12 boot-chain rejection), the founder halts the affected track, files a deviation report, and re-examines DEC-001.

### 23.4 Critical-risk escalation

Risks classified Critical must have:

- A named owner.
- A documented mitigation that is either implemented or scheduled before the next gate.
- A quarterly status report in `docs/RISKS/R-<NN>-<slug>.md`.
- A re-assessment at every gate — even if likelihood is Low.

---

## 24. Success measures

### 24.1 Engineering truth metrics (weekly)

- **Build reproducibility rate:** % of clean-checkout builds that produce bit-identical artifacts. Target: 100% by G1, maintained forever.
- **Gate first-pass rate:** % of gates passed on first attempt. A low rate is not failure — hiding failures is. Target: > 60% first-pass after G3.
- **Deviation rate:** % of sprints FAIL verdict. Target: < 20%.
- **REQ traceability coverage:** % of commits naming a live REQ/DEC/DEF/PEN ID. Target: 100%.
- **TCB size (KLoC):** tracked from G2. Monotonic growth flagged at gate reviews.
- **Unsafe-code ratio in TCB:** tracked from SP-07. Per-release budget review.
- **Test coverage of TCB:** line and branch coverage tracked; target set at G3 (≥ 80% line, ≥ 60% branch as initial targets).

### 24.2 Quality metrics (per gate)

- **Gate evidence pack completeness:** % of required sections present. Target: 100% (fail otherwise).
- **Independent reproduction success:** % of gates reproduced per REQ-QA-201. Target: 100% by G5, maintained.
- **Formal verification coverage:** % of TCB modules with machine-checked proofs. Target: monotonic growth from G10.
- **External audit closure:** % of auditor findings fixed or accepted with rationale. Target: 100% by G10.

### 24.3 Programme health metrics (monthly)

- **Founder decision latency:** mean/median/max days from PEN escalation to resolution. Target: ≤ SLA (REQ-GOV-103).
- **Open PEN items:** count and age distribution. Target: no PEN open > 30 days after escalation.
- **Dogfood usage:** hours per week engineers use Ind OS as primary dev environment (from G5 onward). Target: ≥ 4 hours/week per engineer by G6.
- **Contributor count:** external contributors with merged PRs in the last quarter. Tracked but not optimized — quality over quantity.
- **Documentation staleness:** % of specs with commits in the last 90 days. Target: < 30% stale.
- **Risks in Critical:** count. Target: 0 unmitigated, ≤ 3 mitigated-critical at any gate.

### 24.4 Anti-metrics (what we do NOT optimize)

Explicitly listed to prevent the programme from drifting into perverse incentives:

- **We do not optimize for lines of code.** A smaller TCB is better; adding code is not progress.
- **We do not optimize for commit count.** Velocity is measured by gate-pass rate and DoD completeness, not by output volume.
- **We do not optimize for "demo-ready" speed.** A demo that fails reproducibility (REQ-QA-201) is a party trick.
- **We do not optimize for user count before G10.** Dogfood cohort size is a health metric, not a growth metric.
- **We do not optimize for press mentions.** An externally celebrated milestone that hasn't passed its gate is an N-02 violation waiting to happen.
- **We do not optimize for "AI features shipped."** AIS quality is measured on the REQ-AI-110 corpus, not on a feature count.

### 24.5 Metric review cadence

- **Weekly:** engineering truth metrics reviewed by phase lead + delegate.
- **Monthly:** programme health metrics reviewed by founder + delegate + any external advisor.
- **Per gate:** all metrics reviewed as part of the gate evidence pack.
- **Quarterly:** anti-metric audit — has the programme drifted into optimizing a perverse incentive? Founder signs the audit.

---

## 25. Governance transition framework (activated at G10)

### 25.1 Scope

This section is a **placeholder** per Rule N-02. The governance transition plan cannot be written before G10 evidence exists, because the plan depends on:

- Who actually contributed (not who was imagined to contribute).
- What the actual ABI and capability-model adoption looks like.
- What external audit findings require structural changes.
- What community engagement has materialized.

### 25.2 What is committed now (before G10)

- **P5 of Part 1 (radical openness)** — the Intent ABI and capability model will be under a public governance process eventually; the transition is scheduled, not promised.
- **REQ-GOV-107** — public governance transition plan by G10 (new, §9.5).
- **K5-M4 in PRD-002 §7.6** — governance transition planning milestone (carried forward).

### 25.3 What is not decided (and will not be decided before G10)

- Governance body composition.
- Voting mechanisms.
- Trademark policy.
- Foundation vs. fiscal-host structure.

Premature decisions here would violate N-13 (honesty over momentum) — the founder would commit to a governance structure before knowing what needs governing.

---

## 26. Post-G10 roadmap framework — **ACTIVATED AT G10 RATIFICATION (SP-126)**

> **Status: ACTIVE.** This section was a placeholder until G10. G10 was ratified at SP-126. Phase 11 and Phase 12 are now the live plan of record. Full sprint detail is in §8.18 and §8.19.

### 26.1 Phase 11 plan of record (SP-127 – SP-170, 44 sprints → Gate G11)

**Theme:** Primary Applications & Cross-App Services. Builds the complete user-facing product layer on top of the verified platform from Phase 10.

**Application tiers:**
- **Tier 1 (Mandatory):** Phone, Messages, Contacts, Clock, Calculator, Files, Notes, Settings, Browser, Camera, Gallery, Recorder, Music, Weather, Calendar — all with `[TESTED]` evidence packs at G11.
- **Tier 2 (Expected):** Maps (deferred-first), Email (pending user research), additional language packs.
- **Tier 3 (Deferred):** Streaming Video, Podcast, Banking UPI native, Gaming runtime — each deferred with rationale and re-entry condition in `docs/apps/deferred-apps.md`.

**System surfaces shipped in Phase 11:**
1. **Lock Screen** — hardware-isolated plane; consent cards; emergency shortcut; media controls.
2. **Notification Center** — unified pull-down; batched consent receipts; AIS kill-switch.
3. **Quick Settings** — hardware toggles; privacy feed entry point.
4. **Recents Carousel** — app switcher with swipe-to-kill and preview cards.
5. **Global Search** — cross-app indexed search surface.

**Cross-app services shipped in Phase 11:**
- Share Sheet, Intent Registry, Clipboard (with OTP redaction), Notification Rules.

**Key milestones:**
- SP-127: App Framework Extensions (back-stack, lifecycle, window management, share sheet).
- SP-132–138: Phone + Messages (cellular core apps; emergency dialer hardened).
- SP-154–157: Browser (engine decision required before SP-154 per PEN-APP-008).
- SP-167–168: Cross-app user testing with real external users (15+ people; scripted tasks).
- SP-169–170: Integration QA, a11y audit, G11 evidence assembly.

**Gate G11 exit criteria summary (10 mandatory items — full criteria in §8.18):**
1. All Tier 1 apps pass acceptance with `[TESTED]` evidence.
2. All Tier 2 apps shipped or explicitly deferred with PEN records.
3. Universal App Charter (all 10 commitments) verified per app.
4. A11y: real-user task success recorded per app.
5. Offline-first: `INV-APP-003` passes for every app.
6. REQ-PERF-401..415 budgets met in CI.
7. Cross-app demo script completed by a non-engineer independently.
8. Battery accounting: no app exceeds its declared 24h budget.
9. Founder + steering-group dual sign-off (REQ-GOV-106).
10. Public `AppManifest` and complete evidence pack in `evidence/G11/`.

### 26.2 Phase 12 plan of record (SP-171 – SP-184, 14 sprints → Gate G12)

**Theme:** Post-G11 Beta, SDK, Ecosystem Hardening, and v1.0 Stable Release.

This is a **hardening and stabilization phase — not a feature phase**. Feature gates are closed; only beta-feedback fixes, SDK work, and release infrastructure are in scope.

**Milestones:**
- SP-171: Beta cohort onboarding (50–100 users; diverse geography, language, age, ability).
- SP-177: SDK shipped — documentation, templates, sample apps for Ind-native developers.
- SP-178: Deferred-app PEN review — final decisions on Maps, Banking, Email, Streaming.
- SP-179: First roadmap language pack (Tamil or Telugu per SP-168 user demand data).
- SP-180: Update infrastructure live-operated; staging environment; incident response drilled.
- SP-183: v1.0 release candidate — version string, boot animations, marketing docs.
- SP-184: v1.0 stable release + Gate G12.

**Gate G12 exit criteria summary (9 mandatory items — full criteria in §8.19):**
1. Beta cohort NPS ≥ 40 or documented path.
2. Zero critical bugs; high-severity ≤ 5 with fix dates.
3. Update chain live-operated on fleet (not rehearsed).
4. SDK shipped; at least one third-party app built and deployed.
5. Deferred-app PEN decisions recorded for all PEN-APP items.
6. Founder + steering-group sign-off on v1.0.
7. Transparency log entries for v1.0 published.
8. External audit pass.
9. v1.0 stable images signed (DEF-003 threshold) and publicly available.

### 26.3 Post-v1.0 governance

- **The four core promises** (offline-first, AI-native, provably private, scam-immune) do not get diluted after v1.0.
- **DEC-001 re-examination continues** at every post-v1.0 gate.
- **Post-v1.0 roadmap (v1.1+)** is defined at G12 with evidence — never before (Rule N-02).
- **Governance transition** per §25 begins after G12.

### 26.4 What is forbidden (unchanged)

- Pre-writing G13+ gate criteria without evidence.
- Making external promises about post-G12 features.
- Shipping a v1.0 that partially passes G12 — the gate is binary.

---

## 27. Appendices

### 27.1 Repository layout (final)

```text
indos/
├── README.md                       # Vision, quick-start, architecture diagram
├── CONTRIBUTING.md                 # Process, DCO, commit spec, review rules
├── CODEOWNERS                      # Per-directory ownership
├── LICENSE                         # Apache 2.0 (root; kernel sublicense in kernel/)
├── SECURITY.md                     # Disclosure policy
├── docs/
│   ├── PRD.md                      # INDOS-PRD-001 (v0.1, historical reference)
│   ├── PRD-002.md                  # INDOS-PRD-002 (v1.0, historical reference)
│   ├── BIBLE.md                    # THIS DOCUMENT (Parts 1+2+3 merged)
│   ├── DECISIONS/
│   │   ├── DECISIONS.md            # Register (DEC/DEF/PEN)
│   │   ├── DEC-001-founder-reasoning.md
│   │   ├── PEN-013-android-app-compatibility.md
│   │   ├── founder-delegate.md     # §21.4 delegate naming
│   │   └── gate-GN-dec001-review.md  # Per gate
│   ├── architecture/
│   │   ├── boot-flow.md
│   │   ├── memory.md
│   │   ├── interrupts.md
│   │   ├── scheduler.md
│   │   ├── object-model.md
│   │   ├── syscall-abi-v1.0.md
│   │   ├── capability-model.md
│   │   ├── policy-engine.md
│   │   ├── intent-abi-v1.0.md      # Frozen at SP-42
│   │   ├── semantic-state-exporter.md
│   │   ├── hal-interface.md        # SP-31 deliverable
│   │   ├── threat-model.md         # Expanded from §11
│   │   ├── attack-surface.md       # Living document (§11.3)
│   │   └── OWNERS.md               # Spec owners
│   ├── engineering/
│   │   ├── build.md
│   │   ├── testing.md
│   │   ├── recovery.md
│   │   ├── licensing-and-sbom.md
│   │   ├── unsafe-budget.md
│   │   └── blob-licenses/          # Per-blob legal reviews
│   ├── formal/                     # TLA+/Alloy specs, proof artifacts
│   ├── RISKS/                      # Per-critical-risk files (§23.4)
│   └── agent-reports/
│       ├── deviations/             # §20.3
│       └── <sprint>.md             # Per sprint evidence mini-pack
├── kernel/                         # IndK — Rust, no_std
│   ├── README.md
│   ├── src/
│   ├── Cargo.toml
│   └── rust-toolchain.toml
├── userspace/                      # IndUS — init, services, shell, libs
│   ├── README.md
│   ├── init/
│   ├── lib/
│   ├── shell/
│   └── drivers/                    # User-mode drivers
├── ais/                            # Model runtime, intent broker, memory store
│   ├── README.md
│   ├── runtime/
│   ├── broker/
│   ├── memory/
│   └── models/                     # Signed model artifacts (gitignored payloads; manifest tracked)
├── apps/                           # Shell apps, settings, files, terminal
├── tools/                          # Build, evidence pack generator, model signing
│   ├── agent/prompts/              # Versioned agent prompts (§10.5)
│   ├── evidence/
│   └── qemu-run.sh
├── ci/                             # CI configs, perf gates, reproducibility checks
├── evidence/                       # Machine-generated evidence packs (per gate)
│   ├── G0/
│   ├── G1/
│   └── ...
└── manifest/                       # For future multi-repo if needed
```

### 27.2 Definition of Done (universal)

A change is Done when **all** of the following are true:

1. **Code** exists and builds on `main` (or the sprint branch, pending merge).
2. **Tests** exist for the change (unit, integration, or system, per the change's nature).
3. **Docs** are updated (module spec, README, or ADR, whichever applies).
4. **SBOM** entry exists if a dependency was added or changed (Rule N-09).
5. **Evidence** — the sprint's evidence mini-pack includes the change's test log.
6. **REQ IDs** — commit messages reference the requirement(s) implemented (Rule N-10).
7. **Review** — approved per §10.4.
8. **No `[HYPOTHESIS]` remains** if the change is passing a gate.

Missing any element = Not Done. Not Done = does not merge.

### 27.3 Evidence pack format (referenced from Part 1 §0.7)

Per gate, evidence pack is a directory `evidence/GN/` containing:

```text
evidence/GN/
├── manifest.json              # Hashes of all files below, signed
├── build/
│   ├── build.log
│   ├── build-config.json      # Container hash, toolchain pins, host class
│   └── git-commit.txt         # Full 40-char SHA
├── tests/
│   ├── <test-suite>.log       # One log per suite run
│   └── summary.json           # Machine-readable pass/fail with timings
├── reproduction/
│   ├── commands.txt           # Exact commands for independent reproduction
│   ├── output.log             # Reproducer's successful run log
│   └── reproducer-identity.txt  # Who reproduced; their statement
├── docs-diff/
│   └── <files>.diff
├── deviation/                 # If any sprint in phase < 80% complete
│   └── deviation-report.md
├── dec001-review.md           # §19.3 minute (every gate)
├── signature.txt              # Founder + phase lead sign-off (two-person, REQ-GOV-106)
└── manifest-threshold.sig     # Threshold signature (from G7 onward, DEF-003)
```

Any gate evidence pack missing a required section is invalid. Any evidence pack whose `manifest.json` hashes don't match the artifacts is tampered. Any evidence pack without an independent reproduction (REQ-QA-201) is a party trick.

### 27.4 Ratification record

Authorized by the founder upon signature. Template:

```
# Ratification — INDOS-BIBLE-001 v1.0.0.0

I, [Founder Name], having read the entirety of INDOS-BIBLE-001 (Parts 1, 2, and 3),
hereby ratify it as the single source of truth for Ind OS development.

I acknowledge:
- DEC-001 (from-scratch path) is my decision, made against the audit's recommendation,
  and I have recorded my reasoning in docs/DECISIONS/DEC-001-founder-reasoning.md.
- PEN-013 (Android app compatibility) MUST be resolved before Sprint SP-00 begins.
- DEC-001 will be re-examined at every gate (§19.3); I may re-open the audit's
  recommendation at any gate if evidence warrants.
- The 5–10 year horizon is my accepted position (DEC-003).
- The N-rules (Part 1 §5) are constitutional; I will not amend them without
  N-14 two-person sign-off.

Signature: __________________________
Date: __________________________
```

### 27.5 Review R-01 finding mapping (all 10 items addressed)

| Review finding | Priority | Where addressed | How |
|---|---|---|---|
| **F-01** — No Android app compatibility strategy | P0 | Part 1 §7.3 (PEN-013); §22.3 (resolution record); §8.14 (Part 2 Phase 8); R-02/R-20 | PEN-013 blocks SP-00; positioning revised per PEN-013 outcome |
| **F-02** — DEF-004 crypto default is dangerous | P0 | Part 1 §7.2 (DEF-004 revised); §7.1 (DEC-004); R-05 | Vetted external TLS library is default; from-scratch requires founder exception; SBOM-tracked, non-TCB |
| **F-03** — Sprint count contradicts timeline | P0 | Part 2 §8.1 (corrected); Part 1 §5 N-13 (applies to itself); R-15 | Plan of record: 126 sprints = 5 years pure velocity; realistic 5–10 years; no false precision |
| **F-04** — QEMU→hardware cliff miscalculated | P1 | Part 2 §8.5 (honest accounting); Part 2 §8.15 (40 sprints Phase 9); §9.6 (REQ-HAL series); §14.2 | Re-estimated from 9 to 40 sprints; HAL interface at Phase 4; explicitly classified as re-implementation |
| **F-05** — No HAL portability layer | P1 | §9.6 (REQ-HAL-101..105); Part 2 §8.10 SP-31; DEF-005 (Part 1 §6.2) | HAL interface adopted at G4; conformance tests; two-backend requirement at G9 |
| **F-06** — AIS quality not measured | P1 | §9.2 (REQ-AI-110); Part 2 §8.12 SP-49; §19.2 (G6 gate includes rate ≥ 80%) | Frozen 200-intent corpus; task completion ≥ 80% blocks G6 pass |
| **F-07** — AIS quality not measured [same as F-06, distinct ask: measurable target] | P1 | §9.2 (numeric target ≥ 80%); §24 (quality metrics) | Concrete numeric target with corpus-based measurement |
| **F-08** — No performance pass, no dev OS intermediate | P2 | Part 2 §8.16 (Phase 9.5 performance); Part 2 §8.11 (Developer OS v0.1 at G5); §9.4 (REQ-PERF-101..107); §16 | Phase 9.5 sprint block; Developer OS as shippable intermediate; REQ-PERF framework |
| **F-09** — Testing security-heavy, integration-light | P2 | §9.3 (REQ-QA-201, 202, 203); §17; Part 2 §8.11 SP-39 | Gate reproducibility requirement; independent reproduction; quarterly external reproducer |
| **F-10** — Governance founder-only, no succession | P2 | §9.5 (REQ-GOV-101..107); §21.4 (delegate protocol); §21.2 (approval matrix); Part 1 N-14 (two-person rule) | Two-person rule for constitutional change; founder-unavailability delegation; decision-latency SLA |

### 27.6 Document integrity notes and errata

**E-01 — Part 1 N-13's reference to §7.4.** Part 1 §5 (N-13) states: "§7.4 records a corrected timeline estimate that supersedes the internally inconsistent '~78 sprints (~3 years)…' statement in PRD-002 §8." However, Part 1 §7 ends at §7.3. The corrected timeline estimate actually lives in Part 2 §8.1. **This erratum records the correct reference.** Future versions of this Bible will either (a) insert §7.4 with a cross-reference, or (b) renumber Part 2 §8.1 references. Until then, readers should treat "§7.4" as "Part 2 §8.1."

**E-02 — Part 2 footer says "Sections 9–24" but Part 1 §0.7 references §27.3.** This Part 3 satisfies both by extending to §27 (with §27.3 = evidence pack format). Sections §25 and §26 are placeholders activated at G10; they occupy §25 and §26 to preserve §27 as the appendices section.

**E-03 — Naming collision resolution (Part 1 §1.2).** The audit's "Option A" = Android foundation; the audit's "Option C" = from-scratch. PRD-002 v1.0 mislabelled from-scratch as "Option A." Part 1 §1.2 permanently resolves this by deprecating the audit labels in favour of DEC-001 (from-scratch path). This erratum records that no further use of "Option A/B/C" should occur in any Ind OS artifact.

**E-04 — Part 2 §8.17 says "SP-116–SP-125+" but Part 2 §8.2 table says Phase 10 is "SP-116 – SP-125+ | 10."** These are consistent (10 sprints in SP-116..SP-125), but the "+" reflects that Phase 10 may extend into SP-126+ if hardening continues. The total "126 sprints" in §8.1 counts SP-00 through SP-125; extensions are noted as "+".

### 27.7 Changelog for the Bible

- **1.0.0.0 — 2026-10-26 — Initial ratified version.** Incorporates Expert Review R-01 (all 10 findings, see §27.5). Resolves PRD-002 v1.0's ambiguities (sprint count, crypto default, Option labels). Adds REQ-AI-110, REQ-QA-201/202/203, REQ-PERF-101..107, REQ-GOV-101..107, REQ-HAL-101..105, REQ-SEC-111. Establishes N-14 (two-person rule for constitutional change). Establishes §21.4 founder-unavailability protocol. Splits PEN-008 into 008a/008b/008c. Adds §25 (governance transition) and §26 (post-G10 roadmap) as G10-activated placeholders. Document status: RATIFIED (Sprint SP-00 authorized upon PEN-013 resolution).
- **1.1.0.0 — 2026-10-27 — Phase 11 & 12 Activation + Full App & UX Requirements Absorption.** Absorbs INDOS-UX-PRD-001 v1.0.0.0 (Ind Surface v2) and INDOS-APPS-PRD-001 v1.0.0.0 (Primary Applications). Gate G10 formally ratified at SP-126. Adds: §8.18 (Phase 11, 44 sprints, SP-127–170), §8.19 (Phase 12, 14 sprints, SP-171–184), §8.20 (Universal App Charter + offline-first mandate + ind-ui library), §9.9 (REQ-APP series for all 12 apps + REQ-PERF-401..415), §7.2 additions (DEF-UX-001..003, DEF-APP-001..008), §7.3 additions (PEN-UX-001..007, PEN-APP-001..015), §19 updated to G0–G12 + G13+ (explicit G11/G12 rows), §23.2 additions (R-26..R-38), §26 activated (Phase 11/12 plan of record replacing placeholder), Appendix A (normative design tokens), Appendix B (app capability matrix), Appendix C (app semantic tree spec), Appendix D (deferred apps spec), Appendix E (app evidence pack additions), Appendix F (app launch targets). Source documents INDOS-UX-PRD-001 and INDOS-APPS-PRD-001 absorbed and retired. Total sprints re-baselined: 126 → 185 (SP-00 through SP-184).

### 27.8 Glossary

Terms used in this Bible beyond those in Part 1 §3:

| Term | Meaning |
|---|---|
| **A/B partitions** | Android-style update mechanism with two OS slots; failed update boots previous |
| **Bento Grid** | The adaptive home surface of Ind Surface v2 — 260 px grid units with glassmorphic card layout |
| **BLUF** | Bottom Line Up Front — journalism convention used in risk and gate reports |
| **Capability** | An unforgeable, kernel-derived token granting a specific right on a specific object (Part 1 §6.3) |
| **Consent broker** | The trusted-UI service that presents consent prompts and records decisions |
| **DAST/SAST** | Dynamic/Static Application Security Testing — referenced in Phase 10 |
| **DEC/DEF/PEN** | Decision / Default / Pending (Part 1 §7) |
| **DCO** | Developer Certificate of Origin — signoff protocol on commits |
| **ENF** | Enforcement Layer — capability system + policy engine + consent + audit (Part 1 §3) |
| **Evidence pack** | The hashed, tamper-evident bundle proving a gate's exit criteria (§27.3) |
| **Gate** | Pass/fail phase milestone (Part 2 §8, consolidated in §19 of this part) |
| **HAL** | Hardware Abstraction Layer — the portability boundary between IndK and device drivers (§9.6) |
| **IndK / IndUS / AIS** | Ind Kernel / Ind Userspace / Adaptive Intelligence Subsystem (Part 1 §3) |
| **Ind Surface v2** | The Phase 8 UX shell: 3-zone architecture (64px status bar + 2268px canvas + 128px nav bar), Central Intent Bar, Adaptive Bento Grid |
| **Intent ABI** | The public schema for typed proposals from AIS to ENF (Part 1 §6.3; frozen at SP-42) |
| **Intent Bar** | The Central Intent Bar — a glassmorphic input surface in Zone 2 for voice, text, and contextual AIS proposals |
| **N-rule** | Non-negotiable rule (Part 1 §5) — constitutional |
| **Offline-first** | Design requirement that apps complete their primary task with all network capabilities revoked (`INV-APP-003`) |
| **Pen test / Red team** | Adversarial security exercises |
| **PEN item** | Pending question requiring founder resolution (Part 1 §7.3) |
| **PlaneId::HardwareIsolated** | DSI compositor plane reserved for consent surfaces and call UI; no app can overlay it |
| **REQ ID** | Requirement identifier (Part 1 §0.4, this part §9) |
| **SBOM** | Software Bill of Materials — complete dependency inventory (§15.3) |
| **Semantic tree** | The compositor-verified accessibility tree every app surface must publish (REQ-APP-COMMON-001) |
| **SP-XX** | Sprint number (Part 2 §8) |
| **Suraksha** | The on-device scam/fraud shield — call classifier, SMS classifier, OTP blackout layer |
| **TCB** | Trusted Computing Base (Part 1 §3) |
| **TEE** | Trusted Execution Environment — hardware-isolated execution (Arm TrustZone, CCA) |
| **Tier 1 / 2 / 3** | App tiers — Mandatory / Expected / Deferred (§8.18, §26.1) |
| **Transparency log** | Append-only, hash-chained public record of signed artifacts (§13.5) |
| **Universal App Charter** | The 10-item binding commitment every app makes (§8.20) |

---

## Appendix A — Normative Design Tokens (INV-TOK-001, v1.0.0.0)

These tokens are normative. Any UI component that deviates from these values must file a CR-SEMANTIC (§21.3). Token changes are MINOR version bumps to the Bible.

### A.1 Color

| Token | Value | Use |
|---|---|---|
| `TOK-COLOR-BASE-0` | `#0E0F14` | App background base |
| `TOK-COLOR-BASE-1` | `#16181F` | Raised surface |
| `TOK-COLOR-GLASS-BG` | `rgba(255,255,255,0.06)` | Glass fill |
| `TOK-COLOR-GLASS-BORDER` | `rgba(255,255,255,0.12)` | Glass edge |
| `TOK-COLOR-ACCENT` | `#7C6CFF` | Primary accent (DEF-UX-002) |
| `TOK-COLOR-ACCENT-SOFT` | `rgba(124,108,255,0.16)` | Accent tint |
| `TOK-COLOR-ALERT` | `#FF6B6B` | Deny / destructive (consent cards) |
| `TOK-COLOR-OK` | `#4CD9A6` | Grant / confirm |
| `TOK-COLOR-TEXT-0` | `#F2F3F7` | Primary text |
| `TOK-COLOR-TEXT-1` | `#9BA0AE` | Secondary text |

### A.2 Elevation & Blur

`TOK-BLUR-1: 12px`, `TOK-BLUR-2: 24px`, `TOK-BLUR-3: 40px` — radius degrades under GPU budget pressure, never at the cost of frame time. Elevation via border + blur only; drop shadows are not permitted in v2.

### A.3 Typography

**Type scale:** `TOK-TYPE-S1 12sp / S2 14sp / B1 16sp / B2 18sp / H3 22sp / H2 28sp / H1 36sp`.

**Typeface:** Founder decision per PEN-UX-007. Default: bundled open-license grotesque for Latin scripts + Noto-class Indic companion fonts (Devanagari, Tamil, Telugu, etc.).

### A.4 Spacing & Grid

- `TOK-SPACE-U: 4px` (base unit; all spacing is multiples of 4px)
- Screen margins: `24px` left/right
- Column gutters: `12px`
- Bento Grid unit: `260px`
- Corner radii: `TOK-RADIUS-S 8px / M 16px / L 24px / XL 32px`

### A.5 Motion

- **Duration:** 220 ms default
- **Enter curve:** `cubic-bezier(0.16, 1, 0.3, 1)` (ease-out-expo class)
- **Exit curve:** `cubic-bezier(0.4, 0, 1, 1)` (ease-in)
- **Fling decay:** 220 ms
- **Rule:** All animations run on the compositor frame clock (Phase 4). App-thread animations are a code-review fail.

### A.6 Haptics (subject to PEN-UX-001 hardware reality)

- Tick: 8 ms single pulse
- Confirm: 15 ms double-tick
- Error: 25 ms single pulse
- Default fallback: visual tick if haptic hardware unavailable.

---

## Appendix B — App Capability Matrix (Normative)

Full normative matrix lives in `docs/apps/capability-matrix.md`. Every capability in every app must have a corresponding row justifying the user-facing feature it enables.

**Rule:** Any capability without a matching row is prohibited (`INV-APP-002`). Adding a row is a CR-SEMANTIC per §21.3.

| App | Capability | Justifying user feature |
|---|---|---|
| Phone | `CAP-APP-PHONE-CALLS` | Making/receiving cellular voice calls |
| Phone | `CAP-APP-PHONE-CONTACTS-READ` | Caller ID, quick dial |
| Phone | `CAP-APP-PHONE-AUDIO` | In-call microphone and earpiece |
| Messages | `CAP-APP-MSG-RECEIVE-SMS` | Receiving SMS/MMS |
| Messages | `CAP-APP-MSG-SEND-SMS` | Sending SMS/MMS |
| Contacts | `CAP-APP-CT-STORAGE` | Owning and managing contact database |
| Camera | `CAP-APP-CAM-CAMERA` | Photo and video capture |
| Camera | `CAP-APP-CAM-MIC` | Video audio recording |
| Camera | `CAP-APP-CAM-LOCATION-OPT-IN` | Per-capture GPS geotagging (opt-in) |
| Gallery | `CAP-APP-GAL-STORAGE` | Media directory read/write |
| Browser | `CAP-APP-BRW-NETWORK` | Web browsing |
| Browser | `CAP-APP-BRW-DOWNLOADS` | File downloads to Files with per-file consent |
| Music | `CAP-APP-MUS-AUDIO-OUT` | Audio playback via speaker/headphones |
| Music | `CAP-APP-MUS-NOTIFICATION` | Lock screen and notification media controls |
| Recorder | `CAP-APP-REC-MIC` | Microphone recording |
| Weather | `CAP-APP-WTH-NETWORK` | Fetching weather data |
| Weather | `CAP-APP-WTH-LOCATION-OPT-IN` | Current-location weather (per-fetch consent) |
| Clock | `CAP-APP-CLK-ALARM-SET` | Setting and managing alarms |
| Calendar | `CAP-APP-CAL-STORAGE` | Owning calendar event database |

---

## Appendix C — App Semantic Tree Specification

Every app surface publishes a compositor-verified semantic tree. Missing trees fail the `INV-A11Y-002` CI linter.

### Required node types

| Node type | Role value | Purpose |
|---|---|---|
| **Container** | `container` | A screen, sheet, or major section |
| **Interactive** | `button`, `toggle`, `input`, `select` | Tappable/editable elements |
| **Content** | `text`, `image` | Readable content (with accessible label for images) |
| **Navigation** | `link`, `back`, `tab` | Navigation affordances |
| **Proposal** | `proposal` | AIS-suggested action — screen readers must announce as "Suggestion:" |
| **Status** | `status` | Live regions updated without user action (e.g., call timer) |

### Required node properties

Every node must carry: `id` (stable across app versions), `role`, `label` (human-readable), `state` (enabled/disabled/checked/etc.), `action` (list of available actions).

### CI enforcement

`INV-A11Y-002` — Any node missing a role or label is a CI build failure from G8 onward.

---

## Appendix D — Deferred Applications Specification

Every deferred (Tier 3) app has an entry in `docs/apps/deferred-apps.md` with:

1. **What the app does** — brief description.
2. **Why it is deferred** — honest rationale (bandwidth, complexity, dependency, hardware).
3. **User fallback** — what the user can do instead (browser, third-party app, manual workaround).
4. **Re-entry condition** — the specific PEN resolution or hardware milestone that unlocks the sprint.
5. **Estimated sprint cost** — rough sprint budget once re-entry is approved.

**This document is user-facing** — linked from Settings → About → App Availability.

Current deferred apps (plan of record at v1.0):

| App | Deferred reason | User fallback | Re-entry PEN |
|---|---|---|---|
| Maps / Navigation | Mapping data licensing + driver complexity | Browser (web maps) | PEN-APP-001 |
| Email | User research insufficient to spec at G11 | Browser (webmail) | PEN-APP-004 |
| Music Streaming | Licensing and backend complexity | Local files only | PEN-APP-003 |
| Streaming Video | DRM + network infrastructure | Browser | — |
| Banking / UPI | Regulatory + PEN-013 outcome | Browser (bank website) | PEN-013 |
| Podcast | Scope | Browser | — |
| Calculator Graphing | Scope | Browser (Desmos, etc.) | PEN-APP-011 |
| Browser Extensions | Phase 13+ | — | PEN-APP-012 |
| Calendar Cloud Sync | Privacy architecture | Local .ics export | PEN-APP-013 |
| Face Unlock | Hardware biometric review | PIN / pattern | PEN-APP-007 |
| Gallery Face Grouping | Privacy review | User-tagged albums | PEN-APP-015 |

---

## Appendix E — App Evidence Pack Additions (extends §27.3)

Every app's evidence pack at G11 contains, **in addition to** the standard §27.3 evidence sections:

1. **App capability audit** — app manifest capabilities vs. actual runtime requests; any mismatch is a gate failure.
2. **Charter conformance** — Universal App Charter (§8.20) checked item-by-item with `[PASS]` or `[FAIL]` per item.
3. **A11y evidence** — real-user task success records with test participant demographics (§8.3, REQ-APP-COMMON-A11Y-001).
4. **Offline evidence** — offline test run log with all network capabilities revoked (`INV-APP-003`).
5. **Performance evidence** — CI automated perf gate output for the app's `REQ-PERF-4xx` budget entry.
6. **Battery evidence** — 24h dogfood battery accounting log; budget not exceeded.
7. **Crash evidence** — crash rate from soak test: ≤ 1 per 1000 sessions target.

---

## Appendix F — App Launch Targets

Plan of record for v1.0 stable (all subject to evidence — N-13 applies):

| Phase | Sprints | Duration | Milestone |
|---|---|---|---|
| Phase 11 | SP-127–SP-170 (44 sprints) | ~22 months at pure velocity | All Tier 1 & Tier 2 apps shipped and tested |
| Gate G11 | After SP-170 | — | Feature-complete phone OS |
| Phase 12 | SP-171–SP-184 (14 sprints) | ~7 months at pure velocity | Beta cohort, SDK, hardening, v1.0 |
| Gate G12 | After SP-184 | — | **★ Ind OS v1.0 STABLE RELEASE** |

**Post-G12 (v1.1+)** app roadmap is defined at G12 with evidence — never before (Rule N-02).

**Anti-metrics (from INDOS-APPS-PRD-001 §14.3, binding):**
- We do not optimize for app count — 12 great apps beat 50 mediocre ones.
- We do not optimize for daily-active users or engagement metrics.
- We do not optimize for feature parity with iOS or Android — we optimize for user needs.
- We do not optimize for launch date — G11/G12 are pass/fail; the calendar is not a gate.

---

*End of INDOS-BIBLE-001 v1.1.0.0.*

*The Bible is complete. Phase 11 and Phase 12 are the active plan of record, activated at Gate G10 ratification (SP-126). Sprint SP-127 may begin upon founder sign-off. Every sprint after it stands or falls on the evidence it produces.*

*Nothing in this document claims any app is beautiful, fast, or reliable yet. It defines how the programme will find out, honestly, gate by gate. Rule N-02 applies to this document itself.*

