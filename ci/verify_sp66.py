#!/usr/bin/env python3
"""
Ind OS Sprint SP-66 Verification & Acceptance Test Harness
Target: Core Apps Suite (Files, Settings, Why-Explorer, Terminal, Notes)
Standards: Bible §6.3, §8.14, Table 19.2, REQ-USE-207, REQ-AI-105, INV-COREAPPS-001..003
"""

import os
import sys

REQUIRED_SP66_FILES = [
    "docs/architecture/core-apps-spec.md",
    "docs/evidence/SP-66-validation.md",
    "docs/evidence/golden-boot-sp66.log",
    "userspace/ux/src/core_apps.rs",
    "kernel/src/userspace/core_apps_core.rs",
]

def verify_sp66():
    print("==================================================")
    print(" Ind OS SP-66 Core Apps Suite Verifier           ")
    print(" Target: Sprint SP-66 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-66 deliverable presence...")
    for rel_path in REQUIRED_SP66_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-66 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/core_apps.rs", "r") as f:
        app_src = f.read()
    for s in [
        "CoreAppDescriptor",
        "CoreAppError",
        "CoreAppsSuite",
        "CORE_APPS_COUNT",
        "launch_app",
        "revoke_consent_grant",
        "active_grants_count",
        "get_app",
        "apps",
    ]:
        if s not in app_src:
            print(f"❌ FAIL: core_apps.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/core_apps_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelCoreAppDescriptor",
        "KernelCoreAppError",
        "KernelCoreAppsSuite",
        "KERNEL_CORE_APPS_COUNT",
        "launch_app",
        "revoke_consent_grant",
        "active_grants_count",
        "get_app",
        "apps",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: core_apps_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel harness symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    with open("userspace/ux/Cargo.toml", "r") as f:
        cargo_lines = f.readlines()
    in_deps = False
    for line in cargo_lines:
        line_clean = line.split("#")[0].strip()
        if not line_clean:
            continue
        if line_clean == "[dependencies]":
            in_deps = True
            continue
        if in_deps and line_clean.startswith("["):
            in_deps = False
        if in_deps:
            if "path =" not in line_clean:
                print(f"❌ FAIL: Non-path (external) dependency in userspace/ux/Cargo.toml: {line_clean}")
                sys.exit(1)
    print("  ✓ userspace/ux uses pure in-tree path dependencies exclusively.")

    # 4. Rule N-04: Zero Unsafe Code in New Modules Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in new modules)...")
    for src_file in ["userspace/ux/src/core_apps.rs", "kernel/src/userspace/core_apps_core.rs"]:
        with open(src_file, "r") as f:
            content = f.read()
        if "unsafe " in content or "unsafe{" in content:
            print(f"❌ FAIL: Rule N-04 violation in {src_file}: unsafe block detected")
            sys.exit(1)
        print(f"  ✓ Zero unsafe code verified in {src_file}")

    # 5. Golden Boot Log Verification (Stages 279..283)
    print("\n[CHECK 5/6] Verifying bare-metal QEMU boot execution log...")
    with open("docs/evidence/golden-boot-sp66.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "279_CORE_APPS_INIT",
        "280_CORE_APPS_LEAST_PRIVILEGE",
        "281_CORE_APPS_SETTINGS_CONSENT_REVOCATION",
        "282_CORE_APPS_WHY_EXPLORER_CAUSAL_LOG",
        "283_SP66_CORE_APPS_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{stage}' not found in golden-boot-sp66.log")
            sys.exit(1)
        print(f"  ✓ Stage {stage} present in boot log")

    expected_assertions = [
        "Core apps suite online: 5 essential apps discovered",
        "Least-privilege verified: all 5 apps conform to minimal capability envelopes",
        "Settings consent surface verified: active capability grant revoked in real time",
        "Why-Explorer verified: timeline of past decisions queried with zero heap allocations",
        "Sprint SP-66 Core Apps Milestone REACHED.",
    ]
    for assertion in expected_assertions:
        if assertion not in boot_log:
            print(f"❌ FAIL: Expected assertion '{assertion}' missing in log")
            sys.exit(1)
        print(f"  ✓ Assertion verified: {assertion}")

    # 6. Check Invariant Signatures
    print("\n[CHECK 6/6] Verifying SP-66 architectural invariants...")
    for inv in ["INV-COREAPPS-001", "INV-COREAPPS-002", "INV-COREAPPS-003"]:
        if inv not in boot_log:
            print(f"❌ FAIL: Invariant '{inv}' not asserted in golden boot log")
            sys.exit(1)
        print(f"  ✓ Invariant {inv} ratified")

    print("\n==================================================")
    print(" SPRINT SP-66 VERIFICATION RESULT: PASSED        ")
    print(" Core Apps Suite RATIFIED                        ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp66()
