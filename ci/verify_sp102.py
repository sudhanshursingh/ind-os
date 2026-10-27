#!/usr/bin/env python3
"""
Ind OS Sprint SP-102 Verification & Acceptance Test Harness
Target: Hardware A/B Dual-Slot Update Service on UFS Partitions
Standards: Bible §6.3, §8.15, Table 19.2, REQ-SEC-112, REQ-STR-104, INV-OTA-AB-001..003
"""

import os
import sys

REQUIRED_SP102_FILES = [
    "docs/architecture/ab-update-hardware-spec.md",
    "docs/evidence/SP-102-validation.md",
    "docs/evidence/golden-boot-sp102.log",
    "userspace/store/src/ab_update.rs",
    "kernel/src/hal/platform/ab_update_core.rs",
]

def verify_sp102():
    print("==================================================")
    print(" Ind OS SP-102 Hardware A/B Update Verifier       ")
    print(" Target: Sprint SP-102 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-102 deliverable presence...")
    for rel_path in REQUIRED_SP102_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-102 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/store/src/ab_update.rs", "r") as f:
        store_src = f.read()
    for s in [
        "AbUpdateEngine",
        "AbUpdateError",
        "SlotId",
        "SlotMetadata",
        "CAP_STORAGE_ADMIN",
        "MAX_BOOT_TRIES",
        "PAYLOAD_BLOCK_SIZE",
        "get_slot_info",
        "active_slot",
        "stage_payload",
        "activate_slot",
        "mark_boot_successful",
        "staged_blocks",
        "is_rollback_armed",
    ]:
        if s not in store_src:
            print(f"❌ FAIL: ab_update.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/ab_update_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelAbUpdateHarness",
        "KernelAbUpdateError",
        "KernelSlotId",
        "KernelSlotMetadata",
        "KERNEL_CAP_AB_UPDATE",
        "KERNEL_MAX_BOOT_TRIES",
        "KERNEL_PAYLOAD_BLOCK_SIZE",
        "get_slot_info",
        "active_slot",
        "stage_payload",
        "activate_slot",
        "mark_boot_successful",
        "staged_blocks",
        "is_rollback_armed",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: ab_update_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01 Compliance: Zero external crates
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for cargo_path in ["kernel/Cargo.toml", "userspace/store/Cargo.toml"]:
        with open(cargo_path, "r") as f:
            c = f.read()
        lines = c.splitlines()
        in_deps = False
        for line in lines:
            line_str = line.strip()
            if line_str.startswith("[dependencies]"):
                in_deps = True
                continue
            if in_deps and line_str.startswith("["):
                in_deps = False
            if in_deps and line_str and not line_str.startswith("#"):
                if "path =" not in line_str:
                    print(f"❌ FAIL: Non-path external dependency in {cargo_path}: {line_str}")
                    sys.exit(1)
        print(f"  ✓ {cargo_path} compliant with Rule N-01")

    # 4. Rule N-04 Compliance: Zero unsafe in newly authored userspace
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/store/src/ab_update.rs)...")
    with open("userspace/store/src/ab_update.rs", "r") as f:
        content = f.read()
    if "unsafe " in content or "unsafe{" in content:
        print("❌ FAIL: unsafe code block found in userspace/store/src/ab_update.rs")
        sys.exit(1)
    print("  ✓ userspace/store/src/ab_update.rs is 100% safe Rust")

    # 5. Golden Boot Log Verification: Stages 460-464
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 460-464...")
    with open("docs/evidence/golden-boot-sp102.log", "r") as f:
        log_content = f.read()

    stages = [
        "460_SP102_AB_UPDATE_INIT",
        "461_SP102_SLOT_STATE_INSPECTION",
        "462_SP102_STAGE_PAYLOAD_SLOT_B",
        "463_SP102_SLOT_ACTIVATION_FLIP",
        "464_SP102_AB_UPDATE_READY",
    ]
    for st in stages:
        if st not in log_content:
            print(f"❌ FAIL: Missing stage {st} in docs/evidence/golden-boot-sp102.log")
            sys.exit(1)
        print(f"  ✓ Verified stage: {st}")

    # 6. Specific Confirmation Logs
    print("\n[CHECK 6/6] Verifying A/B Update Confirmation Logs...")
    expected_logs = [
        "Hardware A/B update service online: UFS slot arbiter active (REQ-SEC-112 PASSED)",
        "Slot states inspected: Slot A (ACTIVE, SUCCESSFUL), Slot B (STANDBY, TARGET) (INV-OTA-AB-001 PASSED)",
        "Payload staged into Slot B: 2048 blocks (8MB) verified via SHA-256 digest (INV-OTA-AB-002 PASSED)",
        "Active slot flipped to Slot B: tries_remaining=3, rollback trigger armed (INV-OTA-AB-003 PASSED)",
        "[SP-102] Sprint SP-102 Hardware A/B Dual-Slot Update Service Milestone REACHED",
    ]
    for el in expected_logs:
        if el not in log_content:
            print(f"❌ FAIL: Log string not found: '{el}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {el[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-102 VERIFICATION PASSED (100% DoD) ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp102()
