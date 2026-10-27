# Ind OS Architecture Specification: Ambient Light & Proximity Sensor Driver (Sprint SP-90)

**Document ID:** SPEC-ARCH-SP90-SENSOR-ALS-PROX  
**Status:** Approved  
**Target:** Ambient Light (ALS) & Proximity Sensor Bring-Up  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-110, REQ-SNS-102  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-ALS-001 (ALS/Prox I2C Bus Mapping):** Connected via Qualcomm GENI I2C at physical base `0x0888_4000`, 7-bit slave address `0x39`.
- **INV-ALS-002 (Photometric Lux Calculation):** Photometric conversion formula converts raw ADC counts to lux ($0$ to $65,535$ lux) across auto-ranging optical gain states ($1\times, 8\times, 64\times$).
- **INV-ALS-003 (Proximity Hysteresis Event Dispatch):** Proximity state machine enforces deterministic threshold hysteresis (High threshold $= 50$, Low threshold $= 25$) to avoid bouncing between `Near` and `Far`.

---

### 2. Constants & Data Types

| Constant | Value | Description |
|---|---|---|
| `HANOIP_ALS_I2C_BASE` | `0x0888_4000` | Qualcomm GENI I2C ALS/Prox Base |
| `ALS_PROX_I2C_ADDR` | `0x39` | 7-bit I2C Slave Address |
| `PROX_THRESHOLD_HIGH` | `50` | Near detection threshold |
| `PROX_THRESHOLD_LOW` | `25` | Far release threshold |
| `CAP_SENSOR_READ` | `0x0800_0000` | Capability bit for sensor telemetry read |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Pure in-tree Rust, zero external crates in TCB.
2. **Rule N-04:** Userspace driver `userspace/devmgr/src/sensor_als_prox.rs` is 100% safe Rust.
3. **Rule N-06 / PRD-001 §7.5:** Hysteresis filtering prevents runaway event storms on display backlight control.
