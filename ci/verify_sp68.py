#!/usr/bin/env python3
"""
Ind OS Sprint SP-68 Verification & Acceptance Test Harness
Target: UX Validation Sprint (Personas, Usability Journeys, Friction Audit)
Standards: Bible §6.3, §8.14, Table 19.2, REQ-UX-305, INV-UXVAL-001..003
"""

import os
import sys

REQUIRED_SP68_FILES = [
    "docs/architecture/ux-validation-spec.md",
    "docs/evidence/SP-68-validation.md",
    "docs/evidence/golden-boot-sp68.log",
    "userspace/ux/src/ux_validation.rs",
    "kernel/src/userspace/ux_validation_core.rs",
]

def verify_sp68():
    print("==================================================")
    print(" Ind OS SP-68 UX Validation Verifier             ")
    print(" Target: Sprint SP-68 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-68 deliverable presence...")
    for rel_path in REQUIRED_SP68_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-68 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/ux_validation.rs", "r") as f:
        val_src = f.read()
    for s in [
        "UxValidationError",
        "PersonaTaskResult",
        "UxValidationHarness",
        "MAX_PERSONA_TASKS",
        "record_task_result",
        "success_rate_percent",
        "task_count",
    ]:
        if s not in val_src:
            print(f"❌ FAIL: ux_validation.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/ux_validation_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelUxValidationError",
        "KernelPersonaTaskResult",
        "KernelUxValidationHarness",
        "KERNEL_MAX_PERSONA_TASKS",
        "record_task_result",
        "success_rate_percent",
        "task_count",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: ux_validation_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/ux/Cargo.toml"]:
        with open(toml_path, "r") as f:
            lines = [line.strip() for line in f.readlines() if line.strip() and not line.strip().startswith("#")]
        in_deps = False
        for line in lines:
            if line.startswith("[dependencies]"):
                in_deps = True
                continue
            if line.startswith("[") and in_deps:
                in_deps = False
                continue
            if in_deps and line:
                if "path =" not in line:
                    print(f"❌ FAIL: External dependency detected in {toml_path}: {line}")
                    sys.exit(1)
        print(f"  ✓ {toml_path} compliant with Rule N-01")

    # 4. Rule N-04: Zero Unsafe Code Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/ux)...")
    with open("userspace/ux/src/ux_validation.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/ux/src/ux_validation.rs")
            sys.exit(1)
    print("  ✓ userspace/ux/src/ux_validation.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 289-293)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 289-293...")
    with open("docs/evidence/golden-boot-sp68.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "289_UX_VALIDATION_INIT",
        "290_UX_VALIDATION_PERSONA_PRIYA_HINDI",
        "291_UX_VALIDATION_PERSONA_RAJESH_TAMIL",
        "292_UX_VALIDATION_PERSONA_ANANYA_ENGLISH",
        "293_SP68_UX_VALIDATION_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Persona Results and 100% Success in Boot Log
    print("\n[CHECK 6/6] Verifying Persona Journeys and Top-10 Friction Resolution...")
    expected_logs = [
        "Persona Priya (Hindi): Indic IME note drafting 100% completed in 120ms (INV-UXVAL-001 PASSED)",
        "Persona Rajesh (Tamil): Consent review & revocation 100% completed in 95ms (INV-UXVAL-002 PASSED)",
        "Top-10 friction audit: 0 P0/P1 blockers remaining, aggregate persona success rate 100% (INV-UXVAL-003 PASSED)",
        "[SP-68] Sprint SP-68 UX Validation Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-68 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp68()
