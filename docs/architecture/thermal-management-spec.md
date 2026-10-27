# Thermal Management & Throttling Specification (SP-82)

- **Sprint:** SP-82 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** Qualcomm TSENS Temperature Sensors & Dynamic CPU Throttling
- **Document ID:** SPEC-ARCH-SP82-THERMAL-MGMT
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-THM-101

---

## 1. Objectives & Scope

Sprint SP-82 establishes the hardware thermal monitoring and dynamic CPU throttling subsystem for the Motorola Moto G60. The Snapdragon 732G incorporates on-die temperature sensors (TSENS) mapped at MMIO base `0x01f0_8000`. The driver samples temperature across CPU compute cores and the PMIC battery zone, enforcing conservative throttling thresholds to maintain device surface temperatures within human tolerance without vendor thermal binaries.

---

## 2. Architectural Invariants

### INV-THM-001: TSENS Controller MMIO Mapping
The TSENS hardware registers are memory-mapped at physical base `0x01f0_8000` with 4KB page alignment.

### INV-THM-002: Conservative Multi-Tier Thermal Trip-Points
The driver enforces documented conservative bounds:
- **Normal:** Temperature $< 45^\circ\text{C}$ (100% clock budget).
- **Warning / Alert:** $45^\circ\text{C} \le \text{Temp} < 65^\circ\text{C}$ (Log telemetry, display indicator).
- **Throttle:** $65^\circ\text{C} \le \text{Temp} < 85^\circ\text{C}$ (Drop CPU clock to 50% and cap background task queues).
- **Critical Hot Shutdown:** $\text{Temp} \ge 85^\circ\text{C}$ (Immediate hardware thermal trip protection).

### INV-THM-003: Dynamic Frequency Step-Down Enforcement
When entering `Throttle` state, CPU frequency scaling hooks clamp core frequency to half-rate within 10ms, stabilizing die temperature before reaching thermal shutdown.

---

## 3. Interfaces & Data Structures

- `ThermalZone`: Enum (`Normal`, `Warning`, `Throttle`, `EmergencyShutdown`).
- `ThermalMgmtHarness`: Userspace safe driver modeling sensor telemetry and throttling policies.
- `KernelThermalMgmtHarness`: In-kernel HAL platform driver executing hardware checks.
