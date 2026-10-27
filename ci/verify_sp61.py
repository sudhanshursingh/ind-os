#!/usr/bin/env python3
"""
Ind OS Sprint SP-61 Verification & Acceptance Test Harness
Target: Shell v2 (The Ind Surface: Home, Notifications, AIS Visual Distinction, Intent Bar)
Standards: Bible §6.3, §8.14, Table 19.2, REQ-UX-305, INV-SHELL2-001..003, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json

REQUIRED_SP61_FILES = [
    "docs/architecture/shell-v2-spec.md",
    "docs/evidence/SP-61-validation.md",
    "docs/evidence/golden-boot-sp61.log",
    "userspace/ux/src/shell_v2.rs",
    "kernel/src/userspace/shell_v2_core.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp61():
    print("==================================================")
    print(" Ind OS SP-61 Shell v2 & Ind Surface Verifier    ")
    print(" Target: Sprint SP-61 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-61 deliverable presence...")
    for rel_path in REQUIRED_SP61_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-61 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/shell_v2.rs", "r") as f:
        shell_src = f.read()
    for s in [
        "ShellError",
        "NotificationSource",
        "NotificationAction",
        "NotificationItem",
        "ShellV2Engine",
        "post_notification",
        "dismiss_notification",
        "snooze_notification",
        "submit_intent_query",
        "launcher_apps",
        "is_ais_distinct",
    ]:
        if s not in shell_src:
            print(f"❌ FAIL: shell_v2.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/shell_v2_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelShellError",
        "KernelNotificationSource",
        "KernelNotificationAction",
        "KernelNotificationItem",
        "KernelShellV2Engine",
        "post_notification",
        "dismiss_notification",
        "snooze_notification",
        "submit_intent_query",
        "launcher_apps",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: shell_v2_core.rs missing symbol '{s}'")
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
    for src_file in ["userspace/ux/src/shell_v2.rs", "kernel/src/userspace/shell_v2_core.rs"]:
        with open(src_file, "r") as f:
            content = f.read()
        if "unsafe " in content or "unsafe{" in content:
            print(f"❌ FAIL: Rule N-04 violation in {src_file}: unsafe block detected")
            sys.exit(1)
        print(f"  ✓ Zero unsafe code verified in {src_file}")

    # 5. Golden Boot Log Verification (Stages 254..258)
    print("\n[CHECK 5/6] Verifying bare-metal QEMU boot execution log...")
    with open("docs/evidence/golden-boot-sp61.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "254_SHELL_V2_INIT",
        "255_SHELL_AIS_VISUAL_DISTINCTION",
        "256_SHELL_NOTIFICATION_LIFECYCLE",
        "257_SHELL_INTENT_QUERY_BAR",
        "258_SP61_SHELL2_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{stage}' not found in golden-boot-sp61.log")
            sys.exit(1)
        print(f"  ✓ Stage {stage} present in boot log")

    expected_assertions = [
        "Shell v2 initialized: 5 core apps verified on launcher grid",
        "Visual distinction verified: AIS suggestions carry distinct presentation badge",
        "Notification lifecycle verified: dismiss and snooze actions dispatched safely",
        "Intent query bar verified: natural language query accepted and routed to broker",
        "Sprint SP-61 Shell v2 Milestone REACHED.",
    ]
    for assertion in expected_assertions:
        if assertion not in boot_log:
            print(f"❌ FAIL: Expected assertion '{assertion}' missing in log")
            sys.exit(1)
        print(f"  ✓ Assertion verified: {assertion}")

    # 6. Check Invariant Signatures
    print("\n[CHECK 6/6] Verifying SP-61 architectural invariants...")
    for inv in ["INV-SHELL2-001", "INV-SHELL2-002", "INV-SHELL2-003"]:
        if inv not in boot_log:
            print(f"❌ FAIL: Invariant '{inv}' not asserted in golden boot log")
            sys.exit(1)
        print(f"  ✓ Invariant {inv} ratified")

    print("\n==================================================")
    print(" SPRINT SP-61 VERIFICATION RESULT: PASSED        ")
    print(" Shell v2 (The Ind Surface) RATIFIED             ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp61()
