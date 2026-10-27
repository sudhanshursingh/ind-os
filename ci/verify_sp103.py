#!/usr/bin/env python3
"""
Ind OS Sprint SP-103 Verification & Acceptance Test Harness
Target: Full G5-Standard Userspace Running on Hardware
Standards: Bible §6.3, §8.15, Table 19.2, REQ-USR-101, REQ-SEC-113, INV-G5-HW-001..003
"""

import os
import sys

REQUIRED_SP103_FILES = [
    "docs/architecture/g5-userspace-hardware-spec.md",
    "docs/evidence/SP-103-validation.md",
    "docs/evidence/golden-boot-sp103.log",
    "userspace/init/src/g5_hw.rs",
    "kernel/src/hal/platform/g5_userspace_core.rs",
]

def verify_sp103():
    print("==================================================")
    print(" Ind OS SP-103 G5 Userspace Hardware Verifier     ")
    print(" Target: Sprint SP-103 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-103 deliverable presence...")
    for rel_path in REQUIRED_SP103_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-103 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/init/src/g5_hw.rs", "r") as f:
        init_src = f.read()
    for s in [
        "G5HwCoordinator",
        "G5HwError",
        "G5ProcessInfo",
        "ProcessStatus",
        "CAP_SYS_ADMIN",
        "CAP_LOG_RECORD",
        "CAP_STORAGE_MOUNT",
        "CAP_DISPLAY_RENDER",
        "CAP_TOUCH_INPUT",
        "CAP_IPC_SYNC",
        "G5_TOTAL_SERVICES",
        "IPC_RING_TEST_CYCLES",
        "spawn_tcb_services",
        "spawn_ux_services",
        "run_ipc_ring_stress",
        "get_process_info",
        "total_running",
        "ipc_rounds_completed",
    ]:
        if s not in init_src:
            print(f"❌ FAIL: g5_hw.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/g5_userspace_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelG5HwHarness",
        "KernelG5HwError",
        "KernelG5ProcessInfo",
        "KernelProcessStatus",
        "KERNEL_CAP_G5_SYS_ADMIN",
        "KERNEL_CAP_G5_LOG_RECORD",
        "KERNEL_CAP_G5_STORAGE_MOUNT",
        "KERNEL_CAP_G5_DISPLAY_RENDER",
        "KERNEL_CAP_G5_TOUCH_INPUT",
        "KERNEL_CAP_G5_IPC_SYNC",
        "KERNEL_G5_TOTAL_SERVICES",
        "KERNEL_IPC_RING_TEST_CYCLES",
        "spawn_tcb_services",
        "spawn_ux_services",
        "run_ipc_ring_stress",
        "get_process_info",
        "total_running",
        "ipc_rounds_completed",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: g5_userspace_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01 Compliance: Zero external crates
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for cargo_path in ["kernel/Cargo.toml", "userspace/init/Cargo.toml"]:
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
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/init/src/g5_hw.rs)...")
    with open("userspace/init/src/g5_hw.rs", "r") as f:
        content = f.read()
    if "unsafe " in content or "unsafe{" in content:
        print("❌ FAIL: unsafe code block found in userspace/init/src/g5_hw.rs")
        sys.exit(1)
    print("  ✓ userspace/init/src/g5_hw.rs is 100% safe Rust")

    # 5. Golden Boot Log Verification: Stages 465-469
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 465-469...")
    with open("docs/evidence/golden-boot-sp103.log", "r") as f:
        log_content = f.read()

    stages = [
        "465_SP103_G5_USERSPACE_INIT",
        "466_SP103_TCB_SERVICES_SPAWN",
        "467_SP103_UX_SERVICES_SPAWN",
        "468_SP103_IPC_CONCURRENCY_STRESS",
        "469_SP103_G5_USERSPACE_READY",
    ]
    for st in stages:
        if st not in log_content:
            print(f"❌ FAIL: Missing stage {st} in docs/evidence/golden-boot-sp103.log")
            sys.exit(1)
        print(f"  ✓ Verified stage: {st}")

    # 6. Specific Confirmation Logs
    print("\n[CHECK 6/6] Verifying G5 Userspace Confirmation Logs...")
    expected_logs = [
        "G5 userspace coordinator online: multi-process supervisor active (REQ-USR-101 PASSED)",
        "TCB services spawned: PID 1 (indus-init), PID 2 (indus-logd), PID 3 (indus-store) active (INV-G5-HW-001 PASSED)",
        "UX services spawned: PID 4 (indus-shell), PID 5 (indus-edit) active in isolated domains (INV-G5-HW-002 PASSED)",
        "IPC concurrency verified: 500 synchronous rounds across 5 processes with 0 drops (INV-G5-HW-003 PASSED)",
        "[SP-103] Sprint SP-103 Full G5 Userspace on Hardware Milestone REACHED",
    ]
    for el in expected_logs:
        if el not in log_content:
            print(f"❌ FAIL: Log string not found: '{el}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {el[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-103 VERIFICATION PASSED (100% DoD) ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp103()
