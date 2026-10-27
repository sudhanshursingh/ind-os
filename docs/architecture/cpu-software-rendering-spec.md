# CPU Software Rendering & Compositor Specification (SP-77)

- **Sprint:** SP-77 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** CPU Software Rendering Compositor & Double-Buffered Frame Presentation
- **Document ID:** SPEC-ARCH-SP77-CPU-COMPOSITOR
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, PEN-004, REQ-UX-101

---

## 1. Objectives & Scope

Sprint SP-77 implements the CPU software rendering compositor pipeline per PEN-004. By bypassing closed-source GPU driver stacks, Ind OS renders all UI components (system bars, cards, touch feedback) in software, blits damage rectangles into a double-buffered frame pipeline, and flips buffers synchronously with VSYNC timing to drive the 120Hz Moto G60 display cleanly.

---

## 2. Architectural Invariants

### INV-COMP-001: Double-Buffered Frame Architecture
The compositor maintains two memory-mapped frame buffers (Front Buffer for active scanout, Back Buffer for CPU composition). Buffer flip swaps active pointers atomically under a spinlock.

### INV-COMP-002: Dirty Rectangle Bounding & Validation
Blit operations require an explicit dirty rectangle `(x, y, w, h)`. The compositor clamps and verifies that `x + w <= PANEL_WIDTH` (1080) and `y + h <= PANEL_HEIGHT` (2460). Malformed bounds are strictly rejected before memory copies.

### INV-COMP-003: Capability-Gated Frame Presentation
Frame submission is guarded by `CAP_SURFACE_COMPOSITOR`. Only verified system shell components can flip the display surface, enforcing anti-overlay and anti-tapjacking integrity directly at the scanout level.

---

## 3. Interfaces & Data Structures

- `DirtyRect`: Represents an axis-aligned bounding box of changed pixels.
- `CpuCompositorHarness`: Userspace driver modeling buffer swaps, damage tracking, and presentation stats.
- `KernelCpuCompositorHarness`: In-kernel implementation validating buffer bounds and capability enforcement.
