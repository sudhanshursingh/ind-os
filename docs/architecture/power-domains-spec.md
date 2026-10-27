# Power Domains & PSCI / RPMh Specification (SP-83)

- **Sprint:** SP-83 (Phase 9: Moto G60 Bring-Up)
- **Subsystem:** ARM PSCI v1.1 & Qualcomm RPMh Resource State Coordinator (RSC)
- **Document ID:** SPEC-ARCH-SP83-POWER-DOMAINS
- **Status:** Approved
- **Target Hardware:** Motorola Moto G60 (`hanoip`, Snapdragon SM7150-AC)
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-PWR-103

---

## 1. Objectives & Scope

Sprint SP-83 implements the core CPU power domain management and system sleep coordination subsystem for the Motorola Moto G60. The driver interfaces with ARM standard Power State Coordination Interface (PSCI v1.1) via Secure Monitor Calls (SMC) for CPU core online/offline state switching, and drives Qualcomm's Resource Power Manager Hardened (RPMh) Resource State Coordinator (RSC) at MMIO base `0x179c_0000` to manage shared voltage and clock rails during idle periods.

---

## 2. Architectural Invariants

### INV-PWR-001: ARM Standard PSCI v1.1 SMC Interface
The kernel dispatches CPU power requests using standard 64-bit PSCI SMC calls:
- `PSCI_FN64_CPU_SUSPEND = 0xC400_0001`
- `PSCI_FN_CPU_OFF = 0x8400_0002`
- `PSCI_FN_SYSTEM_RESET = 0x8400_0009`

### INV-PWR-002: Qualcomm RPMh RSC MMIO Mapping
The RPMh Resource State Coordinator registers are memory-mapped at physical base `0x179c_0000` with 64KB command queue aperture.

### INV-PWR-003: Capability-Gated Power Transitions
Core power domain transitions and system sleep states require the `CAP_POWER_DOMAINS` capability token. Unprivileged callers cannot place cores into retention or initiate power resets.

---

## 3. Interfaces & Data Structures

- `PowerDomainState`: Enum (`Active`, `Retention`, `Suspend`, `Off`).
- `PowerDomainsHarness`: Userspace safe driver modeling PSCI calls and RPMh queues.
- `KernelPowerDomainsHarness`: In-kernel HAL platform driver executing hardware checks.
