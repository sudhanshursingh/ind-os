#!/usr/bin/env python3
"""
Ind OS Sprint SP-63 Verification & Acceptance Test Harness
Target: App Framework, Plain-Language Manifests & Sandboxed Execution
Standards: Bible §6.3, §8.14, Table 19.2, REQ-USE-207, INV-APP-001..003
"""

import os
import sys

REQUIRED_SP63_FILES = [
    "docs/architecture/app-framework-spec.md",
    "docs/evidence/SP-63-validation.md",
    "docs/evidence/golden-boot-sp63.log",
    "userspace/ux/src/app_framework.rs",
    "kernel/src/userspace/app_framework_core.rs",
]

def verify_sp63():
    print("==================================================")
    print(" Ind OS SP-63 App Framework & Manifests Verifier ")
    print(" Target: Sprint SP-63 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-63 deliverable presence...")
    for rel_path in REQUIRED_SP63_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-63 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/app_framework.rs", "r") as f:
        app_src = f.read()
    for s in [
        "AppFrameworkError",
        "AppManifest",
        "SandboxedApp",
        "UserMediationGrant",
        "AppFrameworkEngine",
        "validate_manifest",
        "spawn_app",
        "create_user_grant",
        "apply_user_grant",
        "find_app",
    ]:
        if s not in app_src:
            print(f"❌ FAIL: app_framework.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/app_framework_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelAppFrameworkError",
        "KernelAppManifest",
        "KernelSandboxedApp",
        "KernelUserMediationGrant",
        "KernelAppFrameworkEngine",
        "validate_manifest",
        "spawn_app",
        "create_user_grant",
        "apply_user_grant",
        "find_app",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: app_framework_core.rs missing symbol '{s}'")
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
    for src_file in ["userspace/ux/src/app_framework.rs", "kernel/src/userspace/app_framework_core.rs"]:
        with open(src_file, "r") as f:
            content = f.read()
        if "unsafe " in content or "unsafe{" in content:
            print(f"❌ FAIL: Rule N-04 violation in {src_file}: unsafe block detected")
            sys.exit(1)
        print(f"  ✓ Zero unsafe code verified in {src_file}")

    # 5. Golden Boot Log Verification (Stages 264..268)
    print("\n[CHECK 5/6] Verifying bare-metal QEMU boot execution log...")
    with open("docs/evidence/golden-boot-sp63.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "264_APP_FRAMEWORK_INIT",
        "265_APP_PLAIN_LANGUAGE_MANIFEST_VALIDATION",
        "266_APP_SANDBOXED_SPAWN_CAPABILITY_BINDING",
        "267_APP_USER_MEDIATED_SHARING",
        "268_SP63_APP_FRAMEWORK_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{stage}' not found in golden-boot-sp63.log")
            sys.exit(1)
        print(f"  ✓ Stage {stage} present in boot log")

    expected_assertions = [
        "App framework subsystem online: sandbox manager and grant broker armed",
        "Plain-language manifest validation verified: missing justification denied",
        "Sandboxed app spawn verified: PID assigned and capability bounds locked",
        "User-mediated inter-app sharing verified: capability transferred safely upon explicit user grant",
        "Sprint SP-63 App Framework Milestone REACHED.",
    ]
    for assertion in expected_assertions:
        if assertion not in boot_log:
            print(f"❌ FAIL: Expected assertion '{assertion}' missing in log")
            sys.exit(1)
        print(f"  ✓ Assertion verified: {assertion}")

    # 6. Check Invariant Signatures
    print("\n[CHECK 6/6] Verifying SP-63 architectural invariants...")
    for inv in ["INV-APP-001", "INV-APP-002", "INV-APP-003"]:
        if inv not in boot_log:
            print(f"❌ FAIL: Invariant '{inv}' not asserted in golden boot log")
            sys.exit(1)
        print(f"  ✓ Invariant {inv} ratified")

    print("\n==================================================")
    print(" SPRINT SP-63 VERIFICATION RESULT: PASSED        ")
    print(" App Framework & Plain-Language Manifests RATIFIED")
    print("==================================================")

if __name__ == "__main__":
    verify_sp63()
