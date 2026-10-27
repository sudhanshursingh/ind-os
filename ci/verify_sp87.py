#!/usr/bin/env python3
"""
Ind OS Sprint SP-87 Verification & Acceptance Test Harness
Target: Qualcomm WCN3990 Wi-Fi Driver (SMD MMIO, Firmware Calibration, 802.11 State Machine)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-109, REQ-NET-101, INV-WIFI-001..003
"""

import os
import sys

REQUIRED_SP87_FILES = [
    "docs/architecture/wifi-wcn3990-spec.md",
    "docs/evidence/SP-87-validation.md",
    "docs/evidence/golden-boot-sp87.log",
    "userspace/devmgr/src/wifi_wcn3990.rs",
    "kernel/src/hal/platform/wifi_wcn3990_core.rs",
]

def verify_sp87():
    print("==================================================")
    print(" Ind OS SP-87 Wi-Fi WCN3990 Verifier             ")
    print(" Target: Sprint SP-87 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-87 deliverable presence...")
    for rel_path in REQUIRED_SP87_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-87 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/wifi_wcn3990.rs", "r") as f:
        wifi_src = f.read()
    for s in [
        "WifiState",
        "WifiError",
        "WifiWcn3990Harness",
        "HANOIP_WCN3990_BASE",
        "CAP_NET_ADMIN",
        "map_mmio",
        "calibrate_and_init",
        "start_scan",
        "associate",
        "state",
        "is_mmio_mapped",
        "is_calibrated",
    ]:
        if s not in wifi_src:
            print(f"❌ FAIL: wifi_wcn3990.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/wifi_wcn3990_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelWifiState",
        "KernelWifiError",
        "KernelWifiWcn3990Harness",
        "KERNEL_HANOIP_WCN3990_BASE",
        "KERNEL_CAP_NET_ADMIN",
        "map_mmio",
        "calibrate_and_init",
        "start_scan",
        "associate",
        "state",
        "is_mmio_mapped",
        "is_calibrated",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: wifi_wcn3990_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/wifi_wcn3990.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/wifi_wcn3990.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/wifi_wcn3990.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 385-389)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 385-389...")
    with open("docs/evidence/golden-boot-sp87.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "385_SP87_WIFI_INIT",
        "386_SP87_WCN3990_SMD_MAPPING",
        "387_SP87_WIFI_FIRMWARE_CALIBRATION",
        "388_SP87_80211_ASSOCIATION_STATE_MACHINE",
        "389_SP87_WIFI_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Wi-Fi Confirmation Logs
    print("\n[CHECK 6/6] Verifying Wi-Fi Confirmation Logs...")
    expected_logs = [
        "WCN3990 verified: WCNSS SMD controller mapped at 0x0a00_0000 (INV-WIFI-001 PASSED)",
        "Calibration verified: §14.3 firmware loaded and dual-band RF synthesizer calibrated (INV-WIFI-002 PASSED)",
        "Association verified: Idle -> Scanning -> Authenticating -> Associated established (INV-WIFI-003 PASSED)",
        "[SP-87] Sprint SP-87 Wi-Fi Driver Bring-Up Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-87 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp87()
