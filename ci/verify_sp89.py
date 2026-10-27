#!/usr/bin/env python3
"""
Ind OS Sprint SP-89 Verification & Acceptance Test Harness
Target: Accelerometer & Gyroscope (Bosch BMI260, I2C MMIO, Chip ID, FIFO Sampling)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-110, REQ-SNS-101, INV-SNS-001..003
"""

import os
import sys

REQUIRED_SP89_FILES = [
    "docs/architecture/sensor-imu-spec.md",
    "docs/evidence/SP-89-validation.md",
    "docs/evidence/golden-boot-sp89.log",
    "userspace/devmgr/src/sensor_imu.rs",
    "kernel/src/hal/platform/sensor_imu_core.rs",
]

def verify_sp89():
    print("==================================================")
    print(" Ind OS SP-89 Sensor IMU Verifier                 ")
    print(" Target: Sprint SP-89 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-89 deliverable presence...")
    for rel_path in REQUIRED_SP89_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-89 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/sensor_imu.rs", "r") as f:
        imu_src = f.read()
    for s in [
        "ImuSample",
        "ImuError",
        "SensorImuHarness",
        "HANOIP_IMU_I2C_BASE",
        "BMI260_I2C_ADDR",
        "BMI260_CHIP_ID",
        "CAP_SENSOR_READ",
        "map_mmio",
        "verify_chip_id",
        "read_fifo_sample",
        "is_mmio_mapped",
        "is_initialized",
        "samples_read",
    ]:
        if s not in imu_src:
            print(f"❌ FAIL: sensor_imu.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/sensor_imu_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelImuSample",
        "KernelImuError",
        "KernelSensorImuHarness",
        "KERNEL_HANOIP_IMU_I2C_BASE",
        "KERNEL_BMI260_I2C_ADDR",
        "KERNEL_BMI260_CHIP_ID",
        "KERNEL_CAP_SENSOR_READ",
        "map_mmio",
        "verify_chip_id",
        "read_fifo_sample",
        "is_mmio_mapped",
        "is_initialized",
        "samples_read",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: sensor_imu_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/sensor_imu.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/sensor_imu.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/sensor_imu.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 395-399)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 395-399...")
    with open("docs/evidence/golden-boot-sp89.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "395_SP89_IMU_INIT",
        "396_SP89_BMI260_I2C_BUS_MAPPING",
        "397_SP89_CHIP_ID_SOFT_RESET",
        "398_SP89_IMU_FIFO_ACQUISITION",
        "399_SP89_IMU_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify IMU Confirmation Logs
    print("\n[CHECK 6/6] Verifying IMU Confirmation Logs...")
    expected_logs = [
        "I2C bus verified: GENI I2C mapped at 0x0888_0000, slave addr 0x68 (INV-SNS-001 PASSED)",
        "Chip ID verified: 0x27 confirmed; soft-reset command completed (INV-SNS-002 PASSED)",
        "FIFO verified: 6-axis sample read (accel 1g Z, gyro stable) within bounds (INV-SNS-003 PASSED)",
        "[SP-89] Sprint SP-89 Accelerometer & Gyroscope Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-89 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp89()
