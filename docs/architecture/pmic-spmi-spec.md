# Qualcomm PMIC & SPMI Bus Specification (SP-80)

- **Sprint:** SP-80 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** Qualcomm PM7150/PM7150L PMIC & SPMI Host Controller
- **Document ID:** SPEC-ARCH-SP80-PMIC-SPMI
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-PWR-101

---

## 1. Objectives & Scope

Sprint SP-80 implements the low-level power management controller driver for the Motorola Moto G60. The Snapdragon 732G communicates with its Power Management ICs (Qualcomm PM7150 and PM7150L) across the System Power Management Interface (SPMI) bus. This sprint maps the SPMI host controller at MMIO base `0x0940_0000`, enumerates PMIC slave devices, and establishes capability-gated telemetry interfaces.

---

## 2. Architectural Invariants

### INV-PMIC-001: SPMI Host Controller MMIO Mapping
The SPMI arbiter MMIO registers are mapped at physical base `0x0940_0000` with 64KB aperture alignment.

### INV-PMIC-002: Dual-PMIC Slave Enumeration
The driver probes and enumerates the two onboard PMICs:
- Primary PMIC: PM7150 at SPMI Slave ID (SID) 0.
- Secondary PMIC: PM7150L at SPMI Slave ID (SID) 2.

### INV-PMIC-003: Capability-Gated Power Rail Telemetry
All regulator telemetry and status readouts require the `CAP_POWER_MGMT` capability token. Unprivileged access to power rail controls is strictly denied by the kernel broker.

---

## 3. Interfaces & Data Structures

- `PmicSpmiHarness`: Userspace safe driver modeling SPMI transactions and PMIC registers.
- `KernelPmicSpmiHarness`: In-kernel HAL platform driver executing hardware initialization.
