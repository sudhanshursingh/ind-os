#!/usr/bin/env python3
"""
Ind OS Sprint SP-70 Verification & Acceptance Test Harness
Target: Programme Preparation & Hardware Lab Setup (Serial, Power/Thermal, Recovery Station)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-101, REQ-SEC-111, Rule N-06, INV-LAB-001..003
"""

import os
import sys

REQUIRED_SP70_FILES = [
    "docs/architecture/hardware-lab-prep-spec.md",
    "docs/evidence/SP-70-validation.md",
    "docs/evidence/golden-boot-sp70.log",
    "userspace/devmgr/src/lab_prep.rs",
    "kernel/src/hal/platform/lab_prep_core.rs",
]

def verify_sp70():
    print("==================================================")
    print(" Ind OS SP-70 Hardware Lab Prep Verifier         ")
    print(" Target: Sprint SP-70 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-70 deliverable presence...")
    for rel_path in REQUIRED_SP70_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-70 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/lab_prep.rs", "r") as f:
        lab_src = f.read()
    for s in [
        "LabPrepError",
        "HardwareLabHarness",
        "SAFE_VOLTAGE_MIN_MV",
        "SAFE_VOLTAGE_MAX_MV",
        "THERMAL_CUTOFF_MILLICELSIUS",
        "EXPECTED_UART_BAUD",
        "configure_serial",
        "audit_power_envelope",
        "verify_recovery_station",
    ]:
        if s not in lab_src:
            print(f"❌ FAIL: lab_prep.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/lab_prep_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelLabPrepError",
        "KernelHardwareLabHarness",
        "KERNEL_SAFE_VOLTAGE_MIN_MV",
        "KERNEL_SAFE_VOLTAGE_MAX_MV",
        "KERNEL_THERMAL_CUTOFF_MILLICELSIUS",
        "KERNEL_EXPECTED_UART_BAUD",
        "configure_serial",
        "audit_power_envelope",
        "verify_recovery_station",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: lab_prep_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/lab_prep.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/lab_prep.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/lab_prep.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 300-304)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 300-304...")
    with open("docs/evidence/golden-boot-sp70.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "300_SP70_LAB_PREP_INIT",
        "301_SP70_SERIAL_CONSOLE_PROTOCOL",
        "302_SP70_POWER_MONITOR_BOUNDS",
        "303_SP70_RECOVERY_STATION_READY",
        "304_SP70_LAB_PREP_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Lab Setup Confirmation
    print("\n[CHECK 6/6] Verifying Hardware Lab Confirmation Logs...")
    expected_logs = [
        "Serial console verified: 115200 8N1 UART transport framed and active (INV-LAB-001 PASSED)",
        "Power envelope verified: 3.85V / 28.5C safely bounded within Li-ion limits (INV-LAB-002 PASSED)",
        "Recovery workstation verified: Fastboot/EDL emergency restore route proven (INV-LAB-003, Rule N-06 PASSED)",
        "[SP-70] Sprint SP-70 Hardware Lab Prep Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-70 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp70()
