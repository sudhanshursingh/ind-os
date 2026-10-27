# Sprint SP-122 Specification: Beta Cohort Planning & Consent-First Telemetry

## 1. Context & Objectives
- **Milestone:** Phase 10 — Hardening, Verification, Beta Readiness (Bible §8.17)
- **Sprint:** SP-122 — Beta cohort planning & consent-first telemetry
- **Primary Requirement:** `PRD-001 §7.4` (Consent-first local telemetry & privacy posture)
- **Constitutional Invariant:** `INV-BETA-001` (Opt-In Local Telemetry & Instant Revocation)

## 2. Beta Cohort Architecture & Device Tiering
1. **Target Hardware Tiers:**
   - Tier 1 (Reference Silicon): Motorola Moto G60 (`hanoip` / Qualcomm Snapdragon 732G).
   - Tier 2 (Virtual Emulation): QEMU ARM64 `virt` headless & interactive session.
   - Tier 3 (Developer Boards): Radxa Rock 5B / generic Cortex-A76 dev boards.
2. **Consent-First Telemetry Protocol:**
   - Default Posture: STRICT OFF ($\emptyset$ network egress permitted).
   - Explicit Opt-In: Requires user cryptographic consent receipt signed via interactive UI.
   - Local-First Storage: All diagnostic events aggregated in encrypted IndFS `/var/log/telemetry.journal`.
   - Instant Revocation: User toggle immediately wipes local buffer and revokes network capability handle.

## 3. Invariants
- `INV-BETA-001`: Zero diagnostic telemetry packets can be dispatched across any network interface unless an active, unrevoked consent capability is presented to the socket layer.
- `INV-BETA-001a`: Revoking consent deletes staged telemetry logs and transitions network state to isolated mode in $< 5\text{ms}$.
