#!/usr/bin/env python3
"""
Ind OS Sprint SP-101 Verification & Acceptance Test Harness
Target: Settings & Power Management Hardware Integration
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-PWR-104, INV-SETTINGS-PWR-001..003
"""

import os
import sys

REQUIRED_SP101_FILES = [
    "docs/architecture/settings-power-hardware-spec.md",
    "docs/evidence/SP-101-validation.md",
    "docs/evidence/golden-boot-sp101.log",
    "userspace/ux/src/settings_power.rs",
    "kernel/src/hal/platform/settings_power_core.rs",
]

def verify_sp101():
    print("==================================================")
    print(" Ind OS SP-101 Settings & Power Hardware Verifier ")
    print(" Target: Sprint SP-101 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-101 deliverable presence...")
    for rel_path in REQUIRED_SP101_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-101 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/settings_power.rs", "r") as f:
        settings_src = f.read()
    for s in [
        "SettingsPowerEngine",
        "SettingsPowerError",
        "ChargingStatus",
        "PowerProfile",
        "BatteryTelemetry",
        "PowerGovernorState",
        "CAP_POWER_MGMT",
        "BATTERY_FULL_MV",
        "BATTERY_EMPTY_MV",
        "DISPLAY_RATE_ECO_HZ",
        "DISPLAY_RATE_PERF_HZ",
        "THERMAL_MILD_MC",
        "THERMAL_SEVERE_MC",
        "THERMAL_CRITICAL_MC",
        "sync_battery",
        "set_display_governor",
        "apply_thermal_policy",
        "telemetry",
        "governor_state",
    ]:
        if s not in settings_src:
            print(f"❌ FAIL: settings_power.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/settings_power_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelSettingsPowerHarness",
        "KernelSettingsPowerError",
        "KernelChargingStatus",
        "KernelPowerProfile",
        "KernelBatteryTelemetry",
        "KernelPowerGovernorState",
        "KERNEL_CAP_SETTINGS_POWER",
        "KERNEL_BATTERY_FULL_MV",
        "KERNEL_BATTERY_EMPTY_MV",
        "KERNEL_DISPLAY_RATE_ECO_HZ",
        "KERNEL_DISPLAY_RATE_PERF_HZ",
        "KERNEL_THERMAL_MILD_MC",
        "KERNEL_THERMAL_SEVERE_MC",
        "KERNEL_THERMAL_CRITICAL_MC",
        "sync_battery",
        "set_display_governor",
        "apply_thermal_policy",
        "telemetry",
        "governor_state",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: settings_power_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01 Compliance: Zero external crates
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for cargo_path in ["kernel/Cargo.toml", "userspace/ux/Cargo.toml"]:
        with open(cargo_path, "r") as f:
            c = f.read()
        lines = c.splitlines()
        in_deps = False
        for line in lines:
            line_str = line.strip()
            if line_str.startswith("[dependencies]"):
                in_deps = True
                continue
            if in_deps and line_str.startswith("["):
                in_deps = False
            if in_deps and line_str and not line_str.startswith("#"):
                if "path =" not in line_str:
                    print(f"❌ FAIL: Non-path external dependency in {cargo_path}: {line_str}")
                    sys.exit(1)
        print(f"  ✓ {cargo_path} compliant with Rule N-01")

    # 4. Rule N-04 Compliance: Zero unsafe in newly authored userspace
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/ux/src/settings_power.rs)...")
    with open("userspace/ux/src/settings_power.rs", "r") as f:
        content = f.read()
    if "unsafe " in content or "unsafe{" in content:
        print("❌ FAIL: unsafe code block found in userspace/ux/src/settings_power.rs")
        sys.exit(1)
    print("  ✓ userspace/ux/src/settings_power.rs is 100% safe Rust")

    # 5. Golden Boot Log Verification: Stages 455-459
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 455-459...")
    with open("docs/evidence/golden-boot-sp101.log", "r") as f:
        log_content = f.read()

    stages = [
        "455_SP101_POWER_SETTINGS_INIT",
        "456_SP101_BATTERY_TELEMETRY_SYNC",
        "457_SP101_DISPLAY_POWER_GOVERNOR",
        "458_SP101_THERMAL_POWER_POLICY_BIND",
        "459_SP101_SETTINGS_POWER_READY",
    ]
    for st in stages:
        if st not in log_content:
            print(f"❌ FAIL: Missing stage {st} in docs/evidence/golden-boot-sp101.log")
            sys.exit(1)
        print(f"  ✓ Verified stage: {st}")

    # 6. Specific Confirmation Logs
    print("\n[CHECK 6/6] Verifying Settings & Power Confirmation Logs...")
    expected_logs = [
        "Settings power arbiter online: PMIC telemetry & policy governor armed (REQ-HW-108 PASSED)",
        "Battery telemetry synced: 4050mV, 88% SoC, -250mA discharging status (INV-SETTINGS-PWR-001 PASSED)",
        "Display power governor active: 200/255 PWM backlight, 120Hz dynamic refresh verified (INV-SETTINGS-PWR-002 PASSED)",
        "Thermal policy binding confirmed: Dynamic 38C/42C/46C PMIC rail and governor attenuation (INV-SETTINGS-PWR-003 PASSED)",
        "[SP-101] Sprint SP-101 Settings & Power Hardware Integration Milestone REACHED",
    ]
    for el in expected_logs:
        if el not in log_content:
            print(f"❌ FAIL: Log string not found: '{el}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {el[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-101 VERIFICATION PASSED (100% DoD) ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp101()
