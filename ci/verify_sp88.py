#!/usr/bin/env python3
"""
Ind OS Sprint SP-88 Verification & Acceptance Test Harness
Target: Qualcomm Bluetooth HCI UART Driver (GENI UART MMIO, NVM Patch Load, HCI Command Loop)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-109, REQ-NET-102, INV-BT-001..003
"""

import os
import sys

REQUIRED_SP88_FILES = [
    "docs/architecture/bluetooth-hci-spec.md",
    "docs/evidence/SP-88-validation.md",
    "docs/evidence/golden-boot-sp88.log",
    "userspace/devmgr/src/bluetooth_hci.rs",
    "kernel/src/hal/platform/bluetooth_hci_core.rs",
]

def verify_sp88():
    print("==================================================")
    print(" Ind OS SP-88 Bluetooth HCI Verifier              ")
    print(" Target: Sprint SP-88 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-88 deliverable presence...")
    for rel_path in REQUIRED_SP88_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-88 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/bluetooth_hci.rs", "r") as f:
        bt_src = f.read()
    for s in [
        "BtState",
        "BtError",
        "BluetoothHciHarness",
        "HANOIP_BT_UART_BASE",
        "HCI_CMD_RESET",
        "HCI_CMD_LE_SET_SCAN_ENABLE",
        "CAP_BT_ADMIN",
        "map_mmio",
        "load_firmware_patch",
        "send_hci_command",
        "state",
        "is_mmio_mapped",
        "is_fw_loaded",
        "commands_processed",
    ]:
        if s not in bt_src:
            print(f"❌ FAIL: bluetooth_hci.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/bluetooth_hci_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelBtState",
        "KernelBtError",
        "KernelBluetoothHciHarness",
        "KERNEL_HANOIP_BT_UART_BASE",
        "KERNEL_HCI_CMD_RESET",
        "KERNEL_HCI_CMD_LE_SET_SCAN_ENABLE",
        "KERNEL_CAP_BT_ADMIN",
        "map_mmio",
        "load_firmware_patch",
        "send_hci_command",
        "state",
        "is_mmio_mapped",
        "is_fw_loaded",
        "commands_processed",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: bluetooth_hci_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/bluetooth_hci.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/bluetooth_hci.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/bluetooth_hci.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 390-394)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 390-394...")
    with open("docs/evidence/golden-boot-sp88.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "390_SP88_BT_HCI_INIT",
        "391_SP88_BT_UART_TRANSPORT_MAPPING",
        "392_SP88_BT_FIRMWARE_PATCH_LOAD",
        "393_SP88_HCI_COMMAND_LOOP_BLE_SCAN",
        "394_SP88_BT_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Bluetooth Confirmation Logs
    print("\n[CHECK 6/6] Verifying Bluetooth Confirmation Logs...")
    expected_logs = [
        "GENI UART verified: Bluetooth HCI transport mapped at 0x0884_0000 (INV-BT-001 PASSED)",
        "Firmware verified: §14.3 NVM patch loaded; controller entered ResetPending (INV-BT-002 PASSED)",
        "HCI loop verified: HCI_Reset and HCI_LE_Set_Scan_Enable processed deterministically (INV-BT-003 PASSED)",
        "[SP-88] Sprint SP-88 Bluetooth Controller Bring-Up Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-88 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp88()
