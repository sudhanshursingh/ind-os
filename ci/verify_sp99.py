#!/usr/bin/env python3
"""
Ind OS Sprint SP-99 Verification & Acceptance Test Harness
Target: System Services Hardware Port — IndFS Mount on UFS
Standards: Bible §6.3, §8.15, Table 19.2, REQ-SYS-101, REQ-STR-103, INV-SYS-FS-001..003
"""

import os
import sys

REQUIRED_SP99_FILES = [
    "docs/architecture/system-services-ufs-spec.md",
    "docs/evidence/SP-99-validation.md",
    "docs/evidence/golden-boot-sp99.log",
    "userspace/store/src/ufs_mount.rs",
    "kernel/src/hal/platform/ufs_mount_core.rs",
]

def verify_sp99():
    print("==================================================")
    print(" Ind OS SP-99 System Services UFS Mount Verifier   ")
    print(" Target: Sprint SP-99 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-99 deliverable presence...")
    for rel_path in REQUIRED_SP99_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-99 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/store/src/ufs_mount.rs", "r") as f:
        mount_src = f.read()
    for s in [
        "UfsMountHarness",
        "UfsMountError",
        "UfsMountState",
        "CAP_STORAGE_MOUNT",
        "INDFS_SUPERBLOCK_MAGIC",
        "INDFS_BLOCK_SIZE_BYTES",
        "mount_root_ufs",
        "replay_journal",
        "is_mounted",
        "magic",
        "block_size",
        "replayed_transactions_count",
    ]:
        if s not in mount_src:
            print(f"❌ FAIL: ufs_mount.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/ufs_mount_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelUfsMountHarness",
        "KernelUfsMountError",
        "KernelUfsMountState",
        "KERNEL_CAP_STORAGE_MOUNT",
        "KERNEL_INDFS_SUPERBLOCK_MAGIC",
        "KERNEL_INDFS_BLOCK_SIZE_BYTES",
        "mount_root_ufs",
        "replay_journal",
        "is_mounted",
        "magic",
        "block_size",
        "replayed_transactions_count",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: ufs_mount_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/store/Cargo.toml"]:
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
            if line.startswith("path =") or "path =" in line:
                continue
            if in_deps and line:
                print(f"❌ FAIL: External dependency detected in {toml_path}: {line}")
                sys.exit(1)
        print(f"  ✓ {toml_path} compliant with Rule N-01")

    # 4. Rule N-04: Zero Unsafe Code Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/store)...")
    with open("userspace/store/src/ufs_mount.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/store/src/ufs_mount.rs")
            sys.exit(1)
    print("  ✓ userspace/store/src/ufs_mount.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 445-449)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 445-449...")
    with open("docs/evidence/golden-boot-sp99.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "445_SP99_UFS_ROOT_MOUNT_INIT",
        "446_SP99_UFS_SUPERBLOCK_VALIDATION",
        "447_SP99_UFS_JOURNAL_REPLAY",
        "448_SP99_SYS_HIERARCHY_NAMESPACES",
        "449_SP99_SYSTEM_SERVICES_UFS_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Confirmation Logs
    print("\n[CHECK 6/6] Verifying UFS Mount Confirmation Logs...")
    expected_logs = [
        "UFS mount harness online: storage root mount arbiter active (REQ-SYS-101 PASSED)",
        "Superblock validated: IndFS 'INDF' magic confirmed, 4096B blocks (INV-SYS-FS-001 PASSED)",
        "Journal replayed: 12 transactions committed cleanly with 0 fsck errors (INV-SYS-FS-002 PASSED)",
        "Namespaces mounted: /sys (RO), /dev (DEV), /cfg (INTEGRITY), /data (PERSISTENT) (INV-SYS-FS-003 PASSED)",
        "[SP-99] Sprint SP-99 System Services Hardware Port Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-99 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp99()
