#!/usr/bin/env python3
"""
Ind OS Sprint SP-74 Verification & Acceptance Test Harness
Target: Qualcomm UFS 2.2 Storage Controller Bring-Up (UFSHCI, UTP, LUN I/O)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-105, REQ-STR-101, INV-UFS-001..003
"""

import os
import sys

REQUIRED_SP74_FILES = [
    "docs/architecture/ufs-driver-spec.md",
    "docs/evidence/SP-74-validation.md",
    "docs/evidence/golden-boot-sp74.log",
    "userspace/devmgr/src/ufs.rs",
    "kernel/src/hal/platform/ufs_core.rs",
]

def verify_sp74():
    print("==================================================")
    print(" Ind OS SP-74 Qualcomm UFS 2.2 Storage Verifier   ")
    print(" Target: Sprint SP-74 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-74 deliverable presence...")
    for rel_path in REQUIRED_SP74_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-74 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/ufs.rs", "r") as f:
        ufs_src = f.read()
    for s in [
        "UfsDriverError",
        "UfsControllerHarness",
        "HANOIP_UFS_BASE",
        "CAP_STORAGE_ADMIN",
        "map_mmio",
        "submit_utp_transfer",
    ]:
        if s not in ufs_src:
            print(f"❌ FAIL: ufs.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/ufs_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelUfsDriverError",
        "KernelUfsControllerHarness",
        "KERNEL_HANOIP_UFS_BASE",
        "KERNEL_CAP_STORAGE_ADMIN",
        "map_mmio",
        "submit_utp_transfer",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: ufs_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/ufs.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/ufs.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/ufs.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 320-324)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 320-324...")
    with open("docs/evidence/golden-boot-sp74.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "320_SP74_UFS_DRIVER_INIT",
        "321_SP74_UFSHCI_REGISTER_MAPPING",
        "322_SP74_UTP_TRANSFER_CAPABILITY_GATING",
        "323_SP74_UFS_LUN_BLOCK_IO",
        "324_SP74_UFS_DRIVER_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify UFS Storage Confirmation Logs
    print("\n[CHECK 6/6] Verifying UFS Storage Confirmation Logs...")
    expected_logs = [
        "UFSHCI verified: controller registers mapped at 0x01d8_4000 (INV-UFS-001 PASSED)",
        "Capability gating verified: unprivileged or malformed transfers strictly rejected (INV-UFS-002 PASSED)",
        "LUN block I/O verified: 16 blocks transferred to LBA 0x1000 (INV-UFS-003 PASSED)",
        "[SP-74] Sprint SP-74 Qualcomm UFS 2.2 Storage Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-74 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp74()
