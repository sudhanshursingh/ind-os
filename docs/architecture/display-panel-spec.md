# Display Panel Bring-Up Specification (SP-76)

- **Sprint:** SP-76 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** Qualcomm MDSS DSI Host Controller & 120Hz IPS LCD Panel Initialization
- **Document ID:** SPEC-ARCH-SP76-DISPLAY-PANEL
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, PEN-004, REQ-HW-106

---

## 1. Objectives & Scope

Sprint SP-76 specifies and implements the display panel driver for the Motorola Moto G60. The hardware comprises Qualcomm's Mobile Display Subsystem (MDSS) MIPI DSI 4-lane controller mapped at physical address `0x0ae9_4000` driving a 1080 x 2460 120Hz IPS LCD panel. In accordance with PEN-004, display bring-up operates independently of proprietary GPU firmware by utilizing CPU software rendering direct to the scanout buffer.

---

## 2. Architectural Invariants

### INV-DISP-001: MDSS DSI MMIO Base Mapping
The DSI host controller hardware register interface is memory-mapped at physical address `0x0ae9_4000` with strict 4KB page alignment and device uncached memory attributes.

### INV-DISP-002: MIPI DCS Panel Initialization Sequence
The panel initialization follows the MIPI Display Command Set (DCS) standard:
- DCS `0x11` (Exit Sleep Mode / Sleep Out), followed by 120ms guard delay.
- DCS `0x3A` (Set Pixel Format: 24bpp RGB888).
- DCS `0x29` (Set Display On).

### INV-DISP-003: Backlight PWM Duty Cycle Bounding
Backlight control accepts duty cycles bounded strictly in `[0, 100]`. Boot default brightness is set to 30% to conserve thermal budget and prevent power spikes during cold boot.

---

## 3. Interfaces & Data Structures

- `DisplayPanelHarness`: Userspace safe driver modeling DSI state, panel mode, and backlight.
- `KernelDisplayPanelHarness`: In-kernel HAL platform driver implementing hardware verification.
