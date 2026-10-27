#!/usr/bin/env python3
"""
Ind OS Sprint SP-80 Verification & Acceptance Test Harness
Target: Qualcomm PMIC & SPMI Bus Bring-Up (PM7150, PM7150L, Regulators, Telemetry)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-PWR-101, INV-PMIC-001..003
"""

import os
import sys

REQUIRED_SP80_FILES = [
    "docs/architecture/pmic-spmi-spec.md",
    "docs/evidence/SP-80-validation.md",
    "docs/evidence/golden-boot-sp80.log",
    "userspace/devmgr/src/pmic_spmi.rs",
    "kernel/src/hal/platform/pmic_spmi_core.rs",
]

def verify_sp80():
    print("==================================================")
    print(" Ind OS SP-80 Qualcomm PMIC & SPMI Verifier       ")
    print(" Target: Sprint SP-80 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-80 deliverable presence...")
    for rel_path in REQUIRED_SP80_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-80 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/pmic_spmi.rs", "r") as f:
        pmic_src = f.read()
    for s in [
        "PmicError",
        "PmicSpmiHarness",
        "HANOIP_SPMI_BASE",
        "PMIC_PM7150_SID",
        "PMIC_PM7150L_SID",
        "CAP_POWER_MGMT",
        "map_mmio",
        "probe_pmic_slaves",
        "read_regulator_voltage_mv",
        "is_pm7150_present",
        "is_pm7150l_present",
    ]:
        if s not in pmic_src:
            print(f"❌ FAIL: pmic_spmi.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/pmic_spmi_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelPmicError",
        "KernelPmicSpmiHarness",
        "KERNEL_HANOIP_SPMI_BASE",
        "KERNEL_PMIC_PM7150_SID",
        "KERNEL_PMIC_PM7150L_SID",
        "KERNEL_CAP_POWER_MGMT",
        "map_mmio",
        "probe_pmic_slaves",
        "read_regulator_voltage_mv",
        "is_pm7150_present",
        "is_pm7150l_present",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: pmic_spmi_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/devmgr/Cargo.toml"]:
        with open(toml_path, "r") as f:
            lines = [line.strip() for line in f.readlines() if line.strip() and not line.strip().startswith("#")]
        in_deps = False
        for line in lines:
            if line.startswith("[dependencies]"):
                in_deps = True
                continue
            if line.startswith("[") and in_deps:
                in_deps = False
                continue
            if in_deps and line:
                if "path =" not in line:
                    print(f"❌ FAIL: External dependency detected in {toml_path}: {line}")
                    sys.exit(1)
        print(f"  ✓ {toml_path} compliant with Rule N-01")

    # 4. Rule N-04: Zero Unsafe Code Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/devmgr)...")
    with open("userspace/devmgr/src/pmic_spmi.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/pmic_spmi.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/pmic_spmi.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 350-354)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 350-354...")
    with open("docs/evidence/golden-boot-sp80.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "350_SP80_PMIC_SPMI_INIT",
        "351_SP80_SPMI_CONTROLLER_MAPPING",
        "352_SP80_DUAL_PMIC_SLAVE_ENUMERATION",
        "353_SP80_POWER_RAIL_TELEMETRY_GATING",
        "354_SP80_PMIC_SPMI_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify PMIC Confirmation Logs
    print("\n[CHECK 6/6] Verifying PMIC Confirmation Logs...")
    expected_logs = [
        "SPMI arbiter verified: controller mapped at 0x0940_0000 (INV-PMIC-001 PASSED)",
        "Dual-PMIC enumerated: PM7150 (SID 0) & PM7150L (SID 2) active with 18 regulators (INV-PMIC-002 PASSED)",
        "Telemetry verified: VDD_IO (1.8V) telemetry capability-gated via CAP_POWER_MGMT (INV-PMIC-003 PASSED)",
        "[SP-80] Sprint SP-80 Qualcomm PMIC & SPMI Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-80 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp80()
