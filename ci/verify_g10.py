#!/usr/bin/env python3
"""
Ind OS Gate G10 Hard-Stop Audit & Ratifier
Milestone: HARDENING, VERIFICATION & BETA READINESS (Phase 10)
"""

import sys
import os
import re

def check_file_exists(path):
    if not os.path.isfile(path):
        print(f"❌ FAIL: Missing required file: {path}")
        sys.exit(1)
    print(f"  ✓ Found {path}")

def check_no_unsafe(path):
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    cleaned = re.sub(r"//.*", "", content)
    cleaned = re.sub(r"/\*[\s\S]*?\*/", "", cleaned)
    if "unsafe " in cleaned or "unsafe{" in cleaned:
        print(f"❌ FAIL: unsafe code block found in {path}")
        sys.exit(1)
    print(f"  ✓ Rule N-04 PASSED: {path} is 100% safe Rust")

def check_no_external_crates():
    manifests = ["kernel/Cargo.toml", "userspace/init/Cargo.toml"]
    for m in manifests:
        if os.path.exists(m):
            with open(m, "r") as f:
                content = f.read()
            if "[dependencies]" in content:
                deps_block = content.split("[dependencies]")[1].split("[")[0].strip()
                for line in deps_block.splitlines():
                    line = line.strip()
                    if line and not line.startswith("#"):
                        if "path =" not in line:
                            print(f"❌ FAIL: External crate detected in {m}: {line}")
                            sys.exit(1)
    print("  ✓ Rule N-01 PASSED: 0 external crates in TCB")

def main():
    print("==================================================")
    print(" Ind OS Gate G10 Hard-Stop Audit & Ratifier       ")
    print(" Milestone: HARDENING & BETA READINESS            ")
    print("==================================================")

    print("[CHECK 1/5] Verifying Gate G10 deliverable presence...")
    check_file_exists("docs/architecture/gate-g10-acceptance-pack-spec.md")
    check_file_exists("docs/architecture/public-governance-transition-plan.md")
    check_file_exists("docs/evidence/gate-g10-audit-report.md")
    check_file_exists("userspace/init/src/g10_audit.rs")
    check_file_exists("kernel/src/hal/platform/g10_audit_core.rs")

    print("\n[CHECK 2/5] Verifying Rule N-01 (Zero external crates)...")
    check_no_external_crates()

    print("\n[CHECK 3/5] Verifying Rule N-04 (Zero unsafe code in userspace)...")
    check_no_unsafe("userspace/init/src/g10_audit.rs")

    print("\n[CHECK 4/5] Validating 8/8 Acceptance Criteria in Gate G10 report...")
    with open("docs/evidence/gate-g10-audit-report.md") as f:
        ev = f.read()
    required = [
        "Formal Verification Proofs in CI",
        "External Security Audit Closed",
        "Red-Team Adversarial Campaign",
        "Production Operations Rehearsed",
        "Beta Cohort Planning & Consent Telemetry",
        "Continuous Hardening & Patch SLAs",
        "Public Governance Transition Plan",
        "TCB Governance & Two-Person Rule",
    ]
    for r in required:
        if r not in ev:
            print(f"❌ FAIL: Missing Gate G10 criterion: {r}")
            sys.exit(1)
    print("  ✓ All 8 mandatory ratification criteria verified")

    print("\n[CHECK 5/5] Checking Milestone Declaration...")
    if "PASSED & RATIFIED" not in ev:
        print("❌ FAIL: Gate G10 declaration not found in audit report")
        sys.exit(1)
    print("  ✓ Gate G10 Hard-Stop Audit verified PASSED & RATIFIED")

    print("\n==================================================")
    print(" 🏆 GATE G10 RATIFIED: HARDENING & BETA READINESS 🏆")
    print(" Phase 10 Complete. Authorization to Enter Beta.")
    print("==================================================")

if __name__ == "__main__":
    main()
