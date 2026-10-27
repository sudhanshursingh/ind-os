# Input Pipeline to Shell Milestone Specification (SP-79)

- **Sprint:** SP-79 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** Input Pipeline to Shell Milestone & Gesture Recognition
- **Document ID:** SPEC-ARCH-SP79-INPUT-PIPELINE
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, REQ-HW-107, REQ-UX-102, REQ-SEC-104

---

## 1. Objectives & Scope

Sprint SP-79 completes the basic input $\to$ shell milestone on the Motorola Moto G60. It bridges the physical touch digitizer driver (SP-78) with the user shell windowing layer, transforming raw digitizer pulses into semantic gestures (`Tap`, `Scroll`, `LongPress`), routing gestures to the focused system surface, and enforcing anti-tapjacking focus verification (`CAP_INPUT_SHELL`).

---

## 2. Architectural Invariants

### INV-INP-001: Coordinate Space Normalization
Touch digitizer coordinates are translated to shell layout coordinates via deterministic affine transformation. For the Moto G60's native 1080 x 2460 panel, this provides a 1:1 pixel-accurate mapping without coordinate drift.

### INV-INP-002: Deterministic Gesture Recognition Engine
The gesture recognizer aggregates touch trajectories:
- Single Down + Up within 250ms and $\Delta < 20\text{px} \implies$ `Gesture::Tap(x, y)`.
- Continuous movement with $\Delta \ge 20\text{px} \implies$ `Gesture::Scroll(dx, dy)`.
- Sustained Down with duration $\ge 500\text{ms} \implies$ `Gesture::LongPress(x, y)`.

### INV-INP-003: Shell Input Routing & Focus Gating
Events are dispatched only to the active top-level surface holding verified focus. Unfocused background applications cannot receive touch coordinates, and routing requires the `CAP_INPUT_SHELL` capability token.

---

## 3. Interfaces & Data Structures

- `Gesture`: Enum representing `Tap`, `Scroll`, `LongPress`.
- `InputPipelineHarness`: Userspace safe driver transforming touch points into gestures and routing to shell surfaces.
- `KernelInputPipelineHarness`: In-kernel HAL platform verifier validating transformation and focus bounds.
