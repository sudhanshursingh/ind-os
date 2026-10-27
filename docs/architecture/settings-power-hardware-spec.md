# Ind OS SP-101 Architecture Specification
## Settings & Power Management Hardware Integration

**Sprint:** SP-101 (Stage 455–459)  
**Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2; REQ-HW-108, REQ-PWR-104  
**Target Hardware:** Motorola Moto G60 (Qualcomm Snapdragon 732G SM7150-AC, PM6150 / PM6150L PMIC, SMB1395 fuel gauge)  
**Status:** APPROVED FOR IMPLEMENTATION

---

### 1. Purpose and Scope

Sprint SP-101 integrates the OS settings and power management subsystem directly with the physical PMIC power rails, fuel-gauge battery telemetry, and thermal mitigation governors. It provides verified hardware-backed telemetry for battery percentage, voltage, charging state, dynamic display backlight control, variable refresh rate switching (60Hz / 120Hz), and thermal policy throttling.

---

### 2. Architectural Invariants

- **INV-SETTINGS-PWR-001 (Hardware Battery Telemetry Sync):**  
  Battery fuel gauge reads (voltage in mV, State of Charge %, instantaneous current in mA) must be refreshed periodically via I2C/SPMI transactions with monotonic bounding and capability token `CAP_POWER_MGMT`.

- **INV-SETTINGS-PWR-002 (Dynamic Display Power Governance):**  
  Brightness attenuation (0..255 PWM duty cycle) and dynamic panel refresh rate switching (60Hz for battery saver / 120Hz for high fluidity) execute with deterministic hardware state confirmation within <= 2 frames.

- **INV-SETTINGS-PWR-003 (Thermal Mitigation Policy Binding):**  
  Hardware power management policy binds to PMIC thermal zones:
  - Normal (<= 38.0°C): 100% CPU/GPU performance budget, 120Hz display.
  - Mild Throttling (> 38.0°C): Backlight capped at 80%, 60Hz display.
  - Severe Throttling (> 42.0°C): Background task suspension, CPU capped at 50%.
  - Critical Thermal Cutoff (> 46.0°C): PMIC emergency load shed to prevent battery degradation.

---

### 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** Pure memory-safe Rust in userspace implementation (`userspace/ux/src/settings_power.rs`).
