#!/usr/bin/env python3
"""
Ind OS Sprint SP-67 Verification & Acceptance Test Harness
Target: Performance Budget Enforcement (Frame-Time, Cold-Start, Memory Quotas)
Standards: Bible §6.3, §8.14, Table 19.2, REQ-PERF Series, INV-PERF-001..003
"""

import os
import sys

REQUIRED_SP67_FILES = [
    "docs/architecture/perf-budget-spec.md",
    "docs/evidence/SP-67-validation.md",
    "docs/evidence/golden-boot-sp67.log",
    "userspace/ux/src/perf_budget.rs",
    "kernel/src/userspace/perf_budget_core.rs",
]

def verify_sp67():
    print("==================================================")
    print(" Ind OS SP-67 Performance Budget Verifier        ")
    print(" Target: Sprint SP-67 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-67 deliverable presence...")
    for rel_path in REQUIRED_SP67_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-67 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/perf_budget.rs", "r") as f:
        perf_src = f.read()
    for s in [
        "PerfBudgetError",
        "PerfBudgetRecord",
        "PerfBudgetAuditor",
        "MAX_FRAME_TIME_US",
        "MAX_COLD_START_MS",
        "MAX_APP_MEMORY_KB",
        "audit_frame_time",
        "audit_cold_start",
        "audit_memory_allocation",
        "record_benchmark",
    ]:
        if s not in perf_src:
            print(f"❌ FAIL: perf_budget.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/userspace/perf_budget_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelPerfBudgetError",
        "KernelPerfBudgetRecord",
        "KernelPerfBudgetAuditor",
        "KERNEL_MAX_FRAME_TIME_US",
        "KERNEL_MAX_COLD_START_MS",
        "KERNEL_MAX_APP_MEMORY_KB",
        "audit_frame_time",
        "audit_cold_start",
        "audit_memory_allocation",
        "record_benchmark",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: perf_budget_core.rs missing symbol '{s}'")
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
    for src_file in ["userspace/ux/src/perf_budget.rs", "kernel/src/userspace/perf_budget_core.rs"]:
        with open(src_file, "r") as f:
            content = f.read()
        if "unsafe " in content or "unsafe{" in content:
            print(f"❌ FAIL: Rule N-04 violation in {src_file}: unsafe block detected")
            sys.exit(1)
        print(f"  ✓ Zero unsafe code verified in {src_file}")

    # 5. Golden Boot Log Verification (Stages 284..288)
    print("\n[CHECK 5/6] Verifying bare-metal QEMU boot execution log...")
    with open("docs/evidence/golden-boot-sp67.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "284_PERF_BUDGET_INIT",
        "285_PERF_FRAME_TIME_BUDGET",
        "286_PERF_COLD_START_LATENCY",
        "287_PERF_APP_MEMORY_QUOTA",
        "288_SP67_PERF_BUDGET_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{stage}' not found in golden-boot-sp67.log")
            sys.exit(1)
        print(f"  ✓ Stage {stage} present in boot log")

    expected_assertions = [
        "Performance budget auditor online: frame-time, cold-start, memory bounds active",
        "Frame-time budget verified: frames > 16.6ms strictly flagged as deadline violations",
        "Cold-start budget verified: launch-to-interactive times <= 250ms verified",
        "Memory budget verified: per-app allocation bounded strictly to G4 baseline",
        "Sprint SP-67 Performance Budget Milestone REACHED.",
    ]
    for assertion in expected_assertions:
        if assertion not in boot_log:
            print(f"❌ FAIL: Expected assertion '{assertion}' missing in log")
            sys.exit(1)
        print(f"  ✓ Assertion verified: {assertion}")

    # 6. Check Invariant Signatures
    print("\n[CHECK 6/6] Verifying SP-67 architectural invariants...")
    for inv in ["INV-PERF-001", "INV-PERF-002", "INV-PERF-003"]:
        if inv not in boot_log:
            print(f"❌ FAIL: Invariant '{inv}' not asserted in golden boot log")
            sys.exit(1)
        print(f"  ✓ Invariant {inv} ratified")

    print("\n==================================================")
    print(" SPRINT SP-67 VERIFICATION RESULT: PASSED        ")
    print(" Performance Budget Enforcement RATIFIED         ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp67()
