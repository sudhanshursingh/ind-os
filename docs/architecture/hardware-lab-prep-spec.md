# Ind OS Hardware Lab Preparation & Reference Dev Board Specification (SP-70)

## 1. Architectural Mandate
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.15, Table 19.2 (REQ-HW-101, REQ-SEC-111).
- **Target:** Reference dev board staging per PEN-010, hardware lab setup (serial console access, power/thermal monitoring harness, recovery workstation protocols).
- **Recovery-First Law (Rule N-06 / PRD-001 §7.5):** No physical device experiment may proceed without an isolated, verified recovery workstation path.

---

## 2. Architectural Invariants

| ID | Description | Enforcement Mechanism |
| :--- | :--- | :--- |
| **INV-LAB-001** | Serial Console Transport & Baud Integrity | Hardware UART configured to 115200 8N1 with structured framing and heartbeat telemetry |
| **INV-LAB-002** | Power & Thermal Envelope Bounding | Operating voltage clamped to safe Li-ion range `[3.60V, 4.40V]`; hardware thermal cutoff enforced at $\le 45^\circ\text{C}$ |
| **INV-LAB-003** | Recovery Workstation Validation | Recovery host verified with Fastboot and EDL handshake tools before experimental flashing |

---

## 3. Data Structures & API Contracts

```rust
pub const SAFE_VOLTAGE_MIN_MV: u32 = 3600; // 3.60V
pub const SAFE_VOLTAGE_MAX_MV: u32 = 4400; // 4.40V
pub const THERMAL_CUTOFF_MILLICELSIUS: u32 = 45000; // 45.0 C

pub enum LabPrepError {
    VoltageOutOfRange,
    ThermalThresholdExceeded,
    BaudRateMismatch,
    RecoveryStationUnreachable,
}

pub struct HardwareLabHarness {
    serial_baud: u32,
    voltage_mv: u32,
    temp_millicelsius: u32,
    recovery_station_verified: bool,
}
```
