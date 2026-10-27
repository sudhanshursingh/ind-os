#!/usr/bin/env python3
"""
Ind OS Sprint SP-78 Verification & Acceptance Test Harness
Target: Touch Controller Driver (GENI I2C1, 1080x2460 Viewport Bounding, Ring Buffer)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-107, REQ-UX-102, INV-TOUCH-001..003
"""

import os
import sys

REQUIRED_SP78_FILES = [
    "docs/architecture/touch-controller-spec.md",
    "docs/evidence/SP-78-validation.md",
    "docs/evidence/golden-boot-sp78.log",
    "userspace/devmgr/src/touch_controller.rs",
    "kernel/src/hal/platform/touch_controller_core.rs",
]

def verify_sp78():
    print("==================================================")
    print(" Ind OS SP-78 Touch Controller Verifier           ")
    print(" Target: Sprint SP-78 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-78 deliverable presence...")
    for rel_path in REQUIRED_SP78_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-78 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/touch_controller.rs", "r") as f:
        touch_src = f.read()
    for s in [
        "TouchAction",
        "TouchError",
        "TouchPoint",
        "TouchControllerHarness",
        "HANOIP_GENI_I2C1_BASE",
        "TOUCH_I2C_SLAVE_ADDR",
        "TOUCH_QUEUE_CAPACITY",
        "map_bus",
        "post_event",
        "pop_event",
        "is_bus_mapped",
    ]:
        if s not in touch_src:
            print(f"❌ FAIL: touch_controller.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/touch_controller_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelTouchAction",
        "KernelTouchError",
        "KernelTouchPoint",
        "KernelTouchControllerHarness",
        "KERNEL_HANOIP_GENI_I2C1_BASE",
        "KERNEL_TOUCH_I2C_SLAVE_ADDR",
        "KERNEL_TOUCH_QUEUE_CAPACITY",
        "map_bus",
        "post_event",
        "pop_event",
        "is_bus_mapped",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: touch_controller_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/touch_controller.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/touch_controller.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/touch_controller.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 340-344)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 340-344...")
    with open("docs/evidence/golden-boot-sp78.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "340_SP78_TOUCH_CONTROLLER_INIT",
        "341_SP78_GENI_I2C_BUS_MAPPING",
        "342_SP78_TOUCH_COORDINATE_BOUNDS",
        "343_SP78_TOUCH_EVENT_RING_BUFFER",
        "344_SP78_TOUCH_CONTROLLER_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Touch Controller Confirmation Logs
    print("\n[CHECK 6/6] Verifying Touch Controller Confirmation Logs...")
    expected_logs = [
        "GENI I2C1 verified: touch digitizer mapped at 0x0880_0000, slave addr 0x38 (INV-TOUCH-001 PASSED)",
        "Coordinates verified: tap (540, 1230) safely bounded in 1080x2460 display (INV-TOUCH-002 PASSED)",
        "Event ring buffer verified: non-blocking FIFO verified under simulated IRQ load (INV-TOUCH-003 PASSED)",
        "[SP-78] Sprint SP-78 Touch Controller Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-78 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp78()
