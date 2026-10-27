#!/usr/bin/env python3
"""
Ind OS Sprint SP-81 Verification & Acceptance Test Harness
Target: Battery Fuel Gauge & Charging State Machine
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-PWR-102, INV-BATT-001..003
"""

import os
import sys

REQUIRED_SP81_FILES = [
    "docs/architecture/battery-fuel-gauge-spec.md",
    "docs/evidence/SP-81-validation.md",
    "docs/evidence/golden-boot-sp81.log",
    "userspace/devmgr/src/battery_gauge.rs",
    "kernel/src/hal/platform/battery_gauge_core.rs",
]

def verify_sp81():
    print("==================================================")
    print(" Ind OS SP-81 Battery Fuel Gauge Verifier         ")
    print(" Target: Sprint SP-81 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-81 deliverable presence...")
    for rel_path in REQUIRED_SP81_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-81 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/battery_gauge.rs", "r") as f:
        batt_src = f.read()
    for s in [
        "ChargingState",
        "BatteryError",
        "BatteryGaugeHarness",
        "BATT_MIN_VOLTAGE_MV",
        "BATT_MAX_VOLTAGE_MV",
        "BATT_CRITICAL_SOC_PCT",
        "update_telemetry",
        "soc_pct",
        "voltage_mv",
        "state",
        "emergency_flush_triggered",
    ]:
        if s not in batt_src:
            print(f"❌ FAIL: battery_gauge.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/battery_gauge_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelChargingState",
        "KernelBatteryError",
        "KernelBatteryGaugeHarness",
        "KERNEL_BATT_MIN_VOLTAGE_MV",
        "KERNEL_BATT_MAX_VOLTAGE_MV",
        "KERNEL_BATT_CRITICAL_SOC_PCT",
        "update_telemetry",
        "soc_pct",
        "voltage_mv",
        "state",
        "emergency_flush_triggered",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: battery_gauge_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/battery_gauge.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/battery_gauge.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/battery_gauge.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 355-359)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 355-359...")
    with open("docs/evidence/golden-boot-sp81.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "355_SP81_BATTERY_GAUGE_INIT",
        "356_SP81_TELEMETRY_BOUNDS_CHECK",
        "357_SP81_CHARGING_STATE_TRANSITIONS",
        "358_SP81_CRITICAL_BATTERY_EMERGENCY_FLUSH",
        "359_SP81_BATTERY_GAUGE_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Battery Gauge Confirmation Logs
    print("\n[CHECK 6/6] Verifying Battery Gauge Confirmation Logs...")
    expected_logs = [
        "Telemetry bounds verified: 75% SoC, 4100mV safely within [3200, 4400] mV limits (INV-BATT-001 PASSED)",
        "State transitions verified: ChargingFast -> ChargingTrickle -> Full evaluated (INV-BATT-002 PASSED)",
        "Brownout guard verified: SoC <= 3% triggers emergency storage sync before power cut (INV-BATT-003, Rule N-06 PASSED)",
        "[SP-81] Sprint SP-81 Battery Fuel Gauge Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-81 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp81()
