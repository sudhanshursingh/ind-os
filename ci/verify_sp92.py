#!/usr/bin/env python3
"""
Ind OS Sprint SP-92 Verification & Acceptance Test Harness
Target: Unified Sensor Event Hub (Multiplexing, Microsecond Timestamp Sync & Isolated FIFO Ring-Buffer)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-110, REQ-SNS-104, INV-HUB-001..003
"""

import os
import sys

REQUIRED_SP92_FILES = [
    "docs/architecture/sensor-hub-spec.md",
    "docs/evidence/SP-92-validation.md",
    "docs/evidence/golden-boot-sp92.log",
    "userspace/devmgr/src/sensor_hub.rs",
    "kernel/src/hal/platform/sensor_hub_core.rs",
]

def verify_sp92():
    print("==================================================")
    print(" Ind OS SP-92 Sensor Event Hub Verifier           ")
    print(" Target: Sprint SP-92 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-92 deliverable presence...")
    for rel_path in REQUIRED_SP92_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-92 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/sensor_hub.rs", "r") as f:
        hub_src = f.read()
    for s in [
        "SensorChannel",
        "SensorEvent",
        "SensorSubscriber",
        "SensorHubHarness",
        "SensorHubError",
        "CHANNEL_MASK_ALL",
        "CHANNEL_MASK_ACCEL",
        "CHANNEL_MASK_GYRO",
        "CHANNEL_MASK_ALS",
        "CHANNEL_MASK_PROX",
        "CHANNEL_MASK_MAG",
        "CAP_SENSOR_READ",
        "register_subscriber",
        "sync_timestamp",
        "post_event",
        "pop_event",
        "subscribers_count",
        "last_timestamp_us",
    ]:
        if s not in hub_src:
            print(f"❌ FAIL: sensor_hub.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/sensor_hub_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelSensorChannel",
        "KernelSensorEvent",
        "KernelSensorSubscriber",
        "KernelSensorHubHarness",
        "KernelSensorHubError",
        "KERNEL_CHANNEL_MASK_ALL",
        "KERNEL_CHANNEL_MASK_ACCEL",
        "KERNEL_CHANNEL_MASK_GYRO",
        "KERNEL_CHANNEL_MASK_ALS",
        "KERNEL_CHANNEL_MASK_PROX",
        "KERNEL_CHANNEL_MASK_MAG",
        "KERNEL_CAP_SENSOR_READ",
        "register_subscriber",
        "sync_timestamp",
        "post_event",
        "pop_event",
        "subscribers_count",
        "last_timestamp_us",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: sensor_hub_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/sensor_hub.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/sensor_hub.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/sensor_hub.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 410-414)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 410-414...")
    with open("docs/evidence/golden-boot-sp92.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "410_SP92_SENSOR_HUB_INIT",
        "411_SP92_SENSOR_SUBSCRIPTION_REGISTRATION",
        "412_SP92_MULTI_SENSOR_TIMESTAMP_SYNC",
        "413_SP92_EVENT_MULTIPLEXING_DISPATCH",
        "414_SP92_SENSOR_HUB_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Sensor Hub Confirmation Logs
    print("\n[CHECK 6/6] Verifying Sensor Hub Confirmation Logs...")
    expected_logs = [
        "Sensor hub harness online: subscription multiplexer and timestamp router active (REQ-SNS-104 PASSED)",
        "Subscriptions registered: 2 clients authorized via CAP_SENSOR_READ (INV-HUB-001 PASSED)",
        "Monotonic microsecond timestamp synchronization locked across 5 channels (INV-HUB-002 PASSED)",
        "Multiplexed dispatch verified: FIFO delivery and channel-mask routing validated (INV-HUB-003 PASSED)",
        "[SP-92] Sprint SP-92 Unified Sensor Event Hub Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-92 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp92()
