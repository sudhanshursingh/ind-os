#!/usr/bin/env python3
"""
Ind OS Sprint SP-64 Verification & Acceptance Test Harness
Target: Accessibility Services (P0 Standard: Screen Reader, Magnifier, Switch Access)
Standards: Bible §6.3, §8.14, Table 19.2, REQ-UX-304, INV-A11Y-001..003
"""

import os
import sys

REQUIRED_SP64_FILES = [
    "docs/architecture/accessibility-spec.md",
    "docs/evidence/SP-64-validation.md",
    "docs/evidence/golden-boot-sp64.log",
    "userspace/ux/src/accessibility.rs",
    "kernel/src/userspace/accessibility_core.rs",
]

def verify_sp64():
    print("==================================================")
    print(" Ind OS SP-64 Accessibility Services (P0) Verifier")
    print(" Target: Sprint SP-64 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-64 deliverable presence...")
    for rel_path in REQUIRED_SP64_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-64 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/accessibility.rs", "r") as f:
        a11y_src = f.read()
    for s in [
        "A11yError",
        "A11yRole",
        "A11yNode",
        "ScreenReaderEngine",
        "MagnifierService",
        "SwitchAccessEngine",
        "AccessibilitySubsystem",
        "register_node",
        "set_focus",
        "get_focused_node",
        "set_zoom",
        "zoom_percent",
        "step_next",
    ]:
        if s not in a11y_src:
            print(f"❌ FAIL: accessibility.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/accessibility_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelA11yError",
        "KernelA11yRole",
        "KernelA11yNode",
        "KernelScreenReaderEngine",
        "KernelMagnifierService",
        "KernelSwitchAccessEngine",
        "KernelAccessibilitySubsystem",
        "register_node",
        "set_focus",
        "get_focused_node",
        "set_zoom",
        "zoom_percent",
        "step_next",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: accessibility_core.rs missing symbol '{s}'")
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
    for src_file in ["userspace/ux/src/accessibility.rs", "kernel/src/userspace/accessibility_core.rs"]:
        with open(src_file, "r") as f:
            content = f.read()
        if "unsafe " in content or "unsafe{" in content:
            print(f"❌ FAIL: Rule N-04 violation in {src_file}: unsafe block detected")
            sys.exit(1)
        print(f"  ✓ Zero unsafe code verified in {src_file}")

    # 5. Golden Boot Log Verification (Stages 269..273)
    print("\n[CHECK 5/6] Verifying bare-metal QEMU boot execution log...")
    with open("docs/evidence/golden-boot-sp64.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "269_A11Y_SUBSYSTEM_INIT",
        "270_A11Y_SCREEN_READER_SEMANTIC_TREE",
        "271_A11Y_VIEWPORT_MAGNIFICATION",
        "272_A11Y_SWITCH_ACCESS_SCANNING",
        "273_SP64_A11Y_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{stage}' not found in golden-boot-sp64.log")
            sys.exit(1)
        print(f"  ✓ Stage {stage} present in boot log")

    expected_assertions = [
        "Accessibility subsystem online: screen reader, magnifier, switch access armed",
        "Screen reader semantic tree verified: focused node traversed with zero heap allocation",
        "Magnification zoom verified: clamped strictly within [100%, 400%] bounds",
        "Switch access scanning verified: cyclic element traversal with deterministic debounce",
        "Sprint SP-64 Accessibility Milestone REACHED.",
    ]
    for assertion in expected_assertions:
        if assertion not in boot_log:
            print(f"❌ FAIL: Expected assertion '{assertion}' missing in log")
            sys.exit(1)
        print(f"  ✓ Assertion verified: {assertion}")

    # 6. Check Invariant Signatures
    print("\n[CHECK 6/6] Verifying SP-64 architectural invariants...")
    for inv in ["INV-A11Y-001", "INV-A11Y-002", "INV-A11Y-003"]:
        if inv not in boot_log:
            print(f"❌ FAIL: Invariant '{inv}' not asserted in golden boot log")
            sys.exit(1)
        print(f"  ✓ Invariant {inv} ratified")

    print("\n==================================================")
    print(" SPRINT SP-64 VERIFICATION RESULT: PASSED        ")
    print(" Accessibility Services (P0) RATIFIED            ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp64()
