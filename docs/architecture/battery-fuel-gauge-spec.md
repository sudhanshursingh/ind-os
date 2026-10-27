# Battery Fuel Gauge & Charging Specification (SP-81)

- **Sprint:** SP-81 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** Battery Fuel Gauge Telemetry & Charging State Machine
- **Document ID:** SPEC-ARCH-SP81-BATTERY-GAUGE
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-PWR-102

---

## 1. Objectives & Scope

Sprint SP-81 implements the battery fuel gauge and charging state driver for the 6000mAh battery on the Motorola Moto G60. The driver reads Coulomb counter State-of-Charge (SoC), battery terminal voltage, and charging current, executing state transitions across `Discharging`, `ChargingFast`, `ChargingTrickle`, and `Full`. Additionally, it implements a hard emergency shutdown threshold (SoC $\le 3\%$) that triggers an immediate storage sync to prevent dirty file system states during brownouts.

---

## 2. Architectural Invariants

### INV-BATT-001: Fuel Gauge Telemetry Validation
State-of-Charge is validated within `[0, 100]%` and terminal voltage is bounded within safe Li-ion operating envelopes `[3200, 4400] mV`. Out-of-bounds readings raise hardware telemetry faults.

### INV-BATT-002: Deterministic Charging State Machine
The driver transitions states deterministically:
- `ChargingCurrent > 500mA` and `SoC < 90%` $\implies$ `ChargingFast`.
- `SoC >= 90%` and `SoC < 100%` $\implies$ `ChargingTrickle`.
- `SoC == 100%` $\implies$ `Full`.
- Charger disconnected $\implies$ `Discharging`.

### INV-BATT-003: Critical Low-Battery Emergency Storage Flush
When SoC drops to $\le 3\%$ or voltage drops below $3300\text{mV}$, the kernel emits an emergency sync event, issuing SCSI flush commands before battery depletion halts the CPU.

---

## 3. Interfaces & Data Structures

- `ChargingState`: Enum (`Discharging`, `ChargingFast`, `ChargingTrickle`, `Full`, `Fault`).
- `BatteryGaugeHarness`: Userspace driver modeling telemetry and charging states.
- `KernelBatteryGaugeHarness`: In-kernel HAL platform driver executing hardware checks.
