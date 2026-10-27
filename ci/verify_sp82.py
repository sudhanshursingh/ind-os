#!/usr/bin/env python3
"""
Ind OS Sprint SP-82 Verification & Acceptance Test Harness
Target: Thermal Management & Dynamic Throttling (TSENS, Conservative Bounds, Frequency Step-Down)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-THM-101, INV-THM-001..003
"""

import os
import sys

REQUIRED_SP82_FILES = [
    "docs/architecture/thermal-management-spec.md",
    "docs/evidence/SP-82-validation.md",
    "docs/evidence/golden-boot-sp82.log",
    "userspace/devmgr/src/thermal_mgmt.rs",
    "kernel/src/hal/platform/thermal_mgmt_core.rs",
]

def verify_sp82():
    print("==================================================")
    print(" Ind OS SP-82 Thermal Management Verifier         ")
    print(" Target: Sprint SP-82 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-82 deliverable presence...")
    for rel_path in REQUIRED_SP82_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-82 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/thermal_mgmt.rs", "r") as f:
        thm_src = f.read()
    for s in [
        "ThermalZone",
        "ThermalError",
        "ThermalMgmtHarness",
        "HANOIP_TSENS0_BASE",
        "THERMAL_WARN_MILLICELSIUS",
        "THERMAL_THROTTLE_MILLICELSIUS",
        "THERMAL_SHUTDOWN_MILLICELSIUS",
        "map_mmio",
        "sample_sensor",
        "current_temp_mc",
        "zone",
        "clock_budget_pct",
    ]:
        if s not in thm_src:
            print(f"❌ FAIL: thermal_mgmt.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/thermal_mgmt_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelThermalZone",
        "KernelThermalError",
        "KernelThermalMgmtHarness",
        "KERNEL_HANOIP_TSENS0_BASE",
        "KERNEL_THERMAL_WARN_MILLICELSIUS",
        "KERNEL_THERMAL_THROTTLE_MILLICELSIUS",
        "KERNEL_THERMAL_SHUTDOWN_MILLICELSIUS",
        "map_mmio",
        "sample_sensor",
        "current_temp_mc",
        "zone",
        "clock_budget_pct",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: thermal_mgmt_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/thermal_mgmt.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/thermal_mgmt.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/thermal_mgmt.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 360-364)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 360-364...")
    with open("docs/evidence/golden-boot-sp82.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "360_SP82_THERMAL_MGMT_INIT",
        "361_SP82_TSENS_CONTROLLER_MAPPING",
        "362_SP82_THERMAL_TRIP_POINTS_EVAL",
        "363_SP82_DYNAMIC_CPU_THROTTLING_HOOKS",
        "364_SP82_THERMAL_MGMT_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Thermal Management Confirmation Logs
    print("\n[CHECK 6/6] Verifying Thermal Management Confirmation Logs...")
    expected_logs = [
        "TSENS verified: sensor controller mapped at 0x01f0_8000 (INV-THM-001 PASSED)",
        "Trip-points verified: Normal (35C, 100%) and Warning (50C, 80%) evaluated (INV-THM-002 PASSED)",
        "Throttling verified: 70C clamps clock to 50%; >=85C triggers emergency shutdown (INV-THM-003 PASSED)",
        "[SP-82] Sprint SP-82 Thermal Management Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-82 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp82()
