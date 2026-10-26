# Technical RFC: Non-Disruptive Architectural Extensions (Post-SP-11 Baseline)

**Document ID:** `RFC-INDOS-2026-001`  
**Status:** PROPOSED FOR PHASE 4 & PHASE 6 PLANNING  
**Baseline Alignment:** INDOS-BIBLE-001 (v1.0.0.0) | Sprints SP-00..SP-11 Completed on `main`  
**Scope:** Forward-looking architectural enhancements; ZERO disruption to Phase 2/3 core kernel tasks.  

---

### Executive Summary

With 12 sprints (SP-00 through SP-11) successfully merged into GitHub `main`, `IndK` has achieved stable serial boot (Gate G1) and user-mode address-space isolation. To maintain strict alignment with non-negotiable governance rules (N-01, N-07) while preparing for flagship hardware scaling and non-mobile reusability, this RFC proposes two additive enhancements scheduled for Phase 4 and Phase 6.

---

### Proposal 1: `REQ-AI-112` — Dynamic Hardware Profile Manager (`IndHardwareProfile`)

* **Target Phase:** Phase 6 (Sprint SP-41 — AIS Model Runtime Integration)
* **Problem Statement:** Pushing a single static GGUF model configuration across devices causes RAM strain on 6 GB budget phones while underutilizing compute on 12 GB+ flagship hardware.
* **Proposed Specification:**
  1. Introduce an `IndHardwareProfile` daemon inside `userspace/ais/` that executes during early AIS initialization.
  2. The daemon queries physical RAM via read-only kernel memory telemetry (`REQ-KRN-113`) and dynamically configures model parameters:
     * **6 GB Budget Tier:** Enforces `Q4_K_M` GGUF quantization, 1.5B parameters (e.g., Qwen-2.5-1.5B), 2K context window, 128 MB KV-cache allocation (Peak RAM $\le 1,250$ MB).
     * **12 GB+ Flagship Tier:** Expands to `Q8_0` or 7B/8B parameter GGUF models, 8K context window, and multi-threaded CPU/GPU Vulkan acceleration.
* **Non-Disruption Guarantee:** The kernel (`IndK`) and Enforcement Layer (`ENF`) remain 100% untouched. All logic lives strictly inside the unprivileged AIS user-space runtime.

---

### Proposal 2: `REQ-ARCH-101` — Standalone `indk-core` Workspace Decoupling

* **Target Phase:** Phase 4 (Sprint SP-31 — Hardware Abstraction Layer & Refactoring)
* **Problem Statement:** Monolithic repository directory layouts can couple kernel code with mobile-specific userspace assumptions, limiting `IndK` reusability for robotics, IoT, or edge computing.
* **Proposed Specification:**
  1. Restructure the `/kernel` directory into an independent, freestanding Cargo workspace crate named `indk-core` (`aarch64-unknown-none`, `no_std`).
  2. Maintain strict separation so `indk-core` compiles standalone without requiring `IndUS` (mobile userspace) or `AIS` (AI subsystem).
* **Non-Disruption Guarantee:** Refactoring occurs entirely within Sprint SP-31 during the scheduled HAL skeleton build-out, requiring no changes to current SP-12 through SP-23 sprint contracts.

---

### Governance & Traceability

* **Change Control Class:** `CR-SEMANTIC` (Adds `REQ-AI-112` and `REQ-ARCH-101`; MINOR version bump to BIBLE v1.1.0.0 upon Phase 4/6 execution).
* **N-07 Compliance:** This proposal introduces no scope creep for active Phase 2 (SP-12..SP-15) or Phase 3 (SP-16..SP-23) workstreams.
