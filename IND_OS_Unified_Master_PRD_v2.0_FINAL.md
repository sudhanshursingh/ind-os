# IND OS — Unified Master PRD v2.0 (Final)

## From Shipped MVP to AI-Native Kernel. One Document. One Roadmap. No Compromise.

**Document ID:** `IND_OS_UNIFIED_PRD_v2.0_FINAL`
**Supersedes:** `IND_OS_Master_PRD-2.md` (`v1.0.1-BIBLE`) and `IND_OS_FROM_SCRATCH_PRD_v1.0`
**Founder & Lead System Architect:** Sudhanshu Raj Singh
**Official Repository:** `https://github.com/sudhanshursingh/ind-os`
**License:** Apache 2.0 (userspace) / GPLv2-compatible (kernel) — 100% Open Source, forever
**Reference Hardware:** Motorola Moto G60 (`hanoip` — Snapdragon 732G / `SM7150-AC`, 6 GB LPDDR4X, 128 GB UFS 2.1) as Tier-0; architectural portability to future ARM64 mobile/edge devices
**Development Methodology:** BIBLE-locked decisions + formal specification + machine-verifiable DoDs + human architectural sign-off. The AI agent executes; it does not architect.
**Document Relationship:** This document is the **single source of truth** for all Ind OS work from this date forward. All architectural decisions herein are FINAL. Changes require a `[PRD-AMENDMENT]` PR with explicit human sign-off from the Lead Architect.

---

## 0. How To Use This Document

1. **Part I** records what is already shipped and what carries forward. Nothing in Part I is open for re-litigation.
2. **Parts II–VI** are the locked architecture: principles, decisions, layers, Intent ABI, security model.
3. **Part VII** is the executable development plan: sprints, milestones, phases, hard gates, and DoDs. Each sprint block is designed to be pasted to an agentic coding tool as a self-contained work order.
4. **Part VIII** defines release quality gates. A phase may not start until the previous gate passes. A gate is defined by *demonstrable architectural properties*, not dates.
5. Any contradiction between this document and either predecessor document is resolved **in favour of this document**.

---

# PART I — STATE OF THE UNION: WHAT THE 18 SPRINTS DELIVERED

## 1.1 The v1.0 MVP Track: Complete

The AOSP-based MVP defined in `IND_OS_Master_PRD-2.md` (`v1.0.1-BIBLE`) ran 6 phases / 18 sprints / 36 weeks against the Moto G60 testbed. **That track is complete and shipped.** The following components are live, tested, and in production use:

| MVP Deliverable | Status | Quality Gate Result (v1.0 release blockers) |
| :--- | :--- | :--- |
| `IndNeuralCore` — privileged Binder AI service (`indos.neural.daemon`) with native C++ `llama.cpp`/`whisper.cpp` engine (`libind_ai.so`) | **Shipped** | ≥ 15 tok/s on 1.5B Q4_K_M; RSS ≤ 1050 MB; mmap eviction frees >900 MB in <15 ms |
| Zero-copy mmap model pager with `MADV_DONTNEED` eviction on memory pressure | **Shipped** | Verified via `dumpsys meminfo` deltas |
| Deterministic GBNF grammar sampler (`ind_action.gbnf`) | **Shipped** | 100% JSON validity across 500-prompt En/Hi/Hinglish suite |
| `ind-zero-touch` — wake-word, ECAPA-TDNN voice gate, streaming Indic ASR, voice orb | **Shipped** | E2E voice latency ≤ 850 ms; wake TP > 95% at 1.5 m; idle drain ≤ 1.5%/hr |
| Accessibility semantic graph pruner (< 350-token JSON) + gesture injector | **Shipped** | Flagship demo: multi-step WhatsApp flow executed fully hands-free |
| `ActionPolicyEnforcer` — hardcoded Green/Amber/Red matrix, dual-context isolation | **Shipped** | Prompt-injection corpus: 0 actions triggered on injection attempts |
| `ind-suraksha-shield` — screen-share OTP blackout, scam classifier, APK guard | **Shipped** | OTP redaction verified under AnyDesk/TeamViewer screen share |
| SELinux air-gap (`ind_ai_daemon.te`) — zero network sockets for the AI daemon | **Shipped** | `getenforce` = Enforcing; `socket()` returns EACCES from AI context |
| `IndLauncher` + Ind Command Bar + contextual cards; opt-in BYOK cloud relay with PII scrubber | **Shipped** | `tcpdump` shows 0 bytes in default air-gapped mode |
| Full LineageOS 22.x-based flashable ROM (`ind_hanoip-ota-v1.0.zip`), zero-bloat config, sandboxed microG toggle | **Shipped** | BHIM/PhonePe/GPay/YONO pass integrity; hands-free control works post-clean-flash |

**What the MVP proved:** (a) a 1.5B-class on-device model can drive a real voice-first phone experience on 6 GB RAM; (b) GBNF-constrained decoding gives deterministic tool calls from small models; (c) a hardcoded policy enforcer below the LLM survives real prompt-injection attacks; (d) users will trade Play-Integrity convenience for privacy when the alternative is verifiable; (e) agent-driven development with locked decisions compounds velocity instead of oscillating.

**What the MVP could not prove — and never will on this substrate:** (a) that the AI runtime's isolation survives a kernel-level compromise rather than a policy-level mistake; (b) that the semantic layer can be native rather than retrofitted from Accessibility APIs; (c) that the OS can offer *user-verifiable* integrity rather than Google-verifiable integrity; (d) that AI computation (model memory, KV caches, accelerator time) can be scheduled as a first-class kernel resource; (e) that the trusted computing base can shrink below what a monolithic kernel permits.

**Conclusion:** The MVP validated every user-facing promise. The architecture beneath it cannot deliver the next set of promises. The from-scratch track is therefore no longer a parallel research program — it is the only path.

## 1.2 What Carries Forward, What Is Retired

### 1.2.1 Carried Forward (Component → New Role)

| MVP Asset | New-Architecture Role |
| :--- | :--- |
| Voice pipeline design (wake-word → speaker verify → ASR → SLM → action → TTS) | **Retained, hardened.** Identical pipeline structure at Layer 7, now capability-mediated and TEE-isolated. The 850 ms E2E budget is preserved as a hard invariant. |
| GBNF grammar enforcement inside the sampler | **Retained as an implementation detail.** The AI runtime satisfies the Intent ABI schema via constrained decoding (GBNF or equivalent). The ABI, not the grammar, is the contract. |
| Green/Amber/Red action matrix | **Retained, relocated.** Re-expressed as a capability-attenuation hierarchy enforced by the Semantic Gateway *below* the AI. The matrix's tiers become capability derivation classes. |
| `ActionPolicyEnforcer` invariants (no AI clicks on UPI PIN, no forwarding OTPs, no actions from untrusted text) | **Retained, strengthened.** Recoded as the hardcoded, formally specified policy engine of the Semantic Gateway (Layer 6), backed by taint tracking and the capability model. |
| SELinux air-gap | **Retained for Stage A only**, as one isolation layer among several (SMMU, TEE, LSM, capability LSM). Superseded by TEE isolation + capability microkernel in Stage B. |
| Indic ASR / wake-word / speaker-verify models and training data | **Retained.** Repackaged as signed model artifacts deliverable through the transparency-logged update channel. |
| `IndLauncher` UX research, command bar, contextual cards | **Retained.** The launcher becomes a pure consumer of the Intent ABI and semantic state exporter rather than a privileged accessibility client. |
| Repo layout, CI, dev harness discipline, agent operating protocol | **Retained and extended** with formal-specification gates (see Part VII). |
| Community, brand, and the shipping device fleet | **Retained.** Moto G60 devices running v1.0 are the permanent Tier-0 bring-up target and the regression fleet for every new phase. |

### 1.2.2 Retired (With Reasons)

| MVP Element | Retirement Reason |
| :--- | :--- |
| AOSP/LineageOS framework as the long-term base (D1) | A monolithic-kernel, Android-framework substrate cannot host kernel-level AI primitives, native semantic exports, or a below-50-KLoC TCB. It remains the maintenance track for the v1.0 fleet only. |
| Accessibility API as the semantic source (D5) | Retrofitted, screen-scraping-derived, and bypassable. Replaced by the kernel-adjacent semantic state exporter. |
| Accessibility gesture injection as the action mechanism | Replaced by capability-mediated intent declarations executed through the compositor and system services. |
| Play Integrity / microG attestation bridge (D10) | Replaced by measured boot + user-verifiable remote attestation. Banking-app compatibility on the from-scratch OS is a deliberate, documented non-goal for the early phases. |
| `madvise`-based model eviction | Replaced by the kernel's tensor-aware, page-granular AI memory domain manager. `madvise` becomes merely one fallback path inside the memory manager. |
| 1.5B model cap as a *product* ceiling (D3) | Retained as the Stage A ceiling for the Moto G60; no longer a policy. Larger models are permitted where hardware allows; the capability model guarantees model size never implies capability expansion. |
| Overlay/Magisk rapid-iteration workflow (D8) | Retired with the AOSP track. Rapid iteration on the new substrate is achieved via the Ind SDK simulator + QEMU targets + fastboot kernel/boot-image flashing, not framework overlays. |

### 1.2.3 Maintenance Track (Parallel, Non-Blocking)

The v1.0 AOSP build enters a **maintenance track**: security patches, model artifact updates, and community device ports. It consumes no core-team capacity beyond ~10% and is explicitly *not* on the critical path. Its purpose is to keep the user fleet alive and the brand warm while the from-scratch OS is built.

---

# PART II — VISION, PRINCIPLES, AND THE CENTRAL CLAIM

## 2.1 Vision

Ind OS is an open-source, AI-native operating system in which AI is not an app, a service, or a daemon, but a **first-class, kernel-adjacent primitive** with the same structural authority as scheduling, memory management, and IPC — and with *less* authority than the user.

## 2.2 The Central Architectural Claim

> **An AI-native operating system cannot be built by adding AI to an existing OS. It must be built by making the OS's core primitives expressible in terms an AI can reason about, constrain, and verify — and by making AI's computational requirements expressible in terms the kernel can schedule, isolate, and protect.**

## 2.3 The Five Non-Negotiable Principles (Kernel-Native Restatement)

- **P1 — Intent Is the Primary Interface, Not Pixels.** The OS exposes a semantic, machine-readable representation of system state as a kernel-mediated primitive. Humans interact through pixels and voice; AI agents interact through typed intent declarations with deterministic semantics, mediated by a policy engine that lives *below* the AI.
- **P2 — AI Computation Is Scheduled, Isolated, and Accounted for Like Any Other Critical Resource.** Model paging, KV-cache eviction, NPU/DSP/GPU dispatch, and inference isolation are kernel abstractions with admission control, thermal coupling, and accounting.
- **P3 — Security Is an Isolation Property Enforced Below the AI.** The AI runtime is confined by a mechanism that does not depend on the AI's cooperation, the correctness of a policy file alone, or the vigilance of a userspace daemon. Capability-based isolation, TEE-backed execution, and (in Stage B) a formally verified microkernel core.
- **P4 — The User's Data Never Leaves the Device Unless the User's Intent Explicitly Requires It.** The absence of network access for AI and personal-context workloads is the natural state. Network access is a capability that must be explicitly granted, audited, and revocable at the kernel level.
- **P5 — Radical Openness.** Every line of Ind OS — kernel, drivers, AI scheduler, semantic gateway, compositor — is open source. No binary blobs in the critical security path. **The Intent ABI is a public specification**, not a proprietary protocol.

---

# PART III — THE UNIFIED DECISION MATRIX

## 3.1 Legacy Decisions (D1–D10): Final Disposition

| # | MVP Decision | Final Disposition |
| :--- | :--- | :--- |
| D1 | AOSP/LineageOS fork base | **Superseded** (retained only for the v1.0 maintenance track) |
| D2 | llama.cpp + whisper.cpp runtime | **Retained** as the initial AI runtime; replaceable behind the Intent ABI; Rust-native engine targeted Phase 2–3 |
| D3 | 1.5B model ceiling | **Retained** as Stage A hardware ceiling for Tier-0; no longer a policy constraint |
| D4 | 4-stage voice cascade | **Retained**, every stage capability-mediated and TEE-isolated |
| D5 | Accessibility API semantic graph | **Superseded** by the native semantic state exporter |
| D6 | GBNF grammar enforcement | **Retained** as an implementation detail of the AI runtime |
| D7 | Dual-context isolation + Green/Amber/Red + Java verifier | **Superseded and strengthened** by the Semantic Gateway + capability model + information-flow tracking |
| D8 | Overlay rapid-iteration workflow | **Superseded** by simulator + QEMU + fastboot iteration loop |
| D9 | Agent-driven, Bible-locked methodology | **Retained and extended** with formal-specification gates (FS-D12) |
| D10 | microG + Play Integrity | **Superseded** by measured boot + user-verifiable remote attestation |

## 3.2 From-Scratch Decisions (FS-D1–FS-D12): LOCKED

| # | Decision | Difficulty | Risk | Rejected Alternative | Chosen Architecture | Engineering Justification |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **FS-D1** | Kernel Foundation | 10/10 | Critical | Monolithic Linux with AI patches | **Two-stage strategy.** Stage A (Years 1–4): mainline Linux as driver host + isolation substrate, zero AI logic in kernel space. Stage B (Years 4+): seL4-derived, Rust-implemented capability microkernel as primary kernel; Linux retained only as legacy driver VM. | Linux buys hardware access on Day 1 (Qualcomm NPU/GPU upstreaming is advancing) but is structurally hostile to verifiability. seL4-class verified isolation is the only credible path to containing a compromised AI runtime without trusting its sandbox. |
| **FS-D2** | Kernel Language & Safety | 9/10 | High | C-only kernel | **Rust-first kernel development.** All new Ind kernel components in Rust; C only for legacy driver shims and arch assembly. | Rust eliminates entire vulnerability classes; Qualcomm's own Rust driver conversion program and Rust GPUVM in mainline prove viability. A from-scratch OS has no legacy excuse for C. |
| **FS-D3** | AI Workload Scheduling | 9/10 | High | Inference as a normal userspace process | **Kernel-integrated AI workload orchestrator.** An AI Workload Class with model-paging memory domains, accelerator dispatch queues, KV-cache-aware preemption, thermal/power coupling. Stage A: `sched_ext`-style BPF/Rust scheduler; Stage B: native kernel subsystem. | sched_ext enables Rust schedulers and GPU-aware policies today; the end-state requires scheduling AI *memory and accelerator time*, not just CPU time. AgentRM-style multi-level feedback queues for LLM workloads inform the design. |
| **FS-D4** | Memory Architecture for AI | 9/10 | Critical | Anonymous memory + swap | **Unified AI Memory Domain with capability-mediated DMA.** Capability-typed model weight / KV-cache / activation objects, zero-copy DMA-BUF flows, page-granular weight eviction, tensor-aware eviction policy (weights → KV → activations). | Bare-metal tensor virtualization and FlexServe-style SMMU/NPU-only protection show these guarantees are achievable; the kernel must make them *enforceable*, not aspirational. |
| **FS-D5** | Semantic Interface (Intent ABI) | 8/10 | High | Accessibility retrofit; screenshot agents | **Kernel-mediated Semantic Gateway with a formally specified, publicly versioned Intent ABI** (schema, semantics, test vectors). | LUMOS-class semantic layers and AgenticOS-class intent-filter architectures validate the approach; Ind OS makes it native, versioned, and public. |
| **FS-D6** | Capability-Based Security | 9/10 | Critical | SELinux/AppArmor policy files as sole mechanism | **Capability-based security as the only authorization mechanism for AI-adjacent operations.** Unforgeable, attenuable, revocable, auditable tokens; derived by the kernel; the AI's world is a graph of capabilities, not a namespace of paths. | Capability models eliminate the confused-deputy problem that plagues ambient-authority systems; they are the foundation of seL4, Fuchsia, and Genode, and extend naturally to AI agents. |
| **FS-D7** | Confidential AI Execution | 9/10 | High | Normal-world inference with SELinux | **TEE-backed inference isolation (Arm TrustZone now; CCA Realm world when hardware permits).** Weights, KV cache, and runtime protected from a compromised normal-world kernel; I/O cryptographically attested and policy-mediated. | FlexServe demonstrates practical TrustZone inference protection with acceptable overhead; CCA is being explored for Android-class workloads. Confidential AI execution is baseline, not premium. |
| **FS-D8** | Graphics & Compositor | 8/10 | Medium | SurfaceFlinger; X11 | **Wayland-native compositor with semantic surface metadata.** The compositor is the only component receiving raw display buffers; it exports semantic surface descriptions (roles, actions, trust levels) natively. | Wayland is the security-oriented, mobile-capable protocol (Plasma Mobile, Sailfish, postmarketOS); semantic extensions make it the AI's window into the system without pixels. |
| **FS-D9** | Hardware Enablement | 9/10 | High | Vendor BSP kernels; opaque blobs | **Mainline-first with documented, forward-portable vendor patch sets.** Goal: mainline or documented patches for every AI-, display-, and security-critical component. | postmarketOS proves mainline viability on Qualcomm SoCs; upstream Hexagon NPU and Adreno efforts mean "vendor blob forever" is no longer binding. |
| **FS-D10** | Update & Measurement | 8/10 | High | A/B partitions without attestation | **Measured boot + public update transparency log + user-verifiable remote attestation.** Every stage measured into hardware counters; signed, reproducible updates; attestation verifiable by any party the user chooses — not a single gatekeeper. | Hardware roots exist (TrustZone/CCA/TPM-equivalents); the transparency log provides public auditability. This is *more* than Play Integrity: user-verifiable integrity. |
| **FS-D11** | AI Model Lifecycle | 7/10 | Medium | Bundled model, never updated; or silent model updates | **Federated, capability-scoped, signed model updates via the transparency-logged channel.** A new model inherits the replaced model's capability set; capability expansion requires user approval; atomic rollback. | Prevents the "model update as privilege escalation" attack — the AI-era malicious OTA. Enforceable only because of FS-D6. |
| **FS-D12** | Development Methodology | 6/10 | Medium | Agent vibe-coding | **BIBLE-locked decisions + formal specification (Intent ABI, capability model, memory manager) + machine-verifiable DoDs + human sign-off.** The agent's output is model-checked where possible, not merely reviewed. | Agent drift is the dominant failure mode of AI-assisted kernel development. Pre-answered questions plus machine-checkable specifications are the only defence. |

---

# PART IV — THE STACK: LAYER-BY-LAYER SPECIFICATION

Read bottom to top. Each layer communicates only through the interface directly above/below it. No layer may skip a layer.

```
+===============================================================================================+
| LAYER 7 — INTENT & EXPERIENCE                                                                 |
|   Voice-first semantic shell · Semantic launcher · Contextual intent cards                    |
|   [Consumes: Intent ABI. Produces: user-visible actions and feedback.]                       |
+===============================================================================================+
                  |  Intent ABI (typed, versioned, schema-validated)
+===============================================================================================+
| LAYER 6 — SEMANTIC GATEWAY & POLICY ENGINE                                                    |
|   Intent mediator · Capability resolver · Hardcoded action policy · Audit logger ·           |
|   Information-flow tracker                                                                    |
+===============================================================================================+
                  |  Capability invocations (unforgeable tokens)
+===============================================================================================+
| LAYER 5 — AI RUNTIME (CONFIDENTIAL EXECUTION CONTEXT)                                        |
|   LLM inference engine · Speech engine · Semantic graph builder · Intent compiler            |
|   Model pager · KV-cache manager · NPU/DSP/GPU dispatch (via kernel only)                    |
|   [TEE-protected memory. No raw network. No ambient FS. Sees only capability descriptors,    |
|    semantic state, user input buffers.]                                                       |
+===============================================================================================+
                  |  AI Workload ABI (memory objects, accelerator queues, completion events)
+===============================================================================================+
| LAYER 4 — KERNEL AI PRIMITIVES (Stage B: native; Stage A: Linux + BPF/Rust policies)          |
|   AI workload scheduler · AI memory domain manager · Capability table · Semantic state        |
|   exporter · Accelerator dispatch (NPU/GPU/DSP) · DMA-BUF zero-copy fabric                    |
+===============================================================================================+
                  |  Hardware abstraction interface (device model, interrupts, MMIO, SMMU)
+===============================================================================================+
| LAYER 3 — KERNEL CORE (Stage B: microkernel / Stage A: mainline Linux substrate)             |
|   Stage B: seL4-derived capability microkernel, Rust components, formally specified IPC       |
|   Stage A: mainline Linux, driver host + isolation substrate only                             |
+===============================================================================================+
                  |  Boot firmware interface (TF-A, UEFI where supported)
+===============================================================================================+
| LAYER 2 — TRUSTED FIRMWARE & MEASURED BOOT                                                    |
|   ARM Trusted Firmware · OP-TEE or equivalent TEE OS · Measured boot into hardware counters   |
+===============================================================================================+
                  |  ARMv8-A / ARMv9-A exception levels, TrustZone/CCA partitioning
+===============================================================================================+
| LAYER 1 — SILICON & HARDWARE ROOT OF TRUST                                                    |
|   Snapdragon 732G (Tier-0) · Kryo 470 · Adreno 618 · Hexagon 680 DSP · TrustZone · 6 GB RAM  |
|   Future: ARMv9-A + Confidential Compute Architecture (Realm world)                           |
+===============================================================================================+
```

## 4.1 Layer 3 — Kernel Core

### Stage A (Years 1–4): Mainline Linux as Substrate
Linux is used **exclusively as a driver host and isolation primitive**. No AI inference, model loading, or intent interpretation in kernel space. Responsibilities:

1. Memory isolation between AI runtime, semantic gateway, compositor, and application domains.
2. DMA/SMMU programming for zero-copy tensor movement CPU↔GPU↔NPU (FlexServe-style protected memory unmapped from normal-world SMMU).
3. Accelerator driver hosting: Hexagon NPU via FastRPC; Adreno GPU via Freedreno/Turnip.
4. `sched_ext`-style AI workload scheduler (Rust BPF) with accelerator affinity, memory residency, and thermal awareness.

### Stage B (Years 4+): Capability Microkernel Core
Progressive replacement of the Linux substrate by a **Rust-implemented, seL4-lineage capability microkernel** extended with AI-native primitives:

- Address spaces, threads, capability-mediated creation.
- Synchronous/asynchronous IPC with formal semantics.
- Capability derivation and revocation as the **only** authorization mechanism.
- AI workload objects as first-class kernel objects: model memory objects, accelerator queue objects, inference session objects.
- Semantic state export as a kernel-maintained read-only capability.
- Linux retained **only as a legacy driver VM**, the same pattern high-assurance separation kernels use.

**Why a microkernel is the correct end-state:** a monolithic kernel places drivers, filesystems, network stacks, and — in an AI-native OS — inference runtimes in one privileged address space. One memory-safety bug anywhere compromises everything. For an OS whose promise is that the AI cannot be tricked into harming the user, **the AI runtime must not share a privilege domain with the kernel**. Microkernel IPC overhead is mitigated by modern hardware and zero-copy mechanisms.

## 4.2 Layer 4 — Kernel AI Primitives (the reason from-scratch is necessary)

### 4.2.1 AI Workload Scheduler
Optimises end-to-end inference latency under memory/thermal constraints while preserving fairness for non-AI work.

| Dimension | Observes | Controls |
| :--- | :--- | :--- |
| Model residency | Which model pages resident; KV-cache pressure | Admit vs. queue new inference requests |
| Accelerator affinity | NPU/GPU/CPU availability and thermal headroom | Which compute unit executes which model layer |
| Thermal state | SoC/battery temperature, sustained power budget | Downclock, migrate, or defer inference |
| User-visible latency | Interactive path (voice) vs. background (classification) | Priority class and preemption eligibility |
| Capability scope | Session's held capabilities | Preemption safety for in-flight transactions |

Stage A: `sched_ext` BPF + userspace Rust policy engine. Stage B: native kernel subsystem with direct capability-table and memory-domain access. AgentRM-style MLFQ admission control informs the design; moving the decision into the kernel gives it memory-residency and thermal visibility no userspace runtime can reliably obtain.

### 4.2.2 AI Memory Domain Manager
The memory manager understands **tensor semantics**, not just pages.

- **Model Memory Object (MMO):** capability-typed weights; read-only mappable into inference sessions; **page-granular eviction** (not `madvise` hacks) with transparent reload.
- **KV-Cache Object:** capability-typed per-session cache; encryptable at rest; tier-migratable (LPDDR → ZRAM → storage) by recency; **zeroed on session termination with a kernel-enforced guarantee**.
- **Tensor DMA Fabric:** kernel-mediated zero-copy CPU↔GPU↔NPU movement via DMA-BUF with capability-scoped access; a tensor mapped into the NPU cannot be read by an untrusted CPU process.

**Eviction policy:** tensor-aware LRU, classes evicted in order (1) model weight pages → (2) KV-cache pages → (3) activation buffers. Eviction measured in **milliseconds**; triggered by memory pressure and thermal pressure (evicting weight pages cuts bandwidth and power).

### 4.2.3 Semantic State Exporter
Kernel-maintained **read-only capability** exposing a machine-readable graph: running tasks, visible UI surfaces (via compositor cooperation), available capabilities, resource conditions, policy constraints. Stable identifiers, roles, bounds, action affordances — LUMOS-class semantics produced natively below the AI. The AI consumes it through a capability; the gateway validates every action derived from it against the capability graph.

### 4.2.4 Capability Table
Capabilities are: **unforgeable** (only the kernel derives them), **attenuable** (rights can shrink, never grow), **atomically revocable** (with all descendants), **auditable** (every invocation logged with ID, process, policy decision). The AI holds only capabilities the gateway derived for the current task; on task end they are revoked. **No ambient authority**: the AI cannot remember capabilities across tasks or enumerate undiscovered resources.

## 4.3 Layer 5 — AI Runtime (Confidential Execution Context)

Userspace component (Stage B) in a TrustZone secure world / Arm CCA Realm / equivalent TEE. Contains: LLM inference engine (initially llama.cpp-derived; Rust-native target Phase 2–3), speech engine (wake/ASR/speaker-verify/TTS — MVP models carried forward), semantic graph builder, intent compiler, model pager and KV-cache manager (issuing capability-mediated requests to Layer 4).

**Isolation invariants:**
1. **No raw network.** No sockets. External data requires a gateway-mediated capability through a separate network service domain.
2. **No ambient filesystem.** Only capability-typed model objects, input buffers, semantic state.
3. **No direct hardware.** All accelerator dispatch through the kernel scheduler and DMA fabric.
4. **Attested execution.** TEE measures runtime code and model hashes; the gateway verifies the attestation before granting capabilities.

**Why it matters:** prompt-injection-to-code-execution is not impossible. A TEE-protected runtime means a compromised rich OS cannot read weights, KV caches, or user prompts — and the gateway policy still applies regardless.

## 4.4 Layer 6 — Semantic Gateway & Policy Engine

The **only** path by which the AI runtime affects the world. Deliberately **not AI-controlled**; hardcoded, formally specified where possible, human-auditable.

1. **Intent Mediator** — validates Intent ABI schema, checks declared capabilities, dispatches or denies with machine-readable reason.
2. **Capability Resolver** — maps intent to the minimal capability set; escalates to the user (AMBER/RED path) when a required capability is absent.
3. **Action Policy Enforcer** — hardcoded invariants, direct descendant of the MVP verifier: *no AI-originated action may tap a UPI PIN confirmation; no AI-originated action may forward an OTP SMS; no action derived from untrusted text may invoke a money-moving capability.*
4. **Information-Flow Tracker** — taint labels; untrusted screen text cannot become the target of a SEND_MESSAGE/PAYMENT action.
5. **Audit Logger** — append-only, hash-chained, tamper-evident; readable by the user and their chosen auditor without any cloud account.

**The never-trust-the-AI's-self-report invariant:** the AI may output `tier: GREEN` for a RED action. The gateway ignores the AI's tier and computes the tier from action type, capabilities, and taint labels. This is the from-scratch answer to MVP D7: not a verifier *beside* the AI but a policy engine *below* it, in a separate trust domain.

## 4.5 Layer 7 — Intent & Experience

Voice-first, semantic-first shell. Launcher, settings, and notifications are consumers of the Intent ABI, not privileged insiders.

- **Voice pipeline:** wake → speaker verification → streaming ASR → intent compilation → gateway → action → TTS. Identical structure to the MVP cascade; every stage capability-mediated; runtime TEE-isolated. **L_E2E ≤ 850 ms preserved as a hard invariant.**
- **Semantic launcher:** home screen generated from the semantic state exporter; tasks, capabilities, actions as first-class objects.
- **Contextual intent cards:** generated by the gateway (not the AI), capability-scoped, high-confidence.

---

# PART V — INTENT ABI: PUBLIC SPECIFICATION (v1.0 of the ABI)

## 5.1 Intent Declaration Schema (Abridged)

```json
{
  "intent_id": "uuid",
  "timestamp_ms": 0,
  "session_id": "uuid",
  "declared_tier": "GREEN | AMBER | RED",
  "actions": [
    {
      "action_type": "SYS_TOGGLE | OPEN_APP | CLICK_NODE | INPUT_TEXT | SCROLL | READ_SCREEN | PLACE_CALL | SEND_MESSAGE | PAYMENT_INTENT",
      "target": "capability_uri",
      "params": { "key": "value" },
      "data_taint_labels": ["UNTRUSTED_SCREEN_TEXT", "USER_VOICE", "BANKING_SMS"],
      "required_capabilities": ["capability_uri"]
    }
  ],
  "user_feedback": { "tts_text": "string", "visual_state": "optional" }
}
```

## 5.2 Semantic Gateway Decision

- `ALLOW` — within session capability set; taint constraints satisfied; no invariant violated.
- `ALLOW_WITH_CONFIRMATION` — AMBER capability derivation; user confirms via voice or UI.
- `DENY` — required capability absent or hardcoded invariant violated.
- `DENY_AND_AUDIT` — suspected attack (untrusted text → RED capability); blocked, session may be terminated, logged as a security incident.

## 5.3 Why a Public ABI Matters

Third-party apps natively join the semantic layer; researchers formally verify the gateway against the spec; the AI runtime is replaceable without moving the security boundary; the OS is auditable by anyone — the only credible path to trust in an AI-native system. ABI changes go through the public governance process (Part VII, §7.8), not a private roadmap.

---

# PART VI — SECURITY ARCHITECTURE

## 6.1 Trusted Computing Base Minimisation

| Stage | TCB Components | Size Target |
| :--- | :--- | :--- |
| Stage A | TF-A boot firmware, TEE OS (OP-TEE), Linux isolation logic used by the AI path (SMMU/IOMMU, capability LSM checks, AI memory manager invariants), Semantic Gateway policy engine, capability broker | < 100 KLoC |
| Stage B | TF-A, TEE OS, seL4-derived microkernel (formally verified), Rust capability components, Semantic Gateway policy engine | < 50 KLoC |

AI runtime, compositor, network stack, filesystem, and all device drivers are **outside the TCB**. Their compromise must not violate: AI cannot move money without human authorization; AI cannot exfiltrate data without a capability; untrusted text cannot become a trusted action.

## 6.2 Capability Model (Formal Sketch)

A capability `C = (object_id, rights, constraints)` where `rights ⊆ {READ, WRITE, EXECUTE, INVOKE, DERIVE, REVOKE}` and `constraints` are predicates (time bounds, single-use, taint requirements, user-presence). Kernel-enforced laws:

1. Invocation requires holding the capability.
2. Derivation may only attenuate (rights/constraints ⊆ parent's).
3. Revocation invalidates all descendants atomically.
4. No kernel-object access without a capability naming it.

Green/Amber/Red becomes a **capability-attenuation hierarchy**: GREEN = basic capability; AMBER = capability derived only after a voice-confirmation step; RED = derivable only after biometric/passcode gate. The AI never "decides" a tier; the kernel enforces the capability's constraints regardless of AI output.

## 6.3 Measured Boot & Remote Attestation

Every boot stage measures the next into hardware-backed counters: boot firmware → TEE OS → kernel (microkernel hash in Stage B; kernel + module hashes in Stage A) → gateway policy engine → AI runtime hash → model weight hashes. A device-fused key signs a **remote attestation capability**; the measurement log publishes to the public transparency log. Any verifier the user chooses — bank, MDM, or the user's own service — verifies integrity **without trusting a Google-run attestation service**. Deliberate trade-off: banking apps requiring Google-specific attestation may not run; Ind OS prioritises user-verifiable integrity.

## 6.4 Threat Model

| Threat | Stage A Defence | Stage B Defence |
| :--- | :--- | :--- |
| Prompt injection via screen text/messages | Taint tracking + gateway information-flow control; AI cannot generate capability invocation from untrusted text | Same, enforced by capability model below the AI |
| Compromised AI runtime (code execution) | TEE isolation: no weight/KV/prompt access from normal world; gateway policy still applies | TEE + verified microkernel isolation; runtime is userspace, zero kernel privilege |
| Malicious DMA from compromised peripheral | SMMU/IOMMU + AI memory domain manager; protected memory unmapped from normal-world SMMU | Microkernel-enforced device capabilities |
| Kernel exploit via driver | Drivers in Linux, isolated from TEE and gateway; AI memory inaccessible from Linux kernel | Drivers userspace/legacy-VM; outside microkernel TCB |
| Model update as privilege escalation | Capability-inheritance rule: new model ≤ old model's capabilities; expansion needs user approval | Microkernel capability model enforces |
| Supply-chain compromise | Signed artifacts + transparency log + reproducible builds | Same |
| Physical attacker | TEE-protected inference memory + encrypted KV cache + measured boot | Arm CCA Realm memory encryption where supported |

---

# PART VII — DEVELOPMENT PLAN: THE KERNEL TRACK (SPRINTS 19+)

**Sprint numbering continues from the completed MVP (Sprints 1–18).** All kernel-track sprints are 2-week iterations. Every sprint block below is a self-contained work order: locked decisions, tasks, deliverables, machine-verifiable DoD. Hard phase gates (§7.7) are **demonstrable architectural properties** — a phase may take longer; it may not proceed with a gate unmet.

## 7.0 Baseline Assumptions Carried From the MVP

- The v1.0 AOSP fleet is live; the maintenance track (≤ 10% core capacity) keeps it patched. It is the regression/comparison fleet for the new OS.
- MVP voice models, ASR, speaker-verify embeddings, scam-classifier corpora, and the 500-prompt En/Hi/Hinglish test suite are banked assets, repackaged as signed model artifacts.
- The Ind team has shipped 18 sprints of agent-driven development under Bible lock; the FS-D12 protocol is an extension of a proven discipline, not a new one.
- Tier-0 hardware: Moto G60 (`hanoip`, SM7150-AC). Tier-1 target (acquired Phase K3): an ARMv9-A device with CCA support.

---

## 7.1 PHASE K0 — Specification, Formal Models & Hardware Baseline (Sprints 19–30 | Weeks 1–26 | ~6 months)
**Phase Objective:** Produce the machine-checkable specifications, the TEE inference prototype, the mainline hardware baseline, and the development environment. No production OS yet.

### Sprint 19 — Kernel-Track Repository & Toolchain Bootstrap
- **Locked:** FS-D2 (Rust-first), FS-D9 (mainline-first), FS-D12 (spec-first).
- **Tasks:**
  1. Create `kernel/` tree in the repo: `kernel/docs` (specs), `kernel/linux` (mainline fork with patch-queue directory), `kernel/rust`, `kernel/sim` (Ind SDK simulator skeleton), `kernel/tools` (flash/verify harness for `hanoip`).
  2. Pin toolchain: Rust (locked `rust-toolchain.toml`), LLVM/Clang, `bindgen`, OP-TEE build for `hanoip`, TF-A source, `repo` manifest for mainline + device trees.
  3. CI: every commit builds `kernel/rust` crates, runs the spec test vectors, and produces a signed build manifest hash for the transparency log.
- **Deliverables:** Repo skeleton; CI green; reproducible-build script.
- **DoD:** Two independent machines produce bit-identical build artifacts (SHA-256 match logged).

### Sprint 20 — Intent ABI v1.0 Specification
- **Tasks:**
  1. Author `kernel/docs/intent-abi-v1.0.md`: full schema (§5.1), semantics of every `action_type`, tier computation rules, error/denial codes, versioning policy.
  2. Produce JSON-Schema + 200 machine-readable test vectors: valid intents, malformed intents, boundary cases (empty actions, maximum array sizes, unicode Indic text).
  3. Reference validator in Rust (`ind-intent-validate`) that any component can call.
- **DoD:** All 200 vectors pass; validator rejects 100% of intentionally malformed corpus; document publicly committed.

### Sprint 21 — Capability Model Formal Specification (Part 1)
- **Tasks:**
  1. Specify capability tuple, rights lattice, constraint predicate language, derivation rules, revocation semantics in TLA+.
  2. Model-check with TLC: derivation-monotonicity (rights never grow), atomic revocation, no-ambient-authority (a process cannot name an object without a capability).
  3. Begin Rust trait skeletons (`ind-capability`) mirroring the spec 1:1.
- **DoD:** TLC reports 0 invariant violations across ≥ 10⁶ states; spec and Rust skeleton cross-referenced line-by-line.

### Sprint 22 — Capability Model Formal Specification (Part 2) + Attenuation Hierarchy
- **Tasks:**
  1. Encode Green/Amber/Red as derivation predicates in the TLA+ model: GREEN = base rights; AMBER = requires `user_voice_confirmed` constraint in derivation chain; RED = requires `user_presence_biometric` constraint.
  2. Prove: no derivation chain can produce a RED capability without a `user_presence_biometric` link. Add adversarial models (compromised AI runtime attempting self-derivation).
  3. Draft the Stage A capability-broker architecture: LSM (`ind_cap_lsm`) + userspace broker, mapping MVP SELinux domains onto capability sets.
- **DoD:** TLC proves the RED-derivation theorem; adversarial models show 0 reachable violation states.

### Sprint 23 — Mainline Baseline: Boot, Display, Touch on `hanoip`
- **Locked:** FS-D1 Stage A, FS-D9.
- **Tasks:**
  1. Bring mainline Linux (locked LTS) + `hanoip` device tree to fastboot-bootable image: kernel, minimal initramfs, framebuffer/DRM display, touchscreen input.
  2. Document every patch in `kernel/linux/patches/` with `README` (purpose, upstream status, forward-port notes). Zero opaque blobs beyond vendor-required modem/DSP firmware.
- **DoD:** Device boots to a Wayland test client (even a color surface) with working touch; patch log complete.

### Sprint 24 — Mainline Baseline: Wi-Fi, Audio, Power Management
- **Tasks:**
  1. Enable Wi-Fi (ath10k/qualcomm path as applicable), ASoC audio (speaker/mic), battery/thermal sysfs, CPUFreq/idle on all 8 Kryo cores.
  2. Power measurement harness: `scripts/power_probe.sh` reading `/sys/class/thermal`, fuel-gauge, currentnow; nightly CI power report.
- **DoD:** 8-hour idle battery test logged; thermal zones readable; audio loopback verified.

### Sprint 25 — Hexagon NPU Access via FastRPC (Research Spike)
- **Tasks:**
  1. Stand up FastRPC channel from Linux to ADSP/CDSP; enumerate available DSP libraries; run a trivial DSP offload job.
  2. Assess the mainline Hexagon upstreaming state; document what is usable vs. what requires the vendor stack. Write the NPU-enablement decision memo (`kernel/docs/npu-enablement.md`).
- **DoD:** Decision memo committed with concrete Stage A NPU plan and fallback (GPU-first via Turnip) if FastRPC is insufficient.

### Sprint 26 — Adreno GPU Compute via Turnip (Research Spike)
- **Tasks:**
  1. Build Mesa Turnip for Adreno 618; run Vulkan compute conformance subset; benchmark GEMM kernels vs. 4-thread Kryo ARMv8.2 SDOT baseline (reuse MVP benchmark methodology).
  2. Decide Stage A inference acceleration routing: GPU-primary vs. CPU-primary, per model class.
- **DoD:** Benchmark report with tok/s and perf/W for at least two GEMM shapes; routing decision recorded.

### Sprint 27 — TEE Inference Prototype: OP-TEE + llama.cpp
- **Locked:** FS-D7.
- **Tasks:**
  1. Port OP-TEE OS to `hanoip` (secure world boot via TF-A); implement a trusted application (TA) hosting a minimal llama.cpp inference session with a small test GGUF.
  2. Normal world client requests inference via OP-TEE shared memory; weights and KV cache live **only** in secure-world memory.
- **DoD (Gate-critical):** A normal-world root process cannot read model weights or KV cache via `/proc/<pid>/mem`, `ptrace`, DMA, or SMMU remapping — demonstrated by an automated attack script that must fail on every vector.

### Sprint 28 — TEE Prototype Hardening: Attestation & I/O Mediation
- **Tasks:**
  1. TA measures its own code + model hash; exposes an attestation report signed by the device key to the normal world.
  2. Mediate all TA I/O through a single normal-world proxy service: prompt in, tokens out — nothing else.
  3. Measure inference overhead vs. normal-world baseline (target: ≤ 15% regression for protected execution).
- **DoD:** Attestation verifies against the public key; attack surface enumeration shows no second I/O path; overhead report committed.

### Sprint 29 — Semantic State Exporter Prototype
- **Locked:** FS-D5, FS-D8.
- **Tasks:**
  1. Stand up a minimal Wayland compositor (wlroots-based) with semantic surface metadata protocol extension: each surface advertises `role`, `actions`, `trust_level`.
  2. Build the exporter service: compositor + process table + resource conditions → LUMOS-style semantic graph with stable node IDs; publish as a read-only socket.
  3. Port the MVP semantic-graph-pruner test suite against the native graph; token budget ≤ 400 for a full home screen.
- **DoD:** Native graph matches or beats MVP graph quality on the test suite; AI-readable; no accessibility APIs involved.

### Sprint 30 — Semantic Gateway Prototype v0 + Phase K0 Integration
- **Tasks:**
  1. Rust implementation: intent validator (Sprint 20), capability resolver (Sprints 21–22), hardcoded policy rules (MVP matrix ported), taint tracker, audit logger (hash-chained).
  2. Integration demo: TEE runtime (Sprint 27) emits an intent → gateway validates → capability broker executes a real action on the mainline baseline (toggle torch) → audit log records.
  3. Run the MVP 500-prompt suite against the gateway; the injection corpus must produce `DENY_AND_AUDIT` with zero actions.
- **DoD:** End-to-end demo on device; injection corpus result: 0 actions; audit log hash-chain verifies.

### 🔒 GATE K0 (enters Phase K1)
An independent reviewer, reading only the committed specs, can answer **"what happens if the AI runtime is compromised?"** with a precise, mechanism-level answer. The TEE inference prototype demonstrates a normal-world process cannot read weights/KV cache via any demonstrated vector. Intent ABI, capability model, and semantic exporter specs are public and machine-checkable.

---

## 7.2 PHASE K1 — Stage A Kernel & AI Substrate (Sprints 31–82 | ~2 years)
**Phase Objective:** A daily-usable phone — mainline Linux substrate, TEE-isolated AI runtime, capability-mediated semantic gateway, Wayland-native shell — that a real user can carry, while containing zero Android userspace.

Thirteen 4-sprint milestones follow. Each lists its four sprints (one-line scope) then full detail. DoDs are machine-verifiable on `hanoip`.

### Milestone K1-M1 (Sprints 31–34): Wayland Shell & Core Apps
- **S31:** Compositor polish: multi-surface layout, lock screen, notifications surface.
- **S32:** Ind shell (Layer 7 consumer): launcher consuming semantic graph; MVP `IndLauncher` UX ported.
- **S33:** Core capability-native apps: dialer, messaging, settings — written against the Intent ABI, zero Android APIs.
- **S34:** Input method (Indic keyboards), power/session management, OOM policy.
- **DoD (M1):** Device usable as a phone for calls/SMS/settings via touch UI; shell RAM ≤ 450 MB; cold boot to interactive < 25 s; zero Android userspace processes (`ps -A` audited).

### Milestone K1-M2 (Sprints 35–38): AI Memory Domain Manager on Linux
- **Locked:** FS-D4.
- **S35:** Design `ind_ai_mm` kernel module + Rust userspace manager; define Model Memory Object (MMO) and KV-Cache Object ioctls; DMA-BUF import/export for tensors.
- **S36:** Page-granular weight eviction with transparent reload; eviction path benchmark harness (target ≤ 20 ms per 100 MB evict on UFS 2.1).
- **S37:** Tensor-aware LRU policy (weights → KV → activations) driven by memory pressure + thermal pressure.
- **S38:** KV-cache zeroization on session teardown (verified by memory-scrub test); encryption-at-rest option via TEE-wrapped keys.
- **DoD (M2):** Under synthetic memory pressure, the manager holds the interactive inference path under 120% of baseline latency while reclaiming ≥ 800 MB; zeroization verified; DMA-BUF zero-copy path proven (no memcpy in the hot path — verified by ftrace).

### Milestone K1-M3 (Sprints 39–42): AI Workload Scheduler (`sched_ext`)
- **Locked:** FS-D3.
- **S39:** `sched_ext` Rust BPF skeleton: AI workload class registration, per-class attributes (accelerator affinity, latency class, memory residency).
- **S40:** Admission control + MLFQ for inference requests (AgentRM-informed); interactive (voice-path) class with preemption of background inference.
- **S41:** Thermal coupling: scheduler reads thermal zones; downclock/migrate/defer policy at 80%/90%/95% throttle thresholds.
- **S42:** Accelerator-aware dispatch: NPU (FastRPC per Sprint 25 decision) and GPU (Turnip) queues; per-layer routing for supported models.
- **DoD (M3):** Voice-path inference P95 ≤ 220 ms prefill under concurrent background load; no thermal runaway in 30-min sustained generation (≥ 80% peak clocks maintained); scheduler decisions fully logged and replayable.

### Milestone K1-M4 (Sprints 43–46): Capability LSM + Broker (Stage A)
- **Locked:** FS-D6.
- **S43:** Implement `ind_cap_lsm`: capability checks at file/socket/device opens; deny-by-default for the AI runtime domain.
- **S44:** Userspace capability broker: derivation, attenuation, revocation, audit; capability URIs namespaced (`cap://fs/path?rights=r`, `cap://ui/node/42`, `cap://net/dns` …).
- **S45:** Replace MVP-equivalent SELinux domains: AI runtime, gateway, compositor, apps each hold only derived capabilities; SELinux retained as defence-in-depth.
- **S46:** Revocation storm test: revoke a parent capability mid-task; verify atomic descendant invalidation across processes.
- **DoD (M4):** The AI runtime can be shown — by audit log — to possess zero capabilities at task start and only task-scoped capabilities thereafter; `socket()` from the runtime returns EACCES (invariant from MVP carried forward); revocation test passes 1000/1000 iterations.

### Milestone K1-M5 (Sprints 47–50): AI Runtime v1 in TEE
- **Locked:** FS-D7, FS-D11.
- **S47:** Port the full MVP voice pipeline (wake, ECAPA verify, streaming ASR) into TEE/isolated domains with capability-mediated I/O; carry MVP models as signed artifacts.
- **S48:** LLM session management in TEE: model load via MMO, KV-cache object binding, GBNF/ABI-constrained decoding (MVP grammar carried forward).
- **S49:** Model artifact signing + transparency-log verification on load; capability-inheritance rule enforced at load (new model ≤ old model's capability set).
- **S50:** Attested session establishment: gateway verifies TEE attestation before any capability derivation; runtime replacement drill (swap engine build, verify attestation path survives).
- **DoD (M5):** Full voice cascade E2E ≤ 850 ms **with TEE isolation active**; model swap without capability expansion verified; attestation verified on every session; `/proc` + DMA attack vectors re-run (Sprint 27 script) and still fail.

### Milestone K1-M6 (Sprints 51–54): Semantic Gateway v1 (Production)
- **Locked:** FS-D5, FS-D12.
- **S51:** Gateway as a supervised always-on service: mediator, resolver, hardcoded policy, taint tracker, audit logger — full MVP invariant set ported and extended.
- **S52:** Taint engine v1: label propagation across intents, surfaces, clipboard, SMS; information-flow rules (untrusted text cannot reach money-moving capabilities — machine-checked rule table).
- **S53:** AMBER/RED user-confirmation paths: voice confirmation loop with speaker-verify gating (MVP thresholds), biometric/passcode gate for RED.
- **S54:** Audit transparency UX: user-facing audit viewer app, hash-chain verification, export for external auditors.
- **DoD (M6):** MVP injection corpus: 0 actions, 100% `DENY_AND_AUDIT`; AMBER confirmation loop verified hands-free; RED requires biometric (tested, not configurable); audit viewer verifies its own log.

### Milestone K1-M7 (Sprints 55–58): Semantic Launcher & Shell v1
- **S55:** Semantic launcher consuming the native graph: tasks/capabilities/actions as first-class objects.
- **S56:** Contextual intent cards (gateway-generated, capability-scoped); notification stream rebuilt on semantic surfaces.
- **S57:** Screen translate/summarize (offline, capability-mediated screen read); MVP Sprint-14 features ported.
- **S58:** Voice orb UI (MVP design language) on Wayland; TTS feedback pipeline.
- **DoD (M7):** "What can I do with this photo?" answered with real capability-scoped actions; translate/summarize ≤ 3 s on a full screen of text; orb interaction fully hands-free.

### Milestone K1-M8 (Sprints 59–62): Connectivity & Modem
- **S59:** Modem bring-up on mainline: voice/SMS/data via QMI/RIL-equivalent service (documented vendor interfaces only).
- **S60:** Telephony apps (dialer/messaging) complete: calls, SMS, contacts.
- **S61:** Data networking with per-domain policy: network access is a **capability** — AI runtime has none by default; apps request scoped network capabilities.
- **S62:** Wi-Fi calling/cellular battery soak; radio power audit.
- **DoD (M8):** 48-hour carrier soak on the user's SIM: calls, SMS, data stable; AI runtime holds zero network capabilities in audit log at all times.

### Milestone K1-M9 (Sprints 63–66): Camera, Media & Audio Pipeline
- **S63:** Camera ISP bring-up (documented vendor interface or mainline CAMSS path); capability-scoped camera access.
- **S64:** Media codecs (hardware where documented, software fallback); gallery app.
- **S65:** Full ASoC audio pipeline: routing, echo cancellation for voice pipeline, audio focus policy.
- **S66:** Media stress + thermal integration with the AI scheduler (camera + inference concurrency test).
- **DoD (M9):** Camera → photo → semantic share sheet works; concurrent camera preview + background inference respects thermal policy; audio loopback quality meets voice-pipeline SNR requirement.

### Milestone K1-M10 (Sprints 67–70): Measured Boot, Updates & Transparency Log
- **Locked:** FS-D10.
- **S67:** Measured boot chain: TF-A → OP-TEE → kernel → gateway → runtime → model hashes into hardware-backed counter; boot attestation service.
- **S68:** Signed, reproducible OTA updates (A/B) with rollback; update server publishing to the public transparency log.
- **S69:** Independent verifier tool: any user verifies their running system against the transparency log from a second device.
- **S70:** Key ceremony v1: device-fused keys, signing keys in HSM, ceremony documented and witnessed.
- **DoD (M10):** Boot measurements verify end-to-end; a tampered boot image is detected and refuses to attest; OTA round-trip (update + rollback) verified; verifier tool runs from a stock laptop.

### Milestone K1-M11 (Sprints 71–74): BFSI Shield on the New Stack
- **S71:** Port MVP Suraksha Shield: screen-share OTP blackout (now compositor-native — the compositor knows surfaces and trust levels), scam SMS classifier, APK guard (for sideloaded packages on the new packaging format).
- **S72:** Payment-surface hardening: UPI PIN surfaces marked `trust_level=red` in semantic metadata; gateway invariant "no AI click on PIN confirmation" enforced below the AI.
- **S73:** Financial fraud corpus expansion (Indian scam patterns); classifier retrained and signed as artifact.
- **S74:** Red-team exercise: external tester attempts OTP exfiltration + AI-coerced payment across 50 scenarios; all must fail.
- **DoD (M11):** Red-team report: 50/50 scenarios blocked with audit evidence; OTP blackout verified under real screen-share apps.

### Milestone K1-M12 (Sprints 75–78): Performance, Power & Hardening Pass
- **S75:** Budget invariants re-verified on the new stack (§8 table); fix regressions.
- **S76:** 72-hour stress: monkey-equivalent fuzzing + automated voice-loop + thermal cycling; memory-leak audit.
- **S77:** Security hardening: kernel config audit, attack-surface reduction, fuzzing (syzkaller on `ind_cap_lsm`, compositor protocol fuzzing).
- **S78:** Dogfood release: 25 devices into the community; feedback triage loop.
- **DoD (M12):** All §8 gates pass on the new substrate; 72-hour stress with zero critical faults; dogfood NPS and bug report recorded.

### Milestone K1-M13 (Sprints 79–82): Stage A Release — "Ind OS 2.0"
- **S79:** Release candidate builds; reproducibility verified; transparency log entries published.
- **S80:** Documentation: install guide, porting guide, SDK preview, security whitepaper.
- **S81:** Launch: source + signed images + demo film (the hands-free flagship demo on the from-scratch stack).
- **S82:** Retrospective; Stage B staffing; Phase K2 spec finalization.
- **DoD (M13 / Gate K1):** Public flashable build; all §8 gates; the flagship demo runs on a device with **zero Android userspace**.

### 🔒 GATE K1 (enters Phase K2)
On a Moto G60 running Stage A: a user places calls, sends messages, and toggles settings entirely by voice with the AI runtime TEE-isolated; a prompt-injection attempt yields `DENY_AND_AUDIT` with zero action; model weights and KV cache are provably absent from every normal-world process's memory map; boot measurements verify against the transparency log with an independent verifier.

---

## 7.3 PHASE K2 — Microkernel Core Bring-Up (Sprints 83–134 | ~2 years)
**Phase Objective:** Replace the Linux substrate for the security-critical and AI-critical paths with a Rust-implemented, seL4-lineage capability microkernel; Linux demoted to a legacy driver VM.

### Milestones

| Milestone | Sprints | Scope | DoD |
| :--- | :--- | :--- | :--- |
| **K2-M1: Microkernel Foundation** | 83–90 | seL4-derived core ported to SM7150-AC: address spaces, threads, IPC, capability derivation/revocation in Rust; boot on bare metal beside TF-A; formal spec of IPC semantics (CSP-style) committed. | Kernel boots to a hello-world capability server; IPC latency ≤ 2 µs round trip on Kryo 470; spec test vectors pass. |
| **K2-M2: Capability System Complete** | 91–98 | Full capability table, derivation, attenuation, atomic revocation; the TLA+ model from Sprints 21–22 re-validated against the Rust implementation (model-based testing); device-capability model for SMMU-backed devices. | Property-based test suite (10⁹ ops) shows 0 violations of the four capability laws (§6.2). |
| **K2-M3: AI Workload Objects** | 99–106 | First-class kernel objects: model memory objects, accelerator queues, inference sessions; AI Workload Scheduler re-implemented natively with direct capability-table access; semantic state exporter as read-only kernel capability. | The K1 voice flagship demo runs natively on the microkernel (no Linux on the AI path) with E2E ≤ 850 ms. |
| **K2-M4: Legacy Driver VM** | 107–114 | Minimal VMM hosting Linux as a driver guest: display, touch, modem, Wi-Fi, camera passthrough via capability-scoped virtual devices; paravirt drivers where beneficial. | Full hardware functional with Linux confined to the VM; microkernel TCB ≤ 50 KLoC (audited). |
| **K2-M5: Gateway & Runtime on Microkernel** | 115–122 | Semantic Gateway and AI runtime as microkernel userspace servers on capability IPC; TEE attestation chain re-established across the new boot path. | Gate K1 scenario suite re-runs 100% green on the microkernel. |
| **K2-M6: Dogfood & Microkernel Release** | 123–134 | Performance pass (IPC zero-copy fast paths), 72-hour stress, 25-device dogfood, docs. | All §8 gates pass on the microkernel build; Linux VM present only for non-critical drivers. |

### 🔒 GATE K2 (enters Phase K3)
A formal or semi-formal argument (model-checked where possible) demonstrates: (a) a compromised AI runtime cannot invoke a capability it does not hold; (b) a compromised Linux driver VM cannot access AI memory objects or gateway policy state; (c) capability revocation atomically invalidates all descendants; (d) the microkernel TCB is < 50 KLoC with no AI inference code, no network stack, no filesystem.

---

## 7.4 PHASE K3 — Hardware Enablement & Mainline Completion (Sprints 135–186 | ~2 years)
**Phase Objective:** Remove the legacy Linux VM from every AI-, display-, security-, and power-critical path. Prove portability on a second reference device.

### Milestones

| Milestone | Sprints | Scope | DoD |
| :--- | :--- | :--- | :--- |
| **K3-M1: Native Display & Input** | 135–142 | Native microkernel display/audio/input drivers (DRM/KMS-style model, Rust); compositor runs natively. | Display/touch/audio path fully off the Linux VM; measured latency ≤ K2 baseline +5%. |
| **K3-M2: Native NPU/GPU Path** | 143–152 | Native Hexagon NPU driver (or documented minimal FastRPC shim behind a capability device — FS-D9/FlexServe isolation model); Adreno via Freedreno/Turnip or native Rust driver; DMA fabric on microkernel device capabilities. | Inference throughput ≥ Stage A baseline on NPU path; DMA isolation attack suite fails to leak tensors. |
| **K3-M3: Native Power, Modem & Radio** | 153–160 | Power management (CPUFreq/idle/thermal) natively on the microkernel; modem/Wi-Fi via documented vendor interfaces isolated as capability devices. | 48-hour carrier soak passes; thermal policy enforced natively. |
| **K3-M4: Second Reference Device** | 161–174 | Port to an ARMv9-A device with Arm CCA: Realm-world inference (upgrading the TEE model from TrustZone to CCA where available); device tree, drivers, bring-up per the same mainline-first policy. | Second device boots Ind OS, runs the flagship demo, and (where CCA is available) demonstrates Realm-isolated inference. |
| **K3-M5: Linux VM Retirement & Release** | 175–186 | Remaining non-critical drivers ported or documented; Linux VM removed from default builds (kept as optional debug target); porting guide + certification-path docs. | Moto G60 and second device both boot with **no Linux VM on any critical path**; all §8 gates pass on both. |

### 🔒 GATE K3
Both reference devices boot Ind OS with no Android userspace, no AOSP framework, and no Linux VM on the AI-critical path. The AI runtime is TEE/CCA-isolated, the gateway is capability-mediated, and every driver on the display/audio/NPU path is natively implemented or documented and patch-portable.

---

## 7.5 PHASE K4 — Formal Verification & High-Assurance Certification (Sprints 187–238 | ~3 years)
**Phase Objective:** Move from "architecturally sound" to "formally verified where it matters," with an external audit trail.

### Milestones

| Milestone | Sprints | Scope | DoD |
| :--- | :--- | :--- | :--- |
| **K4-M1: Intent ABI & Gateway Proofs** | 187–198 | Intent ABI formalized in Coq/Isabelle/F*; machine-checked proof that the gateway implements the spec (mediation, tier computation, taint rules). | Proof scripts compile in CI; any gateway code change that breaks a proof blocks merge. |
| **K4-M2: Capability & Microkernel Proofs** | 199–212 | Capability derivation/revocation proofs extended to AI workload objects; build on seL4 proofs where applicable; functional correctness of the Rust capability core (or Ferrocene-qualified toolchain where applicable). | Proof coverage published; TCB verification report public. |
| **K4-M3: Information-Flow Proofs** | 213–222 | Machine-checked non-interference-style proof: no path allows `UNTRUSTED_SCREEN_TEXT` to invoke a RED capability; taint engine verified against the rule table. | Proof + 10⁹-op adversarial test campaign, 0 violations. |
| **K4-M4: Certification** | 223–238 | Common Criteria (or equivalent) evaluation of the TCB; reproducible builds end-to-end; transparency log audited by an external party. | Certification package **public**; Gate K4 passed. |

### 🔒 GATE K4
An external auditor can verify, from published proofs and published binary hashes, that the running gateway satisfies the Intent ABI specification and that microkernel isolation properties hold. The certification package is public.

---

## 7.6 PHASE K5 — Ecosystem & Long-Term Evolution (Sprint 239+ | Ongoing)
**Phase Objective:** A self-sustaining open-source ecosystem that survives independence from the founding team.

### Milestones

| Milestone | Scope | DoD |
| :--- | :--- | :--- |
| **K5-M1: Ind SDK 1.0** | Capability-native app SDK + simulator enforcing the Intent ABI and capability model; docs, templates, reference apps. | Third-party dev builds a capability-native app against the simulator in < 1 day. |
| **K5-M2: Community Hardware Program** | postmarketOS-modelled porting program with mainline-first policy and a certification path for device ports; community CI fleet. | First community-maintained port lands with all §8 gates. |
| **K5-M3: Federated Model Registry** | Signed model artifacts, capability-scoped update rules, public transparency log, rollback UI. | Community model ships through the registry with capability-inheritance enforced. |
| **K5-M4: Governance** | Intent ABI and capability model under a public process (RFCs, review board with community seats); trademark and release-key policy. | An ABI change ships through the public process with the founding team in a minority position. |

### 🔒 GATE K5
The OS can be forked, rebuilt, and verified by a community independent of the original team. The Intent ABI and capability model are governed publicly, not by private roadmap.

---

## 7.7 Phase Gates Summary

| Gate | Enters | Hard Requirement (abbreviated — full text in phase sections) |
| :--- | :--- | :--- |
| K0 | K1 | Independent reviewer can derive the compromise answer from specs; TEE prototype defeats all demonstrated read vectors; specs public + machine-checkable |
| K1 | K2 | Flagship voice demo on zero-Android stack; injection → DENY_AND_AUDIT; weights/KV absent from normal world; transparent-log attestation verified independently |
| K2 | K3 | Model-checked isolation arguments; compromised runtime/VM containment; TCB < 50 KLoC |
| K3 | K4 | Two devices, no Linux VM on critical paths, native/documented drivers |
| K4 | K5 | Public proofs + public certification package |
| K5 | — | Independent community can fork, rebuild, verify, govern |

## 7.8 Governance & Amendment Rules

1. **[PRD-AMENDMENT]** PRs with explicit Lead-Architect sign-off are the only path to change §2–§6 (principles, decisions, architecture, ABI, security model).
2. **Intent ABI changes** additionally require the public RFC process from K5-M4 once live; before that, they require a published migration plan with ≥ 2 minor ABI versions of overlap.
3. **Security invariants are CI-enforced**: any change that weakens the air-gap, lowers a speaker-verify threshold, relaxes the action matrix, or enlarges the TCB is automatically rejected and flagged as a security incident.
4. **Models are artifacts, not code**: GGUF/bin files ship via the transparency-logged registry only, SHA-256 pinned; never committed to git.
5. **The agent executes, never architects**: unresolved choices produce `## OPEN-QUESTIONS` entries, not guesses (FS-D12).

---

# PART VIII — PRODUCTION QUALITY GATES (ALL RELEASES)

Every release — dogfood, phase release, or stable tag — must pass all gates on the target device. MVP invariants are preserved where they still apply; substrate-specific rows replace those that cannot.

| # | Metric | Threshold | Verification |
| :--- | :--- | :--- | :--- |
| Q1 | Idle OS RAM (post-boot, 5 min) | ≤ 1.45 GB / 6.0 GB (Stage A); ≤ 1.2 GB (Stage B target) | `dumpsys` equivalent / meminfo |
| Q2 | LLM throughput (1.5B Q4_K_M) | ≥ 15 tok/s CPU fallback; ≥ 25 tok/s accelerated path | benchmark log |
| Q3 | E2E voice command latency | ≤ 850 ms | audio trigger vs. action-executed timestamp |
| Q4 | Intent ABI compliance | 100% valid declarations in 500-prompt En/Hi/Hinglish suite | ABI validator |
| Q5 | AI runtime isolation | 0 ambient capabilities at rest; task-scoped only; 0 network capabilities | capability audit log |
| Q6 | Inference memory protection | Normal-world read of weights/KV fails on every vector of the attack script | automated attack suite (Sprint 27 harness, maintained forever) |
| Q7 | Idle wake-word battery drain | ≤ 1.5%/hour | overnight batterystats |
| Q8 | Injection defence | 0 actions from the full injection corpus; 100% DENY_AND_AUDIT | red-team corpus |
| Q9 | Boot integrity | Measurements verify against transparency log with independent verifier | verifier tool |
| Q10 | Thermal | No runaway; ≥ 80% peak clocks sustained 10-min generation | thermal log |
| Q11 | TCB size | Stage A < 100 KLoC; Stage B < 50 KLoC | cloc audit in CI |
| Q12 | Reproducibility | Bit-identical builds on two machines | CI build-manifest comparison |

---

# PART IX — TEAM, RESOURCES & RISKS

## 9.1 Team Sizing (minimum core, per phase)

| Phase | Duration | Core Team | Specialised Skills |
| :--- | :--- | :--- | :--- |
| K0 | 6 months | 8–12 | ARM64/Qualcomm bring-up, OP-TEE/TrustZone, Rust systems, formal methods (TLA+), Wayland, inference runtime |
| K1 | 2 years | 15–25 | sched_ext/BPF, LSM, memory management, Wayland compositor, speech/AI pipeline, security policy, telephony/radio |
| K2 | 2 years | 20–35 | Microkernel (seL4 lineage), Rust kernel, formal verification, device drivers, virtualization |
| K3 | 2 years | 25–40 | Mainline driver dev (DRM/ASoC/power/NPU), CCA/Realm bring-up, second-platform porting |
| K4 | 3 years | 15–25 + academic partners | Coq/Isabelle/F*, Common Criteria, reproducible builds, external audit |
| K5 | Ongoing | 10–20 core + community | SDK, developer relations, governance |

## 9.2 Top Risks & Pre-Locked Mitigations

| Risk | Likelihood | Impact | Mitigation |
| :--- | :--- | :--- | :--- |
| Hexagon NPU firmware stays closed | Medium | Medium (perf, not security) | FS-D9 isolation: NPU as capability-scoped device with SMMU-restricted memory and gateway-validated outputs; GPU-first fallback (Sprint 26 routing) |
| Microkernel verification takes longer than planned | High | Schedule | Build on seL4's existing proofs (K2-M1); accept K4 slippage — gates, not dates, govern |
| Mainline mobile hardware gaps (modem/camera/power) | High | Medium | Stage A tolerates imperfection; documented patch sets (FS-D9); vendor interfaces isolated as capability devices |
| TEE inference overhead erodes the 850 ms budget | Medium | High | Sprint 28 measures it early; continuous voice-path profiling from M5 onward; admission control (M3) protects the interactive class |
| Agent drift in kernel code | Medium | Critical | FS-D12: spec-first, model-checked DoDs, amendment-only changes; Q11 TCB audit in CI |
| Community fragmentation at K5 | Low | Medium | Public ABI governance (K5-M4) + independent reproducible verification path (K1-M10) |

---

# PART X — FINAL WORD

The MVP proved the promises to users. This document builds the architecture that keeps those promises when the attacker is no longer a prompt but a kernel exploit; when the question is no longer "can the AI do it" but "can anything the AI does hurt you without your hand on it."

> **Specify the interface. Verify the mechanism. Isolate the intelligence. Trust the proof. Ship for the user who cannot audit the code but deserves an OS that can be audited on their behalf.**

An AI-native operating system is not one where the assistant is smarter. It is one where the **architecture makes the assistant's mistakes survivable.** Every sprint, milestone, gate, and capability in Part VII is in service of that standard.

---

## Appendix A — MVP → Kernel-Track Migration Map (quick reference)

| MVP Component (shipped) | Final Resting Place |
| :--- | :--- |
| `IndNeuralCore` / `indaid` | Layer 5 AI Runtime (TEE/CCA) + Layer 4 kernel AI primitives |
| `ActionPolicyEnforcer.java` | Layer 6 Semantic Gateway hardcoded policy engine (formally specified) |
| Accessibility graph pruner | Layer 4 semantic state exporter (native, kernel-mediated) |
| `ind_ai_daemon.te` SELinux air-gap | Stage A: defence-in-depth under the capability LSM; Stage B: capability model |
| Green/Amber/Red matrix | Capability-attenuation derivation predicates (TLA+-proven) |
| GBNF grammar | Constrained-decoding implementation detail of the AI runtime |
| Voice cascade + models | Layer 7/5 voice pipeline, carried models as signed artifacts |
| `IndLauncher` + cards + orb | Layer 7 semantic shell (pure ABI consumer) |
| Suraksha Shield (OTP blackout, scam guard) | Layer 6/7: compositor-native trust levels + gateway invariants + signed classifiers |
| microG / Play Integrity bridge | Measured boot + transparency log + user-verifiable attestation |
| Deploy overlay harness | Simulator + QEMU + fastboot iteration loop |

## Appendix B — Amendments Log

| ID | Version | Date | Decision | Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **U-001** | v1.0.1-BIBLE + FS v1.0 → **v2.0 FINAL** | 2026-10-26 | Unified both predecessor PRDs into one governing document. MVP track closed as shipped and moved to maintenance; kernel track begins at Sprint 19. All FS-D1–FS-D12 decisions locked; MVP D1–D10 dispositions final per §3.1. | Single source of truth; eliminates the two-document ambiguity; sprints continue seamlessly from the completed 18. |
| **A-002** | v2.0.0 → **v2.1.0** (SBA-001) | 2026-10-26 | **Split-Brain Architecture adopted (SBA-001).** Kernel = Policy Plane (verified selectors only). TEE = Inference Plane (full AI runtime). Capability microkernel = enforcement boundary. Intent ABI frozen at v1.0. Information-flow labels mandatory. No model weights in kernel space, ever. | Synthesises AKTS (sub-µs policy switching), FlexServe/TZ-LLM (TEE inference viability), Agentic-seL4 (capability containment), and LUMOS (semantic state) into a single enforceable architecture. Resolves the "AI in kernel vs. AI outside kernel" debate: the answer is both, in different planes, with a verified boundary. |

*Document ends. The MVP is shipped. The kernel work begins — Sprint 19.*

