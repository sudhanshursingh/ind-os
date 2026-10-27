# Ind OS Architecture Specification: Accelerometer & Gyroscope Driver (Sprint SP-89)

**Document ID:** SPEC-ARCH-SP89-SENSOR-IMU  
**Status:** Approved  
**Target:** Bosch BMI260 6-Axis Inertial Measurement Unit (IMU) Bring-Up  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-110, REQ-SNS-101  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-SNS-001 (BMI260 I2C Bus Mapping):** Connected via Qualcomm GENI I2C at physical base `0x0888_0000`, 7-bit slave address `0x68`.
- **INV-SNS-002 (Chip ID & Soft-Reset):** Reads register `0x00` (`CHIP_ID` == `0x27`) and executes software reset via command register `0x7E` (`0xB6`).
- **INV-SNS-003 (FIFO Sample Acquisition):** Configures 100Hz ODR with $\pm 16g$ accel range and $\pm 2000$ dps gyro range; validates FIFO watermark threshold.

---

### 2. Constants & Data Types

| Constant | Value | Description |
|---|---|---|
| `HANOIP_IMU_I2C_BASE` | `0x0888_0000` | Qualcomm GENI I2C IMU Base |
| `BMI260_I2C_ADDR` | `0x68` | 7-bit I2C Slave Address |
| `BMI260_CHIP_ID` | `0x27` | Bosch BMI260 Hardware Chip ID |
| `CAP_SENSOR_READ` | `0x0800_0000` | Capability bit for sensor telemetry read |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Pure in-tree Rust, zero external crates in TCB.
2. **Rule N-04:** Userspace driver `userspace/devmgr/src/sensor_imu.rs` is 100% safe Rust.
3. **Rule N-06 / PRD-001 §7.5:** IMU sensor polling operates within bounded FIFO buffers without dynamic memory allocations in the interrupt path.
