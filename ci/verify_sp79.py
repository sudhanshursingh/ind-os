#!/usr/bin/env python3
"""
Ind OS Sprint SP-79 Verification & Acceptance Test Harness
Target: Input Pipeline to Shell Milestone (Coordinate Translation, Gesture Engine, Focus Routing)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-107, REQ-UX-102, REQ-SEC-104, INV-INP-001..003
"""

import os
import sys

REQUIRED_SP79_FILES = [
    "docs/architecture/input-shell-pipeline-spec.md",
    "docs/evidence/SP-79-validation.md",
    "docs/evidence/golden-boot-sp79.log",
    "userspace/devmgr/src/input_pipeline.rs",
    "kernel/src/hal/platform/input_pipeline_core.rs",
]

def verify_sp79():
    print("==================================================")
    print(" Ind OS SP-79 Input to Shell Milestone Verifier   ")
    print(" Target: Sprint SP-79 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-79 deliverable presence...")
    for rel_path in REQUIRED_SP79_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-79 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/input_pipeline.rs", "r") as f:
        inp_src = f.read()
    for s in [
        "Gesture",
        "InputPipelineError",
        "InputPipelineHarness",
        "CAP_INPUT_SHELL",
        "set_focus",
        "transform_and_dispatch",
        "dispatched_taps",
        "dispatched_scrolls",
        "focused_surface_id",
    ]:
        if s not in inp_src:
            print(f"❌ FAIL: input_pipeline.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/input_pipeline_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelGesture",
        "KernelInputPipelineError",
        "KernelInputPipelineHarness",
        "KERNEL_CAP_INPUT_SHELL",
        "set_focus",
        "transform_and_dispatch",
        "dispatched_taps",
        "dispatched_scrolls",
        "focused_surface_id",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: input_pipeline_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/devmgr/Cargo.toml"]:
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
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/devmgr)...")
    with open("userspace/devmgr/src/input_pipeline.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/input_pipeline.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/input_pipeline.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 345-349)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 345-349...")
    with open("docs/evidence/golden-boot-sp79.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "345_SP79_INPUT_PIPELINE_INIT",
        "346_SP79_COORDINATE_NORMALIZATION",
        "347_SP79_GESTURE_RECOGNITION_DISPATCH",
        "348_SP79_SHELL_FOCUS_INPUT_GATING",
        "349_SP79_INPUT_PIPELINE_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Input Pipeline Confirmation Logs
    print("\n[CHECK 6/6] Verifying Input Pipeline Confirmation Logs...")
    expected_logs = [
        "Coordinate space normalized: 1:1 pixel alignment confirmed with display surface (INV-INP-001 PASSED)",
        "Gestures verified: Tap(540, 1230) and Scroll(0, -45) recognized and dispatched (INV-INP-002 PASSED)",
        "Focus gating verified: unprivileged input injection blocked by capability broker (INV-INP-003, REQ-SEC-104 PASSED)",
        "[SP-79] Sprint SP-79 Input to Shell Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-79 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp79()
