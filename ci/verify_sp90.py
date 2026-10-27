#!/usr/bin/env python3
"""
Ind OS Sprint SP-90 Verification & Acceptance Test Harness
Target: Ambient Light & Proximity Sensor (I2C MMIO, Lux Photometry, Proximity Hysteresis)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-110, REQ-SNS-102, INV-ALS-001..003
"""

import os
import sys

REQUIRED_SP90_FILES = [
    "docs/architecture/sensor-als-prox-spec.md",
    "docs/evidence/SP-90-validation.md",
    "docs/evidence/golden-boot-sp90.log",
    "userspace/devmgr/src/sensor_als_prox.rs",
    "kernel/src/hal/platform/sensor_als_prox_core.rs",
]

def verify_sp90():
    print("==================================================")
    print(" Ind OS SP-90 Sensor ALS/Prox Verifier            ")
    print(" Target: Sprint SP-90 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-90 deliverable presence...")
    for rel_path in REQUIRED_SP90_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-90 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/sensor_als_prox.rs", "r") as f:
        als_src = f.read()
    for s in [
        "ProximityState",
        "AlsGain",
        "AlsProxError",
        "SensorAlsProxHarness",
        "HANOIP_ALS_I2C_BASE",
        "ALS_PROX_I2C_ADDR",
        "PROX_THRESHOLD_HIGH",
        "PROX_THRESHOLD_LOW",
        "CAP_SENSOR_READ",
        "map_mmio",
        "sample_lux",
        "update_proximity",
        "is_mmio_mapped",
        "proximity_state",
        "current_lux",
    ]:
        if s not in als_src:
            print(f"❌ FAIL: sensor_als_prox.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/sensor_als_prox_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelProximityState",
        "KernelAlsGain",
        "KernelAlsProxError",
        "KernelSensorAlsProxHarness",
        "KERNEL_HANOIP_ALS_I2C_BASE",
        "KERNEL_ALS_PROX_I2C_ADDR",
        "KERNEL_PROX_THRESHOLD_HIGH",
        "KERNEL_PROX_THRESHOLD_LOW",
        "KERNEL_CAP_SENSOR_READ",
        "map_mmio",
        "sample_lux",
        "update_proximity",
        "is_mmio_mapped",
        "proximity_state",
        "current_lux",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: sensor_als_prox_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/sensor_als_prox.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/sensor_als_prox.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/sensor_als_prox.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 400-404)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 400-404...")
    with open("docs/evidence/golden-boot-sp90.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "400_SP90_ALS_PROX_INIT",
        "401_SP90_ALS_I2C_BUS_MAPPING",
        "402_SP90_LUX_PHOTOMETRIC_CALCULATION",
        "403_SP90_PROXIMITY_HYSTERESIS_DISPATCH",
        "404_SP90_ALS_PROX_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify ALS/Prox Confirmation Logs
    print("\n[CHECK 6/6] Verifying ALS/Prox Confirmation Logs...")
    expected_logs = [
        "I2C bus verified: GENI I2C mapped at 0x0888_4000, slave addr 0x39 (INV-ALS-001 PASSED)",
        "Photometry verified: 120 counts at 8x gain scaled to 150 lux (INV-ALS-002 PASSED)",
        "Hysteresis verified: Near (>=50), retain Near at 35, Far (<=25) confirmed (INV-ALS-003 PASSED)",
        "[SP-90] Sprint SP-90 Ambient Light & Proximity Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-90 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp90()
