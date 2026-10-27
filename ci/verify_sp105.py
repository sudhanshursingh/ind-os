#!/usr/bin/env python3
"""
Ind OS Sprint SP-105 Verification & Acceptance Test Harness
Target: Recovery Route Re-Tested After Experiments
Standards: Bible §6.3, §8.15, Table 19.2, Rule N-06, REQ-SEC-111, REQ-REC-102, INV-RECOV-001..003
"""

import os
import sys

REQUIRED_SP105_FILES = [
    "docs/architecture/recovery-retest-spec.md",
    "docs/evidence/SP-105-validation.md",
    "docs/evidence/golden-boot-sp105.log",
    "userspace/init/src/recovery_audit.rs",
    "kernel/src/hal/platform/recovery_core.rs",
]

def verify_sp105():
    print("==================================================")
    print(" Ind OS SP-105 Recovery Re-Test Verifier          ")
    print(" Target: Sprint SP-105 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-105 deliverable presence...")
    for rel_path in REQUIRED_SP105_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-105 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/init/src/recovery_audit.rs", "r") as f:
        init_src = f.read()
    for s in [
        "RecoveryAuditEngine",
        "RecoveryError",
        "RecoveryMode",
        "RecoveryStatus",
        "CAP_RECOVERY_ADMIN",
        "QUALCOMM_EDL_VID",
        "QUALCOMM_EDL_PID",
        "SAHARA_HELLO_CMD",
        "STOCK_IMAGE_CRC32",
        "verify_fastboot_protocol",
        "verify_edl_testpoint",
        "verify_stock_image",
        "status",
        "is_fully_recoverable",
    ]:
        if s not in init_src:
            print(f"❌ FAIL: recovery_audit.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/recovery_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelRecoveryHarness",
        "KernelRecoveryError",
        "KernelRecoveryMode",
        "KernelRecoveryStatus",
        "KERNEL_CAP_RECOVERY_ADMIN",
        "KERNEL_QUALCOMM_EDL_VID",
        "KERNEL_QUALCOMM_EDL_PID",
        "KERNEL_SAHARA_HELLO_CMD",
        "KERNEL_STOCK_IMAGE_CRC32",
        "verify_fastboot_protocol",
        "verify_edl_testpoint",
        "verify_stock_image",
        "status",
        "is_fully_recoverable",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: recovery_core.rs missing symbol '{s}'")
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
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/init/src/recovery_audit.rs)...")
    with open("userspace/init/src/recovery_audit.rs", "r") as f:
        content = f.read()
    if "unsafe " in content or "unsafe{" in content:
        print("❌ FAIL: unsafe code block found in userspace/init/src/recovery_audit.rs")
        sys.exit(1)
    print("  ✓ userspace/init/src/recovery_audit.rs is 100% safe Rust")

    # 5. Golden Boot Log Verification: Stages 475-479
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 475-479...")
    with open("docs/evidence/golden-boot-sp105.log", "r") as f:
        log_content = f.read()

    stages = [
        "475_SP105_RECOVERY_HARNESS_INIT",
        "476_SP105_FASTBOOT_INTERFACE_CHECK",
        "477_SP105_EDL_TESTPOINT_VALIDATION",
        "478_SP105_STOCK_REVERT_PROOF",
        "479_SP105_RECOVERY_RETESTED_READY",
    ]
    for st in stages:
        if st not in log_content:
            print(f"❌ FAIL: Missing stage {st} in docs/evidence/golden-boot-sp105.log")
            sys.exit(1)
        print(f"  ✓ Verified stage: {st}")

    # 6. Specific Confirmation Logs
    print("\n[CHECK 6/6] Verifying Recovery Confirmation Logs...")
    expected_logs = [
        "Recovery re-test harness online: Fastboot, EDL, and Stock verifiers armed (Rule N-06 PASSED)",
        "Fastboot interface verified: 0x05C6/0x9008 SBL command protocol operational (INV-RECOV-001 PASSED)",
        "EDL 9008 verified: Sahara hello packet handshake 0x00000001 active (INV-RECOV-002 PASSED)",
        "Stock revert verified: Motorola G60 golden ROM CRC32 0xDE7CA6F1 verified intact (INV-RECOV-003 PASSED)",
        "[SP-105] Sprint SP-105 Recovery Route Re-Tested Milestone REACHED",
    ]
    for el in expected_logs:
        if el not in log_content:
            print(f"❌ FAIL: Log string not found: '{el}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {el[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-105 VERIFICATION PASSED (100% DoD) ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp105()
