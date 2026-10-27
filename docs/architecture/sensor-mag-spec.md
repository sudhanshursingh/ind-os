# Ind OS Architecture Specification: Magnetometer Driver (Sprint SP-91)

**Document ID:** SPEC-ARCH-SP91-SENSOR-MAG  
**Status:** Approved  
**Target:** Asahi Kasei AK09918 3-Axis Electronic Compass / Magnetometer Bring-Up  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-110, REQ-SNS-103  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-MAG-001 (AK09918 I2C Bus Mapping):** Connected via Qualcomm GENI I2C at physical base `0x0888_8000`, 7-bit slave address `0x0C`.
- **INV-MAG-002 (Company & Device ID):** Reads Company ID register `0x00` (`WIA1` == `0x48`) and Device ID register `0x01` (`WIA2` == `0x0C`).
- **INV-MAG-003 (Heading Calculation):** Continuous mode measurement acquires 3-axis magnetic micro-Tesla ($\mu\text{T}$) vector and computes orientation heading ($0^\circ$ to $359^\circ$).

---

### 2. Constants & Data Types

| Constant | Value | Description |
|---|---|---|
| `HANOIP_MAG_I2C_BASE` | `0x0888_8000` | Qualcomm GENI I2C Magnetometer Base |
| `AK09918_I2C_ADDR` | `0x0C` | 7-bit I2C Slave Address |
| `AK09918_WIA1_COMPANY` | `0x48` | AKM Company ID |
| `AK09918_WIA2_DEVICE` | `0x0C` | AK09918 Device ID |
| `CAP_SENSOR_READ` | `0x0800_0000` | Capability bit for sensor telemetry read |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Pure in-tree Rust, zero external crates in TCB.
2. **Rule N-04:** Userspace driver `userspace/devmgr/src/sensor_mag.rs` is 100% safe Rust.
3. **Rule N-06 / PRD-001 §7.5:** Fixed-point integer trigonometry computes magnetic heading safely without hardware float traps.
