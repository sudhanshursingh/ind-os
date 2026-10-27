# Ind OS Architecture Specification: Bluetooth Controller & HCI UART (Sprint SP-88)

**Document ID:** SPEC-ARCH-SP88-BLUETOOTH-HCI  
**Status:** Approved  
**Target:** Qualcomm Bluetooth 5.0 / HCI UART Transport Bring-Up  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-109, REQ-NET-102  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-BT-001 (HCI UART Transport MMIO):** The Qualcomm GENI UART dedicated to Bluetooth HCI is mapped at physical base `0x0884_0000`, operating at 3,000,000 baud with CTS/RTS hardware flow control.
- **INV-BT-002 (Firmware NVM Patch Load):** Bluetooth controller initialization requires vendor NVM firmware patch transfer via the isolated §14.3 loader (`0x8920_0000`) before entering operational mode.
- **INV-BT-003 (HCI Command Loop & BLE Scan):** Deterministic processing of standard HCI commands (`HCI_Reset`, `HCI_LE_Set_Scan_Enable`) and BLE advertisement reporting.

---

### 2. Constants & Data Types

| Constant | Value | Description |
|---|---|---|
| `HANOIP_BT_UART_BASE` | `0x0884_0000` | Qualcomm GENI UART BT Base |
| `HCI_CMD_RESET` | `0x0C03` | Standard HCI Command: Reset |
| `HCI_CMD_LE_SET_SCAN_ENABLE` | `0x200C` | Standard HCI LE Command: Scan Enable |
| `CAP_BT_ADMIN` | `0x0400_0000` | Capability bit for Bluetooth device management |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Pure in-tree Rust, zero external crates in TCB.
2. **Rule N-04:** Userspace driver `userspace/devmgr/src/bluetooth_hci.rs` is 100% safe Rust.
3. **Rule N-06 / PRD-001 §7.5:** Malformed HCI packets from radio baseband are safely dropped without affecting kernel integrity.
