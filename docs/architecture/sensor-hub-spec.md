# Ind OS Architecture Specification: Unified Sensor Event Hub (Sprint SP-92)

**Document ID:** SPEC-ARCH-SP92-SENSOR-HUB  
**Status:** Approved  
**Target:** Unified Sensor Event Hub & Multiplexer Service  
**Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-110, REQ-SNS-104  
**Hardware:** Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G SM7150-AC)

---

### 1. Architectural Invariants

- **INV-HUB-001 (Subscription & Capability Gating):** Sensor event subscription requires `CAP_SENSOR_READ` (`0x0800_0000`) token; unprivileged access is rejected.
- **INV-HUB-002 (Microsecond Timestamp Synchronization):** Sensor telemetry from IMU (accel/gyro), ALS/Prox, and Magnetometer channels is stamped with a monotonic 64-bit microsecond counter (`timestamp_us`), eliminating cross-sensor phase skew.
- **INV-HUB-003 (Multiplexed Event Dispatch & FIFO Isolation):** Ring-buffer dispatch dispatches timestamped events across multiple subscribers in strict chronological sequence without dropping frames under sustained 100 Hz sampling.

---

### 2. Constants & Data Types

| Constant / Type | Value / Signature | Description |
|---|---|---|
| `MAX_SENSOR_SUBSCRIBERS` | `8` | Maximum concurrent subscribers to the sensor hub |
| `SENSOR_EVENT_RING_CAPACITY` | `32` | Circular buffer queue size per subscriber |
| `CAP_SENSOR_READ` | `0x0800_0000` | Mandatory capability bit for sensor reading |
| `SensorChannel` | `ImuAccel`, `ImuGyro`, `AlsLux`, `Proximity`, `Magnetometer` | Telemetry channel classification |
| `SensorEvent` | `{ channel, timestamp_us, data: [i32; 3] }` | Normalized unified event payload |

---

### 3. Safety & Isolation Contracts

1. **Rule N-01:** Zero external crates in TCB. In-tree pure Rust.
2. **Rule N-04:** Userspace driver `userspace/devmgr/src/sensor_hub.rs` is 100% safe Rust.
3. **Rule N-06 / PRD-001 §7.5:** Fixed-size ring buffer guarantees zero memory allocations during real-time event dispatch.
