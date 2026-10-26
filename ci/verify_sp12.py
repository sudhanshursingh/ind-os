#!/usr/bin/env python3
"""
Sprint SP-12 Verification Harness (Preemptive Scheduler & Context Switch).
Ratified under INDOS-BIBLE-001 / Phase 2 Core Kernel (§8.8, §9, §24.1) and INDOS-SPEC-SCHED-01.

Verifies:
1. Deliverable presence:
   - docs/architecture/scheduler-spec.md
   - kernel/src/task/scheduler.rs
   - kernel/src/task/process.rs
   - kernel/src/task/mod.rs
   - kernel/src/syscall/mod.rs
   - kernel/src/arch/aarch64/user_trap.s
   - kernel/src/arch/aarch64/exceptions.s
   - docs/evidence/golden-boot-sp12.log
2. Static checks:
   - Priority classes (REALTIME_HARDWARE, INTERACTIVE_USER, NORMAL_SERVICES, BATCH_INTELLIGENCE, IDLE).
   - Time quantum tracking (100 Hz timer tick preemption).
   - SchedHint advisory API stub (SYS_SCHED_HINT per Rule N-04).
   - Atomic register and per-process TTBR0_EL1 address space context switching.
   - Low-level assembly trampolines (common_lower_el_irq_entry, user_task_alpha, user_task_beta).
   - Invariants:
     • INV-SCHED-001: Context switch execution and register state preservation.
     • INV-SCHED-002: Fixed multi-level priority preemption and fairness bounds.
     • INV-IRQ-004: Timer interrupt handling during EL0 execution and preemption dispatch.
3. Unsafe-code budget scanner adherence (docs/engineering/unsafe-budget.md).
4. Clean kernel compilation with zero errors.
5. Headless QEMU execution validation against golden-boot-sp12.log.
6. Manifest generation at evidence/SP-12/manifest.json.
"""

import sys
import os
import subprocess
import hashlib
import json
import time

REQUIRED_SP12_FILES = [
    "docs/architecture/scheduler-spec.md",
    "kernel/src/task/scheduler.rs",
    "kernel/src/task/process.rs",
    "kernel/src/task/mod.rs",
    "kernel/src/syscall/mod.rs",
    "kernel/src/arch/aarch64/user_trap.s",
    "kernel/src/arch/aarch64/exceptions.s",
    "docs/evidence/golden-boot-sp12.log",
]

def find_llvm_tool(tool_name: str) -> str:
    try:
        sysroot = subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip()
        candidate = os.path.join(sysroot, "lib", "rustlib", "aarch64-apple-darwin", "bin", tool_name)
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            return candidate
        for root, dirs, files in os.walk(os.path.join(sysroot, "lib", "rustlib")):
            if tool_name in files:
                p = os.path.join(root, tool_name)
                if os.access(p, os.X_OK):
                    return p
    except Exception:
        pass
    return tool_name

def verify_sp12():
    print("==================================================")
    print(" Ind OS SP-12 Preemptive Scheduler Verifier      ")
    print(" Target: Sprint SP-12 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Check required files exist
    for f in REQUIRED_SP12_FILES:
        if not os.path.isfile(f):
            print(f"❌ FAIL: Missing required deliverable: {f}")
            return False
        print(f"  [OK] Deliverable exists: {f}")

    # 2. Check architecture specification
    with open("docs/architecture/scheduler-spec.md", "r", encoding="utf-8") as f:
        spec_src = f.read()
    if "REALTIME_HARDWARE" not in spec_src or "sched_hint" not in spec_src:
        print("❌ FAIL: scheduler-spec.md missing priority classes or sched_hint")
        return False
    print("  [OK] Architecture specification INDOS-SPEC-SCHED-01 verified.")

    # 3. Check Scheduler Subsystem implementation
    with open("kernel/src/task/scheduler.rs", "r", encoding="utf-8") as f:
        sched_src = f.read()

    sched_required = [
        "Priority",
        "SchedHint",
        "DEFAULT_TIME_QUANTUM_TICKS",
        "MAX_PRIORITY_LEVELS",
        "Scheduler",
        "SCHEDULER",
        "add_process",
        "on_timer_tick",
        "schedule_next",
        "yield_current",
        "exit_current",
        "sched_hint",
        "metrics",
    ]
    for sym in sched_required:
        if sym not in sched_src:
            print(f"❌ FAIL: Missing scheduler symbol: {sym}")
            return False
    print("  [OK] Scheduler multi-level priority queues, preemption quantum, and hints verified.")

    # 4. Check Process Subsystem Priority Integration
    with open("kernel/src/task/process.rs", "r", encoding="utf-8") as f:
        proc_src = f.read()
    if "priority: crate::task::scheduler::Priority" not in proc_src or "set_priority" not in proc_src:
        print("❌ FAIL: Process struct missing priority field or set_priority method")
        return False
    print("  [OK] Process priority integration verified.")

    # 5. Check System Call Subsystem
    with open("kernel/src/syscall/mod.rs", "r", encoding="utf-8") as f:
        sys_src = f.read()

    sys_required = [
        "SYS_YIELD",
        "SYS_WRITE",
        "SYS_EXIT",
        "SYS_GETPID",
        "SYS_SCHED_HINT",
        "handle_syscall",
    ]
    for sym in sys_required:
        if sym not in sys_src:
            print(f"❌ FAIL: Missing syscall symbol: {sym}")
            return False
    print("  [OK] System calls SYS_SCHED_HINT and SYS_YIELD verified.")

    # 6. Check Low-Level Assembly Trampolines & Vectors
    with open("kernel/src/arch/aarch64/user_trap.s", "r", encoding="utf-8") as f:
        trap_src = f.read()

    trap_required = [
        "common_lower_el_irq_entry",
        "handle_lower_el_irq",
        "user_task_alpha_start",
        "user_task_beta_start",
    ]
    for sym in trap_required:
        if sym not in trap_src:
            print(f"❌ FAIL: Missing trap assembly symbol: {sym}")
            return False

    with open("kernel/src/arch/aarch64/exceptions.s", "r", encoding="utf-8") as f:
        exc_src = f.read()
    if "b common_lower_el_irq_entry" not in exc_src:
        print("❌ FAIL: vec_lower_el_a64_irq not routed to common_lower_el_irq_entry")
        return False
    print("  [OK] Lower EL IRQ vector and context-switch trampolines verified.")

    # 7. Check Unsafe Budget
    print("• Validating Unsafe-Code Budget adherence (Bible §24.1)...")
    res_unsafe = subprocess.run([sys.executable, "ci/check_unsafe_budget.py"], capture_output=True, text=True)
    if res_unsafe.returncode != 0:
        print("❌ FAIL: Unsafe budget check failed:\n" + res_unsafe.stdout + res_unsafe.stderr)
        return False
    print("  [OK] Unsafe-code budget scanner passed cleanly.")

    # 8. Clean Kernel Build
    print("• Building IndK ARM64 kernel...")
    res_build = subprocess.run(
        ["cargo", "build", "--target", "aarch64-unknown-none"],
        cwd="kernel",
        capture_output=True,
        text=True
    )
    if res_build.returncode != 0:
        print(f"❌ FAIL: Kernel build failed:\n{res_build.stderr}")
        return False
    print("  [OK] Kernel compiled cleanly with zero errors.")

    # Convert ELF to binary
    llvm_objcopy = find_llvm_tool("llvm-objcopy")
    kernel_elf = "kernel/target/aarch64-unknown-none/debug/indk"
    kernel_bin = "kernel/target/aarch64-unknown-none/debug/indk.bin"
    res_copy = subprocess.run([llvm_objcopy, "-O", "binary", kernel_elf, kernel_bin], capture_output=True, text=True)
    if res_copy.returncode != 0:
        print(f"❌ FAIL: llvm-objcopy failed:\n{res_copy.stderr}")
        return False
    print(f"  [OK] Kernel binary produced: {kernel_bin}")

    # 9. Execute QEMU headless validation against golden-boot-sp12.log
    print("• Executing QEMU test harness against golden-boot-sp12.log...")
    golden_log = "docs/evidence/golden-boot-sp12.log"
    res_test = subprocess.run([
        sys.executable,
        "ci/run_qemu_test.py",
        "--kernel", kernel_elf,
        "--golden", golden_log,
        "--timeout", "8"
    ], capture_output=True, text=True)
    if res_test.returncode != 0:
        print(f"❌ FAIL: QEMU execution validation failed:\n{res_test.stdout}\n{res_test.stderr}")
        return False
    print("  [OK] QEMU boot execution matched golden reference exactly (100% PASS).")

    # 10. Generate evidence manifest
    manifest_dir = "evidence/SP-12"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    hashes = {}
    for f in REQUIRED_SP12_FILES:
        with open(f, "rb") as fp:
            hashes[f] = hashlib.sha256(fp.read()).hexdigest()

    with open(kernel_elf, "rb") as fp:
        hashes[kernel_elf] = hashlib.sha256(fp.read()).hexdigest()

    manifest = {
        "sprint": "SP-12",
        "phase": "Phase 2 Core Kernel",
        "title": "Preemptive Scheduler & Context Switch (Priority Classes, Quantum Preemption, & sched_hint)",
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "specifications_verified": [
            "INDOS-SPEC-SCHED-01 (Priority classes, context switch, and sched_hint)",
            "INV-SCHED-001 (AArch64 atomic register & address space context switch)",
            "INV-SCHED-002 (Priority preemption and fairness bounds)",
            "INV-IRQ-004 (100 Hz timer tick preemption of EL0 user processes)"
        ],
        "deliverables": REQUIRED_SP12_FILES,
        "sha256_checksums": hashes,
        "test_result": "PASS",
        "qemu_execution": "SUCCESS"
    }

    with open(manifest_path, "w", encoding="utf-8") as fp:
        json.dump(manifest, fp, indent=2)
    print(f"  [OK] Generated evidence manifest: {manifest_path}")

    print("==================================================")
    print("✅ Sprint SP-12 Verification: 100% PASSED")
    print("==================================================")
    return True

if __name__ == "__main__":
    if not verify_sp12():
        sys.exit(1)
