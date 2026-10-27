# Touch Controller Driver Specification (SP-78)

- **Sprint:** SP-78 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** Touch Digitizer Controller & Multi-Touch Coordinate Parsing
- **Document ID:** SPEC-ARCH-SP78-TOUCH-CONTROLLER
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, REQ-HW-107, REQ-UX-102

---

## 1. Objectives & Scope

Sprint SP-78 implements the hardware driver and event parser for the capacitive touch digitizer on the Motorola Moto G60. The controller interfaces over Qualcomm GENI I2C bus 1 (mapped at MMIO `0x0880_0000` with 7-bit slave address `0x38`). It processes multi-touch point packets, clamps physical digitizer coordinates to the display viewport (1080 x 2460), filters electrical contact noise, and pushes normalized events into an interrupt-safe event ring buffer.

---

## 2. Architectural Invariants

### INV-TOUCH-001: GENI I2C Touch Controller Address Mapping
The touch controller bus interface is mapped at MMIO base `0x0880_0000` with standard 400kHz Fast-mode I2C clocking and device address `0x38`.

### INV-TOUCH-002: Multi-Touch Coordinate Bounding & Normalization
Incoming multi-finger coordinates `(x, y)` are validated and clamped: `x < 1080` and `y < 2460`. Finger pressure/contact area values exceeding sensor thresholds are filtered.

### INV-TOUCH-003: Interrupt-Safe Bounded Event Queue
Touch packets are posted into a lock-free or spinlock-protected FIFO ring buffer (capacity 32 events). To prevent CPU starvation and priority inversion in IRQ context, buffer overflow drops stale events with an explicit overflow metric rather than blocking.

---

## 3. Interfaces & Data Structures

- `TouchPoint`: Struct holding `id`, `x`, `y`, `pressure`, and `action` (`Down`, `Move`, `Up`).
- `TouchControllerHarness`: Userspace driver modeling I2C communication and coordinate parsing.
- `KernelTouchControllerHarness`: In-kernel HAL platform driver executing hardware checks.
