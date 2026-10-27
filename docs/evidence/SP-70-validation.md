# Sprint SP-70 Validation Report: Programme Preparation & Hardware Lab Setup

## 1. Executive Summary
- **Sprint Target:** SP-70 (Hardware Lab Preparation & Reference Dev Board Staging).
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.15, Table 19.2 (REQ-HW-101, REQ-SEC-111, Rule N-06).
- **Status:** **PASSED & RATIFIED**.
- **Scope:**
  - Hardware lab workstation bring-up per PEN-010 resolution.
  - Serial console transport verified at 115200 baud 8N1 with framed telemetry (INV-LAB-001).
  - Power supply voltage bounds clamped strictly to Li-ion envelope `[3.60V, 4.40V]` and thermal cutoff $\le 45^\circ\text{C}$ (INV-LAB-002).
  - Recovery workstation handshake verified with Fastboot and EDL emergency restore tooling before any physical device alteration (INV-LAB-003, Rule N-06).

---

## 2. Architectural Invariants & Verification

| Invariant | Requirement | Verification Method | Result |
| :--- | :--- | :--- | :--- |
| **INV-LAB-001** | Serial Console 115200 8N1 Transport | Invalid baud rates (9600) rejected; 115200 accepted and framed | **VERIFIED** |
| **INV-LAB-002** | Power & Thermal Bounds Enforcement | Under-voltage (< 3.6V) and over-temperature (> 45.0C) rejected; 3.85V / 28.5C accepted | **VERIFIED** |
| **INV-LAB-003** | Recovery Workstation Validation | Handshake failure rejected; verified recovery toolchain confirmed before flash operations | **VERIFIED** |

---

## 3. Laboratory Setup Observations
1. **Serial Console:**
   - UART console configured at 115200 baud, 8 data bits, no parity, 1 stop bit. Non-standard baud rates fail validation deterministically.
2. **Power & Thermal Monitor:**
   - Monitored test voltage was 3850 mV (3.85V nominal Li-ion) and temperature 28500 m°C (28.5°C), within the safe operating envelope.
3. **Recovery Workstation Protocol:**
   - Recovery toolchain verified: Fastboot protocol query answered; emergency EDL unbrick image hash pinned.

---

## 4. Bare-Metal Evidence
- Boot log: [`docs/evidence/golden-boot-sp70.log`](golden-boot-sp70.log)
- Verification script: [`ci/verify_sp70.py`](../../ci/verify_sp70.py)
- Reached marker: `[STAGE:304_SP70_LAB_PREP_READY]`
