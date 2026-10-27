# Ind OS Architecture Specification: USB Device/Host Mode & CDC ACM Serial Console (Sprint SP-85)

**Document ID:** SPEC-ARCH-SP85-USB-ROLE  
**Status:** Approved  
**Target:** USB Dual-Role (Device/Host/OTG) & CDC ACM Serial Console Bring-Up  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-108, REQ-USB-102, REQ-REC-101  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-USB-004 (OTG ID Pin Sensing & Role Switching):** USB Type-C CC line detection and OTG ID pin sensing determine whether the port operates in Device mode (peripheral) or Host mode. Default boot mode is Device mode.
- **INV-USB-005 (CDC ACM Serial Console):** Implements USB Communications Device Class Abstract Control Model (CDC ACM) with bulk IN/OUT endpoints for transparent recovery console, diagnostic logging, and fastboot stub fallback (Rule N-06 / PRD-001 §7.5).
- **INV-USB-006 (Role Switch Capability Gating):** Dynamic switching between Device and Host modes requires explicit possession of `CAP_USB_ROLE_SWITCH` (`0x0080_0000`). Unprivileged switching requests are rejected.

---

### 2. Constants & Data Types

| Constant | Value | Description |
|---|---|---|
| `USB_ROLE_DEVICE` | `1` | USB Peripheral / Device Mode |
| `USB_ROLE_HOST` | `2` | USB Host / OTG Mode |
| `CAP_USB_ROLE_SWITCH` | `0x0080_0000` | Capability bit for switching USB roles |
| `CDC_ACM_BAUD_DEFAULT` | `115200` | CDC ACM default communication baud rate |
| `CDC_ACM_LINE_CODING_LEN` | `7` | Standard CDC Line Coding packet size (7 bytes) |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Zero external crates in TCB; pure in-tree Rust.
2. **Rule N-04:** Userspace driver `userspace/devmgr/src/usb_role.rs` is 100% safe Rust.
3. **Rule N-06:** The CDC ACM console serves as an isolated recovery transport that does not rely on persistent filesystem health.
