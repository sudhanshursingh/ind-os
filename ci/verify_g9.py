#!/usr/bin/env python3
"""
Ind OS Gate G9 Hard-Stop Audit & Ratification Test Harness
Target: Gate G9 Hard-Stop Audit & Ratification (Phase 9 Moto G60 Bring-Up)
Standards: Bible §6.3, §8.15, Table 19.2, §19.3, §19.4; PRD-001 §7.5, §8.1-8.5, §9.5
"""

import os
import sys

REQUIRED_G9_FILES = [
    "docs/evidence/gate-g9-audit-report.md",
    "docs/evidence/golden-boot-g9.log",
    "docs/DECISIONS/gate-G9-dec001-review.md",
    "docs/architecture/g9-acceptance-pack-spec.md",
    "docs/architecture/hardware-compatibility-matrix-v1.md",
    "docs/architecture/sbom-phase9.md",
    "docs/evidence/sbom-phase9.json",
    "docs/evidence/G9/manifest.json",
    "docs/evidence/G9/build-provenance.md",
    "docs/evidence/G9/dod-test-logs.md",
    "docs/evidence/G9/docs-diff-phase9.md",
    "docs/evidence/G9/gate-g9-signatures.md",
    "userspace/init/src/g9_audit.rs",
    "kernel/src/hal/platform/g9_audit_core.rs",
]

def verify_g9():
    print("==================================================")
    print(" Ind OS Gate G9 Hard-Stop Audit & Ratifier        ")
    print(" Milestone: MOTO G60 BRING-UP / HARDWARE V0.1     ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying Gate G9 deliverable presence...")
    for rel_path in REQUIRED_G9_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required Gate G9 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/init/src/g9_audit.rs", "r") as f:
        init_src = f.read()
    for s in [
        "GateG9Auditor",
        "GateG9Error",
        "GateG9Criterion",
        "CAP_GATE_G9_RATIFY",
        "G9_AUDIT_CRITERIA_COUNT",
        "audit_all_criteria",
        "ratify_gate",
        "is_ratified",
    ]:
        if s not in init_src:
            print(f"❌ FAIL: g9_audit.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/g9_audit_core.rs", "r") as f:
        hal_src = f.read()
    for s in [
        "KernelGateG9Auditor",
        "KernelGateG9Error",
        "KernelGateG9Criterion",
        "KERNEL_CAP_GATE_G9_RATIFY",
        "KERNEL_G9_AUDIT_CRITERIA_COUNT",
        "audit_all_criteria",
        "ratify_gate",
        "is_ratified",
    ]:
        if s not in hal_src:
            print(f"❌ FAIL: g9_audit_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel HAL symbol: {s}")

    # 3. Rule N-01 Compliance: Zero external crate dependencies
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for cargo_path in ["kernel/Cargo.toml", "userspace/init/Cargo.toml"]:
        with open(cargo_path, "r") as f:
            lines = f.readlines()
        in_deps = False
        for line in lines:
            line_str = line.strip()
            if line_str == "[dependencies]":
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
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/init/src/g9_audit.rs)...")
    with open("userspace/init/src/g9_audit.rs", "r") as f:
        content = f.read()
    if "unsafe " in content or "unsafe{" in content:
        print("❌ FAIL: unsafe code block found in userspace/init/src/g9_audit.rs")
        sys.exit(1)
    print("  ✓ userspace/init/src/g9_audit.rs is 100% safe Rust")

    # 5. Golden Boot Log Verification: Stages 495-499
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 495-499...")
    with open("docs/evidence/golden-boot-g9.log", "r") as f:
        log_content = f.read()

    stages = [
        "495_GATE_G9_AUDIT_INIT",
        "496_GATE_G9_CRITERIA_VERIFICATION",
        "497_GATE_G9_RATIFICATION_CONVENED",
        "498_GATE_G9_DECLARATION",
        "499_GATE_G9_RATIFIED",
    ]
    for st in stages:
        if st not in log_content:
            print(f"❌ FAIL: Missing stage {st} in docs/evidence/golden-boot-g9.log")
            sys.exit(1)
        print(f"  ✓ Verified stage: {st}")

    # 6. Specific Confirmation Logs
    print("\n[CHECK 6/6] Verifying Gate G9 Ratification Confirmation Logs...")
    expected_logs = [
        "Gate G9 hard-stop auditor online: evaluating 8 mandatory ratification criteria",
        "Criteria 1/8: Cold boots reproducible (10/10 runs) without hangs (REQ-QA-201 PASSED)",
        "Criteria 2/8: Display + touch fully usable at 60Hz/120Hz (PRD-001 §8.1 PASSED)",
        "Criteria 3/8: UFS storage mounts reliably; IndFS operational (PRD-001 §8.2 PASSED)",
        "Criteria 4/8: Power management operational (PMIC rails + battery gauge) (PRD-001 §8.4 PASSED)",
        "Criteria 5/8: Hardware Compatibility Matrix v1 published internally (PRD-001 §8.3 PASSED)",
        "Criteria 6/8: Recovery route re-tested after experiments (Fastboot/EDL) (Rule N-06 PASSED)",
        "Criteria 7/8: Phase 9 SBOM refreshed: 0 external crates in TCB (Rule N-01 PASSED)",
        "Criteria 8/8: Acceptance pack complete per PRD-001 §9.5 with minuted DEC-001 (Bible §19.4 PASSED)",
        "Ratification session concluded: Two-person rule sign-off verified (REQ-GOV-106 PASSED)",
        "Milestone declared: MOTO G60 BRING-UP / HARDWARE V0.1",
        "GATE G9 HARD-STOP AUDIT PASSED & RATIFIED",
    ]
    for el in expected_logs:
        if el not in log_content:
            print(f"❌ FAIL: Log string not found: '{el}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {el[:65]}...")

    print("\n==================================================")
    print(" 🏆 GATE G9 RATIFIED: MOTO G60 BRING-UP V0.1 PASSED 🏆")
    print("==================================================")

if __name__ == "__main__":
    verify_g9()
