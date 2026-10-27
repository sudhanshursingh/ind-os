# Ind OS Architecture Specification: Qualcomm WCN3990 Wi-Fi Driver (Sprint SP-87)

**Document ID:** SPEC-ARCH-SP87-WIFI-WCN3990  
**Status:** Approved  
**Target:** Qualcomm WCN3990 / ath10k Compatible 802.11 b/g/n/ac Wi-Fi Bring-Up  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-109, REQ-NET-101  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-WIFI-001 (WCN3990 SMD MMIO Mapping):** The Qualcomm WCN3990 WCNSS / SMD communication registers are mapped at physical base `0x0a00_0000`.
- **INV-WIFI-002 (Firmware & RF Calibration):** Wi-Fi hardware initialization requires valid firmware staged via the §14.3 loader (`0x8900_0000`) followed by RF synthesizer calibration.
- **INV-WIFI-003 (802.11 Association State Machine):** Deterministic state machine (`Disabled` $\to$ `Idle` $\to$ `Scanning` $\to$ `Authenticating` $\to$ `Associated`) governed by network management capability gating.

---

### 2. Constants & Data Types

| Constant | Value | Description |
|---|---|---|
| `HANOIP_WCN3990_BASE` | `0x0a00_0000` | Qualcomm WCN3990 SMD MMIO Base |
| `CAP_NET_ADMIN` | `0x0200_0000` | Capability bit for network configuration |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Pure in-tree Rust, zero external crates in TCB.
2. **Rule N-04:** Userspace driver `userspace/devmgr/src/wifi_wcn3990.rs` is 100% safe Rust.
3. **Rule N-06 / PRD-001 §7.5:** Network interface faults are isolated from kernel core state.
