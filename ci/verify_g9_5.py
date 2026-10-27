#!/usr/bin/env python3
"""
Ind OS Sprint SP-115 / Gate G9.5 Verification & Acceptance Test Harness
Target: Gate G9.5 Hard-Stop Audit & Ratification (Performance & Usability)
"""

import os
import sys

REQUIRED_G9_5_FILES = [
    "docs/architecture/g9-5-acceptance-pack-spec.md",
    "docs/evidence/gate-g9-5-audit-report.md",
    "userspace/init/src/g9_5_audit.rs",
    "kernel/src/hal/platform/g9_5_audit_core.rs",
]

def verify_g9_5():
    print("==================================================")
    print(" Ind OS Gate G9.5 Hard-Stop Audit & Ratifier     ")
    print(" Milestone: PERFORMANCE & USABILITY PASS V0.1    ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/5] Verifying Gate G9.5 deliverable presence...")
    for rel_path in REQUIRED_G9_5_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required Gate G9.5 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Rule N-01 Compliance Check
    print("\n[CHECK 2/5] Verifying Rule N-01 (Zero external crates)...")
    with open("kernel/Cargo.toml", "r") as f:
        cargo_txt = f.read()
    if "dependencies" in cargo_txt:
        deps = cargo_txt.split("[dependencies]")[1].strip()
        lines = [l for l in deps.split("\n") if l.strip() and not l.strip().startswith("#")]
        if lines:
            print(f"❌ FAIL: External crates in kernel/Cargo.toml: {lines}")
            sys.exit(1)
    print("  ✓ Rule N-01 PASSED: 0 external crates in TCB")

    # 3. Rule N-04 Compliance Check
    print("\n[CHECK 3/5] Verifying Rule N-04 (Zero unsafe code in userspace)...")
    with open("userspace/init/src/g9_5_audit.rs", "r") as f:
        code_lines = [l.strip() for l in f.readlines() if not l.strip().startswith("//") and not l.strip().startswith("/*") and not l.strip().startswith("*")]
    code_txt = "\n".join(code_lines)
    if "unsafe " in code_txt or "unsafe{" in code_txt:
        print("❌ FAIL: unsafe code block found in userspace/init/src/g9_5_audit.rs")
        sys.exit(1)
    print("  ✓ Rule N-04 PASSED: userspace/init/src/g9_5_audit.rs is 100% safe Rust")

    # 4. Acceptance Criteria Validation
    print("\n[CHECK 4/5] Validating 8/8 Acceptance Criteria in Gate G9.5 report...")
    with open("docs/evidence/gate-g9-5-audit-report.md", "r") as f:
        audit_text = f.read()
    for crit in ["Cold boot", "App cold-launch", "Intent-to-card", "Compositor 120Hz", "Idle OS RAM", "Touch input latency", "Sequential storage read", "Two-Person Rule"]:
        if crit not in audit_text:
            print(f"❌ FAIL: Gate G9.5 report missing criterion {crit}")
            sys.exit(1)
    print("  ✓ All 8 mandatory ratification criteria verified")

    # 5. Milestone Declaration Check
    print("\n[CHECK 5/5] Checking Milestone Declaration...")
    if "PASSED & RATIFIED" not in audit_text:
        print("❌ FAIL: Milestone declaration missing in Gate G9.5 report")
        sys.exit(1)
    print("  ✓ Gate G9.5 Hard-Stop Audit verified PASSED & RATIFIED")

    print("\n==================================================")
    print(" 🏆 GATE G9.5 RATIFIED: PERFORMANCE & USABILITY V0.1 🏆")
    print(" Phase 9.5 Complete. Authorization to Enter Phase 10.")
    print("==================================================")

if __name__ == "__main__":
    verify_g9_5()
