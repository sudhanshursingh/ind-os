# Compositor & Display Server Specification (SP-60)

**Standard:** Ind OS Development Bible §6.3, §8.14, Table 19.2  
**Status:** **RATIFIED**  
**Phase:** Phase 8 (SP-60)

---

## 1. Architectural Purpose & Scope

The **Ind OS Compositor** (`uxd`) is a capability-fed, user-mode display server responsible for:
1. Window surface management with capability-mediated access tokens (`CAP_DISPLAY_RENDER`, `CAP_DISPLAY_OVERLAY`).
2. Deterministic damage tracking and zero-heap dirty rect clipping for 60fps/120fps display rendering.
3. Anti-overlay protection (`INV-COMP-003`): Non-bypassable isolation ensuring system consent surfaces (Consent Broker V2) render at the topmost unforgeable z-order, preventing overlay attacks, tapjacking, and UI spoofing.
4. Focus integrity and strict input routing (`INV-COMP-004`): Hardware pointer and touch events are dispatched strictly to authorized top surfaces.

---

## 2. Architectural Invariants

- **INV-COMP-001 (Capability-Gated Display Authority):** Creating surfaces and presenting framebuffers requires explicit `CAP_DISPLAY_RENDER`. Creating top-level system overlays requires `CAP_DISPLAY_OVERLAY`, which is strictly denied to third-party user applications.
- **INV-COMP-002 (Zero-Heap Damage Tracking):** Damage tracking uses bounded, pre-allocated dirty rectangle pools, eliminating dynamic heap allocations in the vsync rendering fast path.
- **INV-COMP-003 (Anti-Overlay Consent Isolation):** System consent surfaces possess unconditional topmost z-order priority. Application surfaces cannot render above or occlude consent cards.
- **INV-COMP-004 (Focus & Input Event Integrity):** Input events are dispatched strictly to the focused window. Occluded or background windows cannot capture ambient input events.
