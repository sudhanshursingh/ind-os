#!/usr/bin/env python3
"""
Ind OS Headless QEMU Test Orchestrator (Sprint SP-03).
Coordinates:
1. Launching QEMU headless session.
2. Capturing serial output.
3. Comparing against golden log specifications using golden_log_comparator.
4. Cryptographic hash recording of test artifacts into evidence directory.
5. In-harness verification mode (--self-test).
"""

import sys
import os
import subprocess
import hashlib
import json
import time
import argparse

from golden_log_comparator import compare_logs

def calculate_sha256(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()

def run_test(kernel_path: str, golden_path: str, evidence_dir: str, timeout: int = 15) -> bool:
    os.makedirs(evidence_dir, exist_ok=True)
    serial_log = os.path.join(evidence_dir, "serial.log")
    runner_script = os.path.join(os.path.dirname(__file__), "..", "tools", "run_qemu.sh")

    cmd = [
        "bash",
        runner_script,
        "--kernel", kernel_path,
        "--log", serial_log,
        "--timeout", str(timeout),
    ]

    print(f"• Launching QEMU test runner: {' '.join(cmd)}")
    p = subprocess.run(cmd)
    qemu_exit = p.returncode

    if not os.path.isfile(serial_log):
        print(f"❌ FAIL: Serial log not generated at {serial_log}")
        return False

    with open(serial_log, "r", encoding="utf-8", errors="replace") as sf:
        actual_lines = sf.readlines()

    with open(golden_path, "r", encoding="utf-8") as gf:
        golden_patterns = [line.rstrip("\n") for line in gf if line.strip() and not line.startswith("#")]

    res = compare_logs(actual_lines, golden_patterns)

    # Generate evidence report
    result_data = {
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "kernel_path": kernel_path,
        "kernel_sha256": calculate_sha256(kernel_path) if os.path.exists(kernel_path) else None,
        "golden_path": golden_path,
        "golden_sha256": calculate_sha256(golden_path) if os.path.exists(golden_path) else None,
        "serial_log_sha256": calculate_sha256(serial_log),
        "qemu_exit_code": qemu_exit,
        "comparison_passed": res.passed,
        "message": res.message,
    }

    result_file = os.path.join(evidence_dir, "test_result.json")
    with open(result_file, "w", encoding="utf-8") as rf:
        json.dump(result_data, rf, indent=2)

    if not res.passed:
        print(f"\n❌ FAIL: Serial output deviated from golden specification:")
        print(f"  {res.message}")
        print("\n--- Unified Diff ---")
        print(res.diff)
        return False

    print(f"✅ PASS: Serial output verified against golden reference.")
    print(f"• Evidence record: {result_file}")
    return True

def run_self_test() -> bool:
    """
    Self-test harness validating:
    1. Positive scenario: matching synthetic log passes.
    2. Negative scenario: deviated synthetic log fails loudly with diff.
    """
    print("==================================================")
    print(" Ind OS Test Harness Self-Verification Loop       ")
    print(" Target: Validate Golden Comparator & Diff Engine ")
    print("==================================================")

    synthetic_golden = [
        "[BOOT] IndK ARM64 virt initializing...",
        "re:^\\[BOOT\\] git_commit: [0-9a-f]{7,40}$",
        "[BOOT] DTB magic 0xd00dfeed verified.",
        "[MEM] Initializing physical frame allocator...",
        "[IRQ] GICv2 distributor & CPU interfaces online.",
        "[SCHED] 4 priority classes initialized.",
        "[BOOT] Handing off to kernel_main()."
    ]

    matching_log = [
        "[BOOT] IndK ARM64 virt initializing...\n",
        "[BOOT] git_commit: db6dcb2\n",
        "[BOOT] DTB magic 0xd00dfeed verified.\n",
        "[MEM] Initializing physical frame allocator...\n",
        "[IRQ] GICv2 distributor & CPU interfaces online.\n",
        "[SCHED] 4 priority classes initialized.\n",
        "[BOOT] Handing off to kernel_main().\n"
    ]

    deviated_log = [
        "[BOOT] IndK ARM64 virt initializing...\n",
        "[BOOT] git_commit: db6dcb2\n",
        "[BOOT] DTB magic 0xBADF00D corrupted!\n", # Mismatch!
        "[PANIC] Halting CPU.\n"
    ]

    # Test 1: Positive match
    res_pos = compare_logs(matching_log, synthetic_golden)
    if not res_pos.passed:
        print(f"❌ FAIL: Expected positive test to pass, but failed: {res_pos.message}")
        return False
    print("  [OK] Positive test passed (exact + regex sequence).")

    # Test 2: Negative match (deviation)
    res_neg = compare_logs(deviated_log, synthetic_golden)
    if res_neg.passed:
        print("❌ FAIL: Expected negative test to fail loudly, but it passed!")
        return False
    if not res_neg.diff:
        print("❌ FAIL: Deviation detected but unified diff was empty!")
        return False
    print(f"  [OK] Negative test failed loudly as expected: {res_neg.message}")
    print("  [OK] Unified diff generated successfully:")
    for dline in res_neg.diff.splitlines()[:5]:
        print(f"       {dline}")

    print("--------------------------------------------------")
    print("✅ Test Harness Self-Test: ALL CHECKS PASSED.")
    print("==================================================")
    return True

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Ind OS QEMU Test Orchestrator")
    parser.add_argument("--self-test", action="store_true", help="Run harness self-verification test")
    parser.add_argument("--kernel", type=str, help="Path to kernel ELF")
    parser.add_argument("--golden", type=str, help="Path to golden log spec")
    parser.add_argument("--evidence", type=str, default="evidence/test", help="Evidence output directory")
    parser.add_argument("--timeout", type=int, default=15, help="QEMU execution timeout in seconds")

    args = parser.parse_args()

    if args.self_test:
        if not run_self_test():
            sys.exit(1)
        sys.exit(0)

    if not args.kernel or not args.golden:
        parser.print_help()
        sys.exit(2)

    success = run_test(args.kernel, args.golden, args.evidence, args.timeout)
    sys.exit(0 if success else 1)
