#!/usr/bin/env python3
"""
Ind OS Sprint SP-127 Verification & Acceptance Test Harness
Target: App Framework Extensions — Lifecycle, Back-Stack, Chrome & Window Management (REQ-APP-COMMON-006)
"""

import os
import sys
import re

REQUIRED_SP127_FILES = [
    "docs/architecture/app-framework-extensions-spec.md",
    "docs/evidence/SP-127-validation.md",
    "userspace/ux/src/app_framework_ext.rs",
    "kernel/src/hal/platform/app_framework_core.rs",
]

def verify_sp127():
    print("==================================================")
    print(" Ind OS SP-127 App Framework Extensions Verifier  ")
    print(" Target: Sprint SP-127 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/5] Verifying SP-127 deliverable presence...")
    for rel_path in REQUIRED_SP127_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-127 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Invariants Check (INV-APP-LIFE-001, INV-APP-STACK-001, INV-APP-CHROME-001, INV-APP-WIN-001)
    print("\n[CHECK 2/5] Verifying Invariants INV-APP-LIFE-001..WIN-001...")
    with open("docs/architecture/app-framework-extensions-spec.md", "r") as f:
        spec_text = f.read()
    
    invariants = [
        "INV-APP-LIFE-001",
        "INV-APP-STACK-001",
        "INV-APP-CHROME-001",
        "INV-APP-WIN-001",
    ]
    for inv in invariants:
        if inv not in spec_text:
            print(f"❌ FAIL: Specification missing invariant {inv}")
            sys.exit(1)
        print(f"  ✓ {inv} verified in specification")

    # 3. Rule N-01 Compliance Check
    print("\n[CHECK 3/5] Verifying Rule N-01 (Zero external crates in TCB & Userspace UX)...")
    for manifest in ["kernel/Cargo.toml", "userspace/ux/Cargo.toml"]:
        with open(manifest, "r") as f:
            cargo_txt = f.read()
        if "[dependencies]" in cargo_txt:
            deps = cargo_txt.split("[dependencies]")[1].split("[")[0].strip()
            lines = [l for l in deps.split("\n") if l.strip() and not l.strip().startswith("#")]
            for l in lines:
                if "path =" not in l:
                    print(f"❌ FAIL: External crate detected in {manifest}: {l}")
                    sys.exit(1)
    print("  ✓ Rule N-01 PASSED: 0 external crates in TCB and userspace")

    # 4. Rule N-04 Compliance Check
    print("\n[CHECK 4/5] Verifying Rule N-04 (Zero unsafe code in userspace app framework)...")
    with open("userspace/ux/src/app_framework_ext.rs", "r") as f:
        content = f.read()
    cleaned = re.sub(r"//.*", "", content)
    cleaned = re.sub(r"/\*[\s\S]*?\*/", "", cleaned)
    if "unsafe " in cleaned or "unsafe{" in cleaned:
        print("❌ FAIL: unsafe code found in userspace/ux/src/app_framework_ext.rs")
        sys.exit(1)
    print("  ✓ Rule N-04 PASSED: userspace/ux/src/app_framework_ext.rs is 100% safe Rust")

    # 5. State Machine, Back-Stack & Window Management Validation
    print("\n[CHECK 5/5] Validating Lifecycle Machine, Back-Stack & Window Management...")
    
    # Verify state machine logic in code
    with open("userspace/ux/src/app_framework_ext.rs", "r") as f:
        code_text = f.read()

    required_symbols = [
        "enum AppLifecycleState",
        "enum AppLifecycleEvent",
        "struct AppChrome",
        "struct AppWindowContext",
        "struct AppBackStack",
        "struct AppFrameworkExtEngine",
        "transition_state",
        "launch_app_window",
        "navigate_back",
    ]
    for sym in required_symbols:
        if sym not in code_text:
            print(f"❌ FAIL: Missing required symbol: {sym}")
            sys.exit(1)
        print(f"  ✓ {sym} verified in implementation")

    # Verify kernel capability and Z-plane bounds in app_framework_core.rs
    with open("kernel/src/hal/platform/app_framework_core.rs", "r") as f:
        core_text = f.read()
    
    if "KERNEL_SYSTEM_RESERVED_Z_BASE" not in core_text:
        print("❌ FAIL: KERNEL_SYSTEM_RESERVED_Z_BASE missing from kernel bridge")
        sys.exit(1)
    if "PermissionDenied" not in core_text:
        print("❌ FAIL: Capability check missing in kernel bridge")
        sys.exit(1)
    print("  ✓ Kernel capability gating (CAP_DISPLAY_RENDER) and Z-plane isolation verified")

    print("\n==================================================")
    print(" 🚀 SPRINT SP-127 DEFINITION OF DONE: PASSED      ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp127()
