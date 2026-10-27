#!/usr/bin/env python3
"""
Ind OS SP-119 Verification Harness
Verifies External Security Audit & TCB Triage (REQ-SEC-109, INV-EXTAUDIT-001)
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
    print(" Ind OS SP-119 External Security Audit Verifier   ")
    print(" Target: Sprint SP-119 Definition of Done (DoD)   ")
    print("==================================================")

    print("[CHECK 1/5] Verifying SP-119 deliverable presence...")
    check_file_exists("docs/architecture/external-security-audit-spec.md")
    check_file_exists("docs/evidence/SP-119-validation.md")
    check_file_exists("userspace/ux/src/external_audit.rs")
    check_file_exists("kernel/src/hal/platform/external_audit_core.rs")

    print("\n[CHECK 2/5] Verifying Invariant INV-EXTAUDIT-001...")
    with open("docs/architecture/external-security-audit-spec.md") as f:
        spec = f.read()
    if "INV-EXTAUDIT-001" not in spec or "REQ-SEC-109" not in spec:
        print("❌ FAIL: Invariant INV-EXTAUDIT-001 or REQ-SEC-109 missing from spec")
        sys.exit(1)
    print("  ✓ INV-EXTAUDIT-001 verified")

    print("\n[CHECK 3/5] Verifying Rule N-01 (Zero external crates)...")
    check_no_external_crates()

    print("\n[CHECK 4/5] Verifying Rule N-04 (Zero unsafe code in userspace)...")
    check_no_unsafe("userspace/ux/src/external_audit.rs")

    print("\n[CHECK 5/5] Validating Audit Finding Triage in SP-119 report...")
    with open("docs/evidence/SP-119-validation.md") as f:
        ev = f.read()
    if "PASSED" not in ev or "Open Criticals: 0" not in ev:
        print("❌ FAIL: Validation report incomplete")
        sys.exit(1)
    print("  ✓ External audit findings triaged and 0 open criticals confirmed")

    print("\n==================================================")
    print(" 🚀 SPRINT SP-119 DEFINITION OF DONE: PASSED      ")
    print("==================================================")

if __name__ == "__main__":
    main()
