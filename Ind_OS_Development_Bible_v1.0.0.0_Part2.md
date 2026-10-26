# IND OS — THE DEVELOPMENT BIBLE

**Document ID:** INDOS-BIBLE-001
**Version:** 1.0.0.0 (Part 2 of 3 — Programme Structure, Phases, Sprints, Validations)
**Date:** 2026-10-26
**Companion file:** Part 1 (`Ind_OS_Development_Bible_v1.0.0.0.md`) contains Sections 0–7 and is the authoritative foundation. This file continues the SAME document — when merged, Part 2 content follows Part 1 content directly. All cross-references (N-rules, DEC/DEF/PEN items, REQ IDs, gates) resolve against Part 1.

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
| 10 | SP-116 – SP-125+ | 10 | Hardening, formal verification, external audit, beta readiness | G10 | Beta candidate |

**Total: 126 sprints (SP-00 – SP-125).** Gates G11+ (beta, v1.0) are deliberately not pre-written — they are defined at G10 with evidence (Rule N-02).

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
- **SP-31** — **HAL skeleton + performance baselines**: the Hardware Abstraction Layer interface (memory-mapped register access, IRQ, DMA, clocks/resets) that every driver must route through — designed with Phase 9's Qualcomm targets in mind. Simultaneously: **performance baselines measured and recorded** (boot time, IPC round-trip, syscall latency, context-switch cost, FS throughput) — these become the targets' *denominators* for the Phase 8/9.5 budgets (§8.12). DEF-001/002 sunset review held and minuted.

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

- **SP-41** — Model runtime sandbox (REQ-AI-101): signed-model loader only, isolated inference process with minimal capabilities, CPU backend, scheduler-enforced memory/time limits.
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

**Goal:** the user-friendliness promise becomes visible. App-framework design MUST respect the PEN-013 resolution (recorded at SP-00); if PEN-013 chose an Android-compat track, its feasibility spike lands here as a parallel workstream with its own budget — and does not expand scope silently (N-07).

- **SP-60** — Compositor & display server (user mode, capability-fed): damage tracking, vsync, input routing with focus integrity (anti-overlay inherits Consent Broker isolation).
- **SP-61** — Shell v2 (the Ind surface): home, notifications (AIS suggestions visually distinct — REQ-UX-305), search/intent bar.
- **SP-62** — Intent surface UI: type/speak intent → proposal card → consent → execution → explanation receipt; Indian-language IME architecture begins (REQ-AI-108).
- **SP-63** — App framework: plain-language capability manifests (REQ-USE-207); sandbox spawn; inter-app sharing via user-mediated grants.
- **SP-64** — Accessibility services (REQ-UX-304, P0): screen reader over the accessibility IPC bus, magnification, switch access; **tested with real users recruited this sprint** — results minuted, no exceptions.
- **SP-65** — Proactive engine UI (REQ-AI-106): suggestion ranking controls; dismiss/snooze/explain on every suggestion; aggressiveness dial.
- **SP-66** — Core apps suite: files, settings (consent surface), why-explorer, terminal, notes — capability-minimal dogfood of the framework.
- **SP-67** — Performance budget enforcement: frame-time, cold-start, and per-app memory budgets enforced in CI against the G4-measured baselines (REQ-PERF series, Part 3 §13); regressions block merge.
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

### 8.17 Phase 10 — Hardening, verification, beta readiness (SP-116 – SP-125+)

**Goal:** turn the working system into something defensible.

- **SP-116 – SP-118** — Formal verification push (REQ-SEC-106): capability-core invariants proved (delegation monotonicity, revocation soundness); proof artifacts in CI.
- **SP-119** — External security audit (REQ-SEC-109): contracted TCB review; findings triaged; criticals fixed before any outward "secure" language.
- **SP-120** — Red-team programme (REQ-SEC-110): multi-week adversarial campaign; results and fixes documented internally.
- **SP-121** — Production ops: signing ceremonies rehearsed for real; update infrastructure staging; incident-response playbook; support/docs readiness.
- **SP-122** — Beta cohort planning: devices (QEMU + hanoip + dev boards), consent-first telemetry (opt-in, local-first — PRD-001 §7.4), user-research protocol.
- **SP-123 – SP-125+** — Continuous hardening cadence: rolling security sprints, patch SLAs, roadmap beyond beta decided by founder with G9/G9.5/G10 evidence in hand.

**Gate G10 exit:** external audit closed; verification proofs in CI; update chain operated for real; beta plan approved. **G11+ (beta, v1.0) defined at G10 with evidence — not pre-written here (Rule N-02).**

---

*…continues in Part 3 (Sections 9–24: consolidated gates table, requirements deltas including REQ-AI-110 / REQ-QA-201 / REQ-PERF series, deviation & change control with founder-unavailability protocol, risk register, success measures, appendices).*
