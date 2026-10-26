# IndOS Deterministic Memory Pressure, Reclamation & OOM Architecture Specification

**Document Identifier:** `INDOS-SPEC-OOM-01`  
**Applicable Sprints:** Sprint SP-29 (Phase 4), Bible §8.10  
**Status:** Approved & Formally Specified  
**Classification:** Core Kernel Architecture & Resilience Spine  
**Dependencies:** `INDOS-SPEC-MEM-01`, `INDOS-SPEC-SCHED-01`

---

## 1. Executive Summary & Rationale

IndOS is designed for mission-critical, AI-integrated edge and personal computing. Under constrained RAM topologies (ranging from 4 GB to 16 GB), unexpected memory spikes can destabilize the operating system. Standard desktop kernels often employ heuristic out-of-memory (OOM) heuristics or rely on heuristic swap thrashing, leading to multi-second latency freezes.

Per Bible §8.10 and Line 340, IndOS enforces two non-negotiable architectural mandates:
1. **Swap Decision Spike (`DEF` / `PEN`):** Swap to disk is explicitly deferred/penalized. Non-volatile flash wear, unbounded block I/O latency tails, and priority inversions make swap unacceptable for real-time and interactive guarantees. Memory elasticity is achieved solely via structured multi-tier cache reclamation and deterministic OOM termination.
2. **Deterministic OOM Policy (`INV-OOM-001`):** The OOM killer is **strictly algorithmic, deterministic, and NEVER model-controlled**. While userspace AI services (such as AIS in SP-47) may provide advisory hints regarding prefetching or cache sizing, the kernel OOM killer operates independently under formal invariants, guaranteeing zero termination of real-time and system-critical tasks.

---

## 2. Machine Invariants

- **`INV-OOM-001` (Deterministic Non-Model-Controlled OOM):**
  The kernel OOM victim selection algorithm must be 100% deterministic, strictly algorithmic, and immune to userspace or AI model manipulation. Tasks possessing `Priority::Realtime`, core system identity (`pid <= 1`), or explicit kernel immunity must NEVER be terminated by the OOM killer under any condition.
- **`INV-OOM-002` (Multi-Tier Monotonic Pressure Transitions):**
  Memory pressure transitions must follow strict monotonic thresholds (`Normal` $\rightarrow$ `Warning` $\rightarrow$ `Critical` $\rightarrow$ `Oom`). The kernel must notify registered reclamation subscribers and execute voluntary/involuntary cache pruning before escalating to process termination.

---

## 3. Multi-Tier Memory Pressure Detection

Memory pressure is computed continuously by comparing free physical frames to the total usable physical frames managed by `PageFrameAllocator`:

$$\text{Free Ratio} = \frac{\text{free\_frames}}{\text{total\_frames}}$$

| Tier | Free Ratio Threshold | Description | Kernel Action |
| :--- | :--- | :--- | :--- |
| **Normal** | $\ge 25.0\%$ | Adequate memory headroom. | No reclamation. Normal allocation fast-path. |
| **Warning** | $10.0\% \le \text{ratio} < 25.0\%$ | Moderate memory constraint. | Voluntary reclamation: clean page caches, stale vnode buffers, and temporary directory lookups are released. |
| **Critical** | $2.0\% \le \text{ratio} < 10.0\%$ | Severe memory exhaustion imminent. | Involuntary aggressive reclamation: shrink slab allocations, purge discardable read buffers, block non-essential allocations. |
| **Oom** | $< 2.0\%$ | Physical memory depleted. | Deterministic OOM killer triggered. Victim selected and terminated. |

---

## 4. Swap Decision Spike Evaluation (`DEF` / `PEN`)

A comprehensive kernel engineering spike was conducted regarding swap-to-disk on mobile/embedded eMMC and NVMe storage:
- **Latency Jitter:** Swapping anonymous memory pages to flash introduces read/write stalls spanning 5 ms to 250 ms. This directly violates the 10 ms real-time audio/telemetry scheduling quantum (`DEFAULT_TIME_QUANTUM_TICKS`).
- **Flash Endurance (TBW):** Constant write-amplification from swap churn causes premature cell degradation on embedded consumer storage.
- **Decision:** **DEFERRED (`DEF`) / PENALIZED (`PEN`)**. IndOS disallows physical swap-to-disk. Instead, IndOS implements:
  1. Multi-tier voluntary and involuntary memory reclamation.
  2. Future in-memory zram/zswap compression hints (SP-47 advisory hints).
  3. Strict, instantaneous, deterministic OOM termination.

---

## 5. Deterministic Victim Selection Algorithm

When memory pressure reaches `Oom` and reclamation fails to recover sufficient headroom, `OomKiller` identifies the candidate with the highest badness score:

### 5.1 Immunity Criteria
A process is completely excluded from OOM consideration if:
1. `pid <= 1`: Core kernel, idle loop, or system init daemon (`ImmunityReason::SystemCritical`).
2. `priority == Priority::Realtime`: Real-time hardware audio, GIC driver, or sensor spine (`ImmunityReason::RealtimePriority`).
3. `is_immune == true`: Explicit kernel-granted resilience immunity (`ImmunityReason::ExplicitImmunity`).

### 5.2 Badness Score Calculation
For all non-immune processes:

$$\text{Base Score} = \frac{\text{allocated\_frames} \times 1000}{\text{total\_frames}}$$

$$\text{Badness Score} = \text{Base Score} + \text{Priority Penalty}$$

Priority penalties enforce class preservation:
- `Priority::Idle`: $+400$
- `Priority::BatchIntelligence`: $+300$ (e.g. background LLM inference, indexing)
- `Priority::Normal`: $+100$ (standard user services)
- `Priority::Interactive`: $+0$ (user-facing UI, shell)

### 5.3 Deterministic Tie-Breaking
If two candidate processes yield identical badness scores:
1. Higher frame consumption is selected first.
2. If frame consumption is equal, higher `pid` is selected first.

This guarantees mathematical reproducibility and zero nondeterminism.

---

## 6. Implementation Deliverables

1. `docs/architecture/memory-pressure-oom-spec.md` — This specification.
2. `kernel/src/mm/pressure.rs` — Multi-tier memory pressure monitor and reclamation subscriber registry.
3. `kernel/src/mm/oom.rs` — Deterministic OOM killer engine enforcing `INV-OOM-001`.
4. `kernel/src/mm/mod.rs` — Re-exports and global singletons `PRESSURE_MANAGER` and `OOM_KILLER`.
5. `kernel/src/main.rs` — Boot verification stages 90–94.
6. `ci/verify_sp29.py` — Automated verification test harness.
