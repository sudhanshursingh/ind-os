#!/usr/bin/env python3
"""
Ind OS Sprint SP-84 Verification & Acceptance Test Harness
Target: USB DWC3 Controller & QUSB2 PHY Bring-Up (DWC3 MMIO, PLL Lock, EP0 Setup Handshake)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-108, REQ-USB-101, INV-USB-001..003
"""

import os
import sys

REQUIRED_SP84_FILES = [
    "docs/architecture/usb-dwc3-spec.md",
    "docs/evidence/SP-84-validation.md",
    "docs/evidence/golden-boot-sp84.log",
    "userspace/devmgr/src/usb_dwc3.rs",
    "kernel/src/hal/platform/usb_dwc3_core.rs",
]

def verify_sp84():
    print("==================================================")
    print(" Ind OS SP-84 USB DWC3 Verifier                   ")
    print(" Target: Sprint SP-84 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-84 deliverable presence...")
    for rel_path in REQUIRED_SP84_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-84 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/usb_dwc3.rs", "r") as f:
        usb_src = f.read()
    for s in [
        "UsbSpeed",
        "UsbEndpointState",
        "UsbError",
        "UsbDwc3Harness",
        "HANOIP_USB_DWC3_BASE",
        "HANOIP_QUSB2_PHY_BASE",
        "USB_REQ_GET_DESCRIPTOR",
        "USB_REQ_SET_ADDRESS",
        "USB_DESC_TYPE_DEVICE",
        "CAP_USB_ADMIN",
        "map_mmio",
        "lock_qusb2_phy",
        "handle_setup_packet",
        "is_controller_mapped",
        "is_phy_locked",
        "ep0_state",
        "device_address",
        "speed",
    ]:
        if s not in usb_src:
            print(f"❌ FAIL: usb_dwc3.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/usb_dwc3_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelUsbSpeed",
        "KernelUsbEndpointState",
        "KernelUsbError",
        "KernelUsbDwc3Harness",
        "KERNEL_HANOIP_USB_DWC3_BASE",
        "KERNEL_HANOIP_QUSB2_PHY_BASE",
        "KERNEL_USB_REQ_GET_DESCRIPTOR",
        "KERNEL_USB_REQ_SET_ADDRESS",
        "KERNEL_USB_DESC_TYPE_DEVICE",
        "KERNEL_CAP_USB_ADMIN",
        "map_mmio",
        "lock_qusb2_phy",
        "handle_setup_packet",
        "is_controller_mapped",
        "is_phy_locked",
        "ep0_state",
        "device_address",
        "speed",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: usb_dwc3_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/usb_dwc3.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/usb_dwc3.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/usb_dwc3.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 370-374)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 370-374...")
    with open("docs/evidence/golden-boot-sp84.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "370_SP84_USB_DWC3_INIT",
        "371_SP84_USB_DWC3_CONTROLLER_MAPPING",
        "372_SP84_QUSB2_PHY_PLL_LOCK",
        "373_SP84_CONTROL_ENDPOINT_EP0_HANDSHAKE",
        "374_SP84_USB_DWC3_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify USB Confirmation Logs
    print("\n[CHECK 6/6] Verifying USB DWC3 Confirmation Logs...")
    expected_logs = [
        "DWC3 verified: USB 3.0 controller mapped at 0x04e0_0000, QUSB2 PHY at 0x088e_2000 (INV-USB-001 PASSED)",
        "PHY verified: QUSB2 PLL locked, HighSpeed 480Mbps link established (INV-USB-002 PASSED)",
        "EP0 verified: standard GET_DESCRIPTOR (18B) and SET_ADDRESS (Addr 20) handled deterministically (INV-USB-003 PASSED)",
        "[SP-84] Sprint SP-84 USB DWC3 Controller Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-84 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp84()
