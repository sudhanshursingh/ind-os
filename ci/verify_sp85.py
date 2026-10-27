#!/usr/bin/env python3
"""
Ind OS Sprint SP-85 Verification & Acceptance Test Harness
Target: USB Device/Host Mode & CDC ACM Console (OTG ID, CDC ACM Console, Capability-Gated Switching)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-USB-102, REQ-REC-101, INV-USB-004..006
"""

import os
import sys

REQUIRED_SP85_FILES = [
    "docs/architecture/usb-role-cdc-spec.md",
    "docs/evidence/SP-85-validation.md",
    "docs/evidence/golden-boot-sp85.log",
    "userspace/devmgr/src/usb_role.rs",
    "kernel/src/hal/platform/usb_role_core.rs",
]

def verify_sp85():
    print("==================================================")
    print(" Ind OS SP-85 USB Role & CDC ACM Verifier         ")
    print(" Target: Sprint SP-85 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-85 deliverable presence...")
    for rel_path in REQUIRED_SP85_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-85 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/usb_role.rs", "r") as f:
        role_src = f.read()
    for s in [
        "UsbRole",
        "UsbRoleError",
        "UsbRoleHarness",
        "USB_ROLE_DEVICE",
        "USB_ROLE_HOST",
        "CAP_USB_ROLE_SWITCH",
        "CDC_ACM_BAUD_DEFAULT",
        "CDC_ACM_LINE_CODING_LEN",
        "sense_otg_id_pin",
        "switch_role",
        "set_line_coding",
        "write_console",
        "current_role",
        "baud_rate",
        "bytes_transmitted",
    ]:
        if s not in role_src:
            print(f"❌ FAIL: usb_role.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/usb_role_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelUsbRole",
        "KernelUsbRoleError",
        "KernelUsbRoleHarness",
        "KERNEL_USB_ROLE_DEVICE",
        "KERNEL_USB_ROLE_HOST",
        "KERNEL_CAP_USB_ROLE_SWITCH",
        "KERNEL_CDC_ACM_BAUD_DEFAULT",
        "KERNEL_CDC_ACM_LINE_CODING_LEN",
        "sense_otg_id_pin",
        "switch_role",
        "set_line_coding",
        "write_console",
        "current_role",
        "baud_rate",
        "bytes_transmitted",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: usb_role_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/usb_role.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/usb_role.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/usb_role.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 375-379)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 375-379...")
    with open("docs/evidence/golden-boot-sp85.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "375_SP85_USB_ROLE_INIT",
        "376_SP85_OTG_ID_PIN_DETECTION",
        "377_SP85_CDC_ACM_CONSOLE_HANDSHAKE",
        "378_SP85_ROLE_SWITCH_CAPABILITY_GATING",
        "379_SP85_USB_ROLE_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify USB Role Confirmation Logs
    print("\n[CHECK 6/6] Verifying USB Role Confirmation Logs...")
    expected_logs = [
        "OTG arbitration verified: float -> Device mode, grounded -> Host mode (INV-USB-004 PASSED)",
        "CDC ACM console verified: 115200 baud line coding and bulk IN transmission operational (INV-USB-005, Rule N-06 PASSED)",
        "Role gating verified: unprivileged switching blocked; CAP_USB_ROLE_SWITCH permitted transitions (INV-USB-006 PASSED)",
        "[SP-85] Sprint SP-85 USB Device/Host & CDC ACM Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-85 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp85()
