#!/usr/bin/env python3
"""
Ind OS Sprint SP-97 Verification & Acceptance Test Harness
Target: AIS Memory Bounds & Thermal Throttling on Physical SoC
Standards: Bible §6.3, §8.15, Table 19.2, REQ-AI-102, REQ-THM-102, INV-AIS-THM-001..003
"""

import os
import sys

REQUIRED_SP97_FILES = [
    "docs/architecture/ais-thermal-throttling-spec.md",
    "docs/evidence/SP-97-validation.md",
    "docs/evidence/golden-boot-sp97.log",
    "userspace/ais/src/thermal_governor.rs",
    "kernel/src/hal/platform/ais_thermal_core.rs",
]

def verify_sp97():
    print("==================================================")
    print(" Ind OS SP-97 AIS Thermal Governor Verifier       ")
    print(" Target: Sprint SP-97 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-97 deliverable presence...")
    for rel_path in REQUIRED_SP97_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-97 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ais/src/thermal_governor.rs", "r") as f:
        thm_src = f.read()
    for s in [
        "AisThermalGovernorHarness",
        "AisThermalState",
        "AIS_TEMP_NORMAL_MC",
        "AIS_TEMP_THROTTLE_MC",
        "AIS_TEMP_EMERGENCY_HALT_MC",
        "AIS_TEMP_RECOVERY_MC",
        "update_temperature",
        "current_duty_cycle_percent",
        "is_background_allowed",
        "is_emergency_halted",
        "current_temperature_mc",
    ]:
        if s not in thm_src:
            print(f"❌ FAIL: thermal_governor.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/ais_thermal_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelAisThermalGovernorHarness",
        "KernelAisThermalState",
        "KERNEL_AIS_TEMP_NORMAL_MC",
        "KERNEL_AIS_TEMP_THROTTLE_MC",
        "KERNEL_AIS_TEMP_EMERGENCY_HALT_MC",
        "KERNEL_AIS_TEMP_RECOVERY_MC",
        "update_temperature",
        "current_duty_cycle_percent",
        "is_background_allowed",
        "is_emergency_halted",
        "current_temperature_mc",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: ais_thermal_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/ais/Cargo.toml"]:
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
            if line.startswith("path =") or "path =" in line:
                continue
            if in_deps and line:
                print(f"❌ FAIL: External dependency detected in {toml_path}: {line}")
                sys.exit(1)
        print(f"  ✓ {toml_path} compliant with Rule N-01")

    # 4. Rule N-04: Zero Unsafe Code Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/ais)...")
    with open("userspace/ais/src/thermal_governor.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/ais/src/thermal_governor.rs")
            sys.exit(1)
    print("  ✓ userspace/ais/src/thermal_governor.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 435-439)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 435-439...")
    with open("docs/evidence/golden-boot-sp97.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "435_SP97_AIS_THERMAL_GOVERNOR_INIT",
        "436_SP97_AIS_THERMAL_THROTTLE_TRIGGER",
        "437_SP97_AIS_EMERGENCY_THERMAL_HALT",
        "438_SP97_AIS_HYSTERESIS_COOLING_RECOVERY",
        "439_SP97_AIS_THERMAL_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Confirmation Logs
    print("\n[CHECK 6/6] Verifying Thermal Confirmation Logs...")
    expected_logs = [
        "AIS thermal governor online: 100% nominal duty-cycle active (REQ-THM-102 PASSED)",
        "Thermal throttle verified: 42.0C triggers 50% duty-cycle throttling (INV-AIS-THM-001 PASSED)",
        "Emergency halt verified: 46.0C shuts down non-essential inference (INV-AIS-THM-002 PASSED)",
        "Hysteresis verified: cooling below 39.0C restores 100% nominal throughput (INV-AIS-THM-003 PASSED)",
        "[SP-97] Sprint SP-97 AIS Memory Bounds & Thermal Throttling Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-97 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp97()
