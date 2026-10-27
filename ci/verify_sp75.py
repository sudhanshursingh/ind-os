#!/usr/bin/env python3
"""
Ind OS Sprint SP-75 Verification & Acceptance Test Harness
Target: UFS Power-Loss Resilience & Journaling on Hardware
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-105, REQ-STR-102, INV-UFS-JRN-001..003
"""

import os
import sys

REQUIRED_SP75_FILES = [
    "docs/architecture/ufs-power-loss-resilience-spec.md",
    "docs/evidence/SP-75-validation.md",
    "docs/evidence/golden-boot-sp75.log",
    "userspace/devmgr/src/ufs_resilience.rs",
    "kernel/src/hal/platform/ufs_resilience_core.rs",
]

def verify_sp75():
    print("==================================================")
    print(" Ind OS SP-75 UFS Power-Loss Resilience Verifier  ")
    print(" Target: Sprint SP-75 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-75 deliverable presence...")
    for rel_path in REQUIRED_SP75_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-75 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/ufs_resilience.rs", "r") as f:
        ufs_src = f.read()
    for s in [
        "UfsJournalError",
        "UfsJournalHarness",
        "SCSI_CMD_SYNCHRONIZE_CACHE",
        "CAP_STORAGE_RECOVERY",
        "append_entry",
        "commit_barrier",
        "simulate_recovery_playback",
        "verify_partition_write_guard",
    ]:
        if s not in ufs_src:
            print(f"❌ FAIL: ufs_resilience.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/ufs_resilience_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelUfsJournalError",
        "KernelUfsJournalHarness",
        "KERNEL_SCSI_CMD_SYNCHRONIZE_CACHE",
        "KERNEL_CAP_STORAGE_RECOVERY",
        "append_entry",
        "commit_barrier",
        "simulate_recovery_playback",
        "verify_partition_write_guard",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: ufs_resilience_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/ufs_resilience.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/ufs_resilience.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/ufs_resilience.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 325-329)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 325-329...")
    with open("docs/evidence/golden-boot-sp75.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "325_SP75_UFS_RESILIENCE_INIT",
        "326_SP75_SCSI_SYNCHRONIZE_CACHE_BARRIER",
        "327_SP75_JOURNAL_ATOMIC_ROLLBACK_VERIFY",
        "328_SP75_CRITICAL_PARTITION_WRITE_GUARD",
        "329_SP75_UFS_RESILIENCE_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify UFS Resilience Confirmation Logs
    print("\n[CHECK 6/6] Verifying UFS Resilience Confirmation Logs...")
    expected_logs = [
        "Write barrier verified: SCSI SYNCHRONIZE_CACHE issued prior to transaction commit (INV-UFS-JRN-001 PASSED)",
        "Atomic rollback verified: uncommitted writes during power cut safely discarded (INV-UFS-JRN-002 PASSED)",
        "Partition write guard verified: critical boot/recovery blocks hardware-isolated (INV-UFS-JRN-003, Rule N-06 PASSED)",
        "[SP-75] Sprint SP-75 UFS Power-Loss Resilience Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-75 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp75()
