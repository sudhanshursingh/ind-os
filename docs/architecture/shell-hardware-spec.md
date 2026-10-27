# Ind OS SP-100 Architecture Specification
## Shell v2 on Physical Display and Touch Integration

**Sprint:** SP-100 (Stage 450–454)  
**Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-106, REQ-UX-102  
**Target Hardware:** Motorola Moto G60 (Qualcomm Snapdragon 732G / SM7150-AC, Tianma FT8726 DSI panel, FocalTech FT3518 I2C touch)  
**Status:** APPROVED FOR IMPLEMENTATION

---

### 1. Purpose and Scope

Sprint SP-100 integrates the Phase 8 Shell v2 system with the physical display and touch hardware drivers brought up in SP-76..79. It binds the compositor surface pipeline directly to the hardware DSI zero-copy framebuffer swapchain (1080x2460 @ 120Hz) and multiplexes low-latency touch inputs into user-interface gestures (Tap, Drag, Fling, Multi-touch release) with sub-16ms latency.

---

### 2. Architectural Invariants

- **INV-SHELL-HW-001 (Direct DSI Swapchain Binding):**  
  The Shell v2 compositor must bind to the hardware display surface with capability token `CAP_DISPLAY_RENDER`. Double-buffered frame presentation executes without heap churn, updating back/front buffer indices deterministically.

- **INV-SHELL-HW-002 (Deterministic Touch-to-Gesture Dispatch):**  
  Touch coordinates from the physical I2C touch controller are transformed via fixed-point calibration into UI events within <= 16ms (1 frame at 60Hz / 2 frames at 120Hz). Dynamic heap allocation is strictly forbidden.

- **INV-SHELL-HW-003 (Hardware Shell Chrome Presentation):**  
  Shell chrome layout maps the 1080x2460 physical screen into dedicated hardware zones:
  - Status Bar: 1080x64 pixels at top (Battery %, Wi-Fi RSSI, Clock, AIS state indicator).
  - Main Surface / Intent Canvas: 1080x2268 pixels.
  - Navigation / Gesture Bar: 1080x128 pixels at bottom.

---

### 3. Capability Security & Attenuation

- `CAP_DISPLAY_RENDER`: Required to push framebuffers to the physical DSI scanout hardware.
- `CAP_TOUCH_INPUT`: Required to receive raw touch coordinates from the kernel touch driver.
- Spoofed or unprivileged processes attempting to render to scanout or inject touch inputs fail with `ShellHwError::PermissionDenied`.

---

### 4. Zero Unsafe & No External Crates Compliance

- Userspace implementation in `userspace/ux/src/shell_hw.rs` contains exactly zero unsafe blocks (Rule N-04).
- Kernel HAL driver in `kernel/src/hal/platform/shell_hw_core.rs` contains zero external crates (Rule N-01).
