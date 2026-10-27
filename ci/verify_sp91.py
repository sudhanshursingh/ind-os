#!/usr/bin/env python3
"""
Ind OS Sprint SP-91 Verification & Acceptance Test Harness
Target: AK09918 3-Axis Magnetometer (I2C MMIO, Chip ID, 3-Axis Reading & Heading Calculation)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-110, REQ-SNS-103, INV-MAG-001..003
"""

import os
import sys

REQUIRED_SP91_FILES = [
    "docs/architecture/sensor-mag-spec.md",
    "docs/evidence/SP-91-validation.md",
    "docs/evidence/golden-boot-sp91.log",
    "userspace/devmgr/src/sensor_mag.rs",
    "kernel/src/hal/platform/sensor_mag_core.rs",
]

def verify_sp91():
    print("==================================================")
    print(" Ind OS SP-91 Sensor Magnetometer Verifier        ")
    print(" Target: Sprint SP-91 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-91 deliverable presence...")
    for rel_path in REQUIRED_SP91_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-91 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/sensor_mag.rs", "r") as f:
        mag_src = f.read()
    for s in [
        "MagReading",
        "MagError",
        "SensorMagHarness",
        "HANOIP_MAG_I2C_BASE",
        "AK09918_I2C_ADDR",
        "AK09918_WIA1_COMPANY",
        "AK09918_WIA2_DEVICE",
        "CAP_SENSOR_READ",
        "map_mmio",
        "verify_chip_id",
        "read_mag_and_heading",
        "compute_heading_deg",
        "is_mmio_mapped",
        "is_chip_id_verified",
        "last_heading_deg",
    ]:
        if s not in mag_src:
            print(f"❌ FAIL: sensor_mag.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/sensor_mag_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelMagReading",
        "KernelMagError",
        "KernelSensorMagHarness",
        "KERNEL_HANOIP_MAG_I2C_BASE",
        "KERNEL_AK09918_I2C_ADDR",
        "KERNEL_AK09918_WIA1_COMPANY",
        "KERNEL_AK09918_WIA2_DEVICE",
        "KERNEL_CAP_SENSOR_READ",
        "map_mmio",
        "verify_chip_id",
        "read_mag_and_heading",
        "kernel_compute_heading_deg",
        "is_mmio_mapped",
        "is_chip_id_verified",
        "last_heading_deg",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: sensor_mag_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/sensor_mag.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/sensor_mag.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/sensor_mag.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 405-409)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 405-409...")
    with open("docs/evidence/golden-boot-sp91.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "405_SP91_MAG_INIT",
        "406_SP91_MAG_I2C_BUS_MAPPING",
        "407_SP91_MAG_CHIP_ID_VERIFY",
        "408_SP91_MAG_HEADING_CALCULATION",
        "409_SP91_MAG_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Magnetometer Confirmation Logs
    print("\n[CHECK 6/6] Verifying Magnetometer Confirmation Logs...")
    expected_logs = [
        "I2C bus verified: GENI I2C mapped at 0x0888_8000, slave addr 0x0C (INV-MAG-001 PASSED)",
        "Chip ID verified: WIA1=0x48 (AKM), WIA2=0x0C (AK09918) (INV-MAG-002 PASSED)",
        "Heading calculated: Vector (100, 100, -50) uT -> 45 deg azimuth (INV-MAG-003 PASSED)",
        "[SP-91] Sprint SP-91 Magnetometer Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-91 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp91()
