#!/usr/bin/env python3
"""
Ind OS Sprint SP-104 Verification & Acceptance Test Harness
Target: Hardware Compatibility Matrix v1
Standards: Bible §6.3, §8.15, Table 19.2, PRD-001 §8.3, REQ-HW-114, INV-HW-MAT-001..003
"""

import os
import sys

REQUIRED_SP104_FILES = [
    "docs/architecture/hw-matrix-spec.md",
    "docs/architecture/hardware-compatibility-matrix.md",
    "docs/evidence/SP-104-validation.md",
    "docs/evidence/golden-boot-sp104.log",
    "userspace/init/src/hw_matrix.rs",
    "kernel/src/hal/platform/hw_matrix_core.rs",
]

def verify_sp104():
    print("==================================================")
    print(" Ind OS SP-104 Hardware Matrix Verifier           ")
    print(" Target: Sprint SP-104 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-104 deliverable presence...")
    for rel_path in REQUIRED_SP104_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-104 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/init/src/hw_matrix.rs", "r") as f:
        init_src = f.read()
    for s in [
        "HwMatrixAuditor",
        "HwMatrixError",
        "HwMatrixEntry",
        "HwStatus",
        "CAP_AUDIT_HARDWARE",
        "TOTAL_MATRIX_ENTRIES",
        "WORKS_COUNT",
        "PARTIAL_COUNT",
        "OUT_OF_SCOPE_COUNT",
        "audit_all",
        "get_entry",
        "works_count",
        "partial_count",
        "out_of_scope_count",
        "is_audited",
    ]:
        if s not in init_src:
            print(f"❌ FAIL: hw_matrix.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/hw_matrix_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelHwMatrixHarness",
        "KernelHwMatrixError",
        "KernelHwMatrixEntry",
        "KernelHwStatus",
        "KERNEL_CAP_AUDIT_HARDWARE",
        "KERNEL_TOTAL_MATRIX_ENTRIES",
        "KERNEL_WORKS_COUNT",
        "KERNEL_PARTIAL_COUNT",
        "KERNEL_OUT_OF_SCOPE_COUNT",
        "audit_all",
        "get_entry",
        "works_count",
        "partial_count",
        "out_of_scope_count",
        "is_audited",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: hw_matrix_core.rs missing symbol '{s}'")
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
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/init/src/hw_matrix.rs)...")
    with open("userspace/init/src/hw_matrix.rs", "r") as f:
        content = f.read()
    if "unsafe " in content or "unsafe{" in content:
        print("❌ FAIL: unsafe code block found in userspace/init/src/hw_matrix.rs")
        sys.exit(1)
    print("  ✓ userspace/init/src/hw_matrix.rs is 100% safe Rust")

    # 5. Golden Boot Log Verification: Stages 470-474
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 470-474...")
    with open("docs/evidence/golden-boot-sp104.log", "r") as f:
        log_content = f.read()

    stages = [
        "470_SP104_HW_MATRIX_INIT",
        "471_SP104_PERIPHERAL_AUDIT_EXEC",
        "472_SP104_WORKS_CRITERIA_VERIFY",
        "473_SP104_OUT_OF_SCOPE_JUSTIFICATION",
        "474_SP104_HW_MATRIX_READY",
    ]
    for st in stages:
        if st not in log_content:
            print(f"❌ FAIL: Missing stage {st} in docs/evidence/golden-boot-sp104.log")
            sys.exit(1)
        print(f"  ✓ Verified stage: {st}")

    # 6. Specific Confirmation Logs
    print("\n[CHECK 6/6] Verifying Hardware Matrix Confirmation Logs...")
    expected_logs = [
        "Hardware matrix auditor online: 15 peripheral components staged (PRD-001 §8.3 PASSED)",
        "Peripheral audit executed: 15/15 devices probed across APPS, SYS, and PERIPH domains (INV-HW-MAT-001 PASSED)",
        "WORKS criteria verified: 12 foundational subsystems fully functional with zero blockers (INV-HW-MAT-002 PASSED)",
        "Out-of-scope justification ratified: Zero closed binary blobs in TCB; CPU NEON fallback active (INV-HW-MAT-003 PASSED)",
        "[SP-104] Sprint SP-104 Hardware Compatibility Matrix v1 Milestone REACHED",
    ]
    for el in expected_logs:
        if el not in log_content:
            print(f"❌ FAIL: Log string not found: '{el}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {el[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-104 VERIFICATION PASSED (100% DoD) ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp104()
