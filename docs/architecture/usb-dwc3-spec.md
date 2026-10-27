# Ind OS Architecture Specification: USB DWC3 Controller & QUSB2 PHY (Sprint SP-84)

**Document ID:** SPEC-ARCH-SP84-USB-DWC3  
**Status:** Approved  
**Target:** Qualcomm USB 3.0 / Synopsys DesignWare Core 3 (DWC3) & QUSB2 PHY Bring-Up  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-108, REQ-USB-101  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-USB-001 (Controller MMIO Base):** The Qualcomm DWC3 USB controller is located at physical MMIO base `0x04e0_0000`. Any attempt to access outside this region is blocked.
- **INV-USB-002 (QUSB2 PHY PLL Lock):** Before USB transaction processing, the QUSB2 PHY PLL must be locked and calibrated with a verified power-on sequence.
- **INV-USB-003 (EP0 Control Handshake):** Control Endpoint 0 (EP0 IN / EP0 OUT) must process standard USB 2.0/3.0 setup packets (e.g. `GET_DESCRIPTOR`, `SET_ADDRESS`) deterministically.

---

### 2. Register Layout & Constants

| Constant | Value | Description |
|---|---|---|
| `HANOIP_USB_DWC3_BASE` | `0x04e0_0000` | Qualcomm DWC3 Controller MMIO Base |
| `HANOIP_QUSB2_PHY_BASE` | `0x088e_2000` | Qualcomm QUSB2 High-Speed PHY Base |
| `USB_REQ_GET_DESCRIPTOR` | `0x06` | Standard USB Request: Get Descriptor |
| `USB_REQ_SET_ADDRESS` | `0x05` | Standard USB Request: Set Address |
| `USB_DESC_TYPE_DEVICE` | `0x01` | USB Descriptor Type: Device |
| `CAP_USB_ADMIN` | `0x0040_0000` | Capability bit for USB controller management |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Pure in-tree Rust, no external dependencies.
2. **Rule N-04:** Userspace driver in `userspace/devmgr/src/usb_dwc3.rs` is 100% safe Rust.
3. **Rule N-06:** Deterministic endpoint buffering prevents memory corruptions on malformed USB setup packets.
