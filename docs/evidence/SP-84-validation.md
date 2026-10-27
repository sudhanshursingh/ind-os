# Sprint SP-84 Acceptance & Validation Report
## Qualcomm USB 3.0 / DWC3 Controller & QUSB2 PHY Bring-Up

**Milestone Context:** Phase 9 (Moto G60 Bring-Up) — Sprint SP-84  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-108, REQ-USB-101  
**Target Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants Validation

| Invariant ID | Requirement | Validation Method | Result |
|---|---|---|---|
| **INV-USB-001** | Qualcomm DWC3 Controller MMIO base address mapping (`0x04e0_0000`) & PHY (`0x088e_2000`) | Verified strict MMIO base boundary matching (`0x04e0_0000`, `0x088e_2000`) and rejection of invalid addresses | **PASSED** |
| **INV-USB-002** | QUSB2 PHY PLL lock and reset sequencing | Verified state progression to PLL lock, HighSpeed 480Mbps link initialization, and EP0 ready state | **PASSED** |
| **INV-USB-003** | Control Endpoint 0 setup packet processing and descriptor handshake | Handled standard USB 2.0 `GET_DESCRIPTOR` (18B Device Descriptor) and `SET_ADDRESS` (Addr 20) requests | **PASSED** |

---

### 2. Verified Boot Sequence Stages (Stages 370–374)

```text
[STAGE:370_SP84_USB_DWC3_INIT] Qualcomm USB 3.0 DWC3 controller and QUSB2 PHY harness initialized
[INFO ] usb: USB DWC3 harness online: Synopsys DWC3 core and QUSB2 PHY auditor active (REQ-USB-101 PASSED)
[STAGE:371_SP84_USB_DWC3_CONTROLLER_MAPPING] DWC3 hardware register MMIO base mapping verified
[INFO ] usb: DWC3 verified: USB 3.0 controller mapped at 0x04e0_0000, QUSB2 PHY at 0x088e_2000 (INV-USB-001 PASSED)
[STAGE:372_SP84_QUSB2_PHY_PLL_LOCK] QUSB2 PHY initialization and PLL lock sequencing verified
[INFO ] usb: PHY verified: QUSB2 PLL locked, HighSpeed 480Mbps link established (INV-USB-002 PASSED)
[STAGE:373_SP84_CONTROL_ENDPOINT_EP0_HANDSHAKE] Control Endpoint 0 setup packet handling and device descriptor verified
[INFO ] usb: EP0 verified: standard GET_DESCRIPTOR (18B) and SET_ADDRESS (Addr 20) handled deterministically (INV-USB-003 PASSED)
[STAGE:374_SP84_USB_DWC3_READY] Sprint SP-84 USB DWC3 Controller Bring-Up Milestone REACHED
--------------------------------------------------
 [SP-84] Sprint SP-84 USB DWC3 Controller Milestone REACHED.
 [SP-84] - DWC3 Controller Mapped at 0x04e0_0000 (INV-USB-001, REQ-USB-101),
 [SP-84] - QUSB2 High-Speed PHY PLL Locked at 480 Mbps (INV-USB-002),
 [SP-84] - Control Endpoint 0 Setup Packet & Descriptors Verified (INV-USB-003).
==================================================
```

---

### 3. Compliance Summary
- **Rule N-01:** Zero external crates in TCB; pure in-tree Rust.
- **Rule N-04:** Userspace driver `userspace/devmgr/src/usb_dwc3.rs` is 100% safe Rust.
- **Rule N-06 / PRD-001 §7.5:** USB interface respects recovery boundaries and descriptor sanitization.
