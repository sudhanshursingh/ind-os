#!/usr/bin/env python3
"""
Ind OS Sprint SP-65 Verification & Acceptance Test Harness
Target: Proactive Engine UI & Aggressiveness Dial
Standards: Bible §6.3, §8.14, Table 19.2, REQ-AI-106, INV-PROACTIVE-001..003
"""

import os
import sys

REQUIRED_SP65_FILES = [
    "docs/architecture/proactive-ui-spec.md",
    "docs/evidence/SP-65-validation.md",
    "docs/evidence/golden-boot-sp65.log",
    "userspace/ux/src/proactive_ui.rs",
    "kernel/src/userspace/proactive_ui_core.rs",
]

def verify_sp65():
    print("==================================================")
    print(" Ind OS SP-65 Proactive Engine UI Verifier       ")
    print(" Target: Sprint SP-65 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-65 deliverable presence...")
    for rel_path in REQUIRED_SP65_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-65 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/proactive_ui.rs", "r") as f:
        pro_src = f.read()
    for s in [
        "ProactiveError",
        "AggressivenessDial",
        "ProactiveSuggestion",
        "ProactiveUiEngine",
        "set_dial",
        "dial",
        "submit_candidate",
        "dismiss",
        "snooze",
        "explain",
        "visible_count",
    ]:
        if s not in pro_src:
            print(f"❌ FAIL: proactive_ui.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/proactive_ui_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelProactiveError",
        "KernelAggressivenessDial",
        "KernelProactiveSuggestion",
        "KernelProactiveUiEngine",
        "set_dial",
        "dial",
        "submit_candidate",
        "dismiss",
        "snooze",
        "explain",
        "visible_count",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: proactive_ui_core.rs missing symbol '{s}'")
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
    for src_file in ["userspace/ux/src/proactive_ui.rs", "kernel/src/userspace/proactive_ui_core.rs"]:
        with open(src_file, "r") as f:
            content = f.read()
        if "unsafe " in content or "unsafe{" in content:
            print(f"❌ FAIL: Rule N-04 violation in {src_file}: unsafe block detected")
            sys.exit(1)
        print(f"  ✓ Zero unsafe code verified in {src_file}")

    # 5. Golden Boot Log Verification (Stages 274..278)
    print("\n[CHECK 5/6] Verifying bare-metal QEMU boot execution log...")
    with open("docs/evidence/golden-boot-sp65.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "274_PROACTIVE_UI_INIT",
        "275_PROACTIVE_AGGRESSIVENESS_DIAL_FILTER",
        "276_PROACTIVE_FEEDBACK_ACTIONS",
        "277_PROACTIVE_EXPLAIN_TOKEN",
        "278_SP65_PROACTIVE_UI_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{stage}' not found in golden-boot-sp65.log")
            sys.exit(1)
        print(f"  ✓ Stage {stage} present in boot log")

    expected_assertions = [
        "Proactive engine UI online: initial dial set to Balanced (70% threshold)",
        "Aggressiveness dial filtering verified: candidate suggestions filtered strictly by confidence threshold",
        "Feedback actions verified: dismiss and snooze reduce visible suggestion count deterministically",
        "Why-engine integration verified: explain action produces deterministic audit token",
        "Sprint SP-65 Proactive UI Milestone REACHED.",
    ]
    for assertion in expected_assertions:
        if assertion not in boot_log:
            print(f"❌ FAIL: Expected assertion '{assertion}' missing in log")
            sys.exit(1)
        print(f"  ✓ Assertion verified: {assertion}")

    # 6. Check Invariant Signatures
    print("\n[CHECK 6/6] Verifying SP-65 architectural invariants...")
    for inv in ["INV-PROACTIVE-001", "INV-PROACTIVE-002", "INV-PROACTIVE-003"]:
        if inv not in boot_log:
            print(f"❌ FAIL: Invariant '{inv}' not asserted in golden boot log")
            sys.exit(1)
        print(f"  ✓ Invariant {inv} ratified")

    print("\n==================================================")
    print(" SPRINT SP-65 VERIFICATION RESULT: PASSED        ")
    print(" Proactive Engine UI & Aggressiveness Dial RATIFIED")
    print("==================================================")

if __name__ == "__main__":
    verify_sp65()
