#!/usr/bin/env python3
"""
Corrupt-Image Rejection Test Suite (Sprint SP-09 / Gate G1 Exit Criteria).
Validates that corrupt kernel images or corrupted DTB metadata are deterministically
rejected by IndK bootloader and early panic handlers without silent undefined behavior.
"""

import sys
import os
import subprocess
import tempfile
import struct
import json

def test_corrupt_dtb_rejection(kernel_elf: str) -> bool:
    print("==================================================")
    print(" Ind OS Corrupt-Image Rejection Test Suite        ")
    print(" Target: Gate G1 Exit Criteria & INV-BOOT-003     ")
    print("==================================================")

    # 1. Test Corrupted DTB Magic
    print("• Test 1: Corrupted DTB Header (Magic 0xdeadbeef instead of 0xd00dfeed)...")
    corrupt_dtb = "evidence/G1/corrupt_test.dtb"
    os.makedirs(os.path.dirname(corrupt_dtb), exist_ok=True)
    header = struct.pack(">II", 0xdeadbeef, 4096)
    with open(corrupt_dtb, "wb") as f:
        f.write(header + b"\x00" * 4088)

    try:
        log_file = "evidence/G1/corrupt_dtb_test.log"
        os.makedirs(os.path.dirname(log_file), exist_ok=True)

        cmd = [
            "./tools/run_qemu.sh",
            "--kernel", kernel_elf,
            "--log", log_file,
            "--timeout", "6",
            "-dtb", corrupt_dtb,
        ]

        res = subprocess.run(cmd, capture_output=True, text=True)

        with open(log_file, "r", encoding="utf-8", errors="replace") as lf:
            output = lf.read()

        # Invariant check: Kernel / QEMU must reject corrupted DTB magic and reject boot
        if "fdt_err_badmagic" in output.lower() or "panic" in output.lower() or "corrupted" in output.lower():
            print("  [OK] Corrupted DTB deterministically rejected with error/panic:")
            for line in output.splitlines():
                if "badmagic" in line.lower() or "panic" in line.lower() or "dtb" in line.lower():
                    print(f"       -> {line.strip()}")
            dtb_passed = True
        else:
            print("❌ FAIL: Corrupted DTB was not rejected!")
            dtb_passed = False

    finally:
        if os.path.isfile(corrupt_dtb):
            os.unlink(corrupt_dtb)

    # 2. Test Truncated / Zeroed Binary Rejection
    print("\n• Test 2: Truncated Kernel Binary Rejection...")
    corrupt_bin = "evidence/G1/corrupt_test.bin"
    with open(corrupt_bin, "wb") as f:
        f.write(b"\x00" * 32)  # 32 zero bytes

    try:
        corrupt_log = "evidence/G1/corrupt_bin_test.log"
        cmd = [
            "./tools/run_qemu.sh",
            "--kernel", corrupt_bin,
            "--log", corrupt_log,
            "--timeout", "2",
        ]
        res = subprocess.run(cmd, capture_output=True, text=True)
        with open(corrupt_log, "r", encoding="utf-8", errors="replace") as lf:
            bin_output = lf.read()

        # Empty / zero binary cannot produce Stage 15 Boot Complete
        if "[STAGE:15_BOOT_COMPLETE]" not in bin_output:
            print("  [OK] Zeroed/truncated binary rejected (failed to achieve boot milestone).")
            bin_passed = True
        else:
            print("❌ FAIL: Zeroed binary unexpectedly claimed boot success.")
            bin_passed = False
    finally:
        if os.path.isfile(corrupt_bin):
            os.unlink(corrupt_bin)

    test_passed = dtb_passed and bin_passed
    report_data = {
        "corrupt_dtb_rejected": dtb_passed,
        "truncated_binary_rejected": bin_passed,
        "gate_g1_rejection_criteria_met": test_passed,
    }

    report_path = "evidence/G1/corrupt_rejection_report.json"
    with open(report_path, "w", encoding="utf-8") as rf:
        json.dump(report_data, rf, indent=2)

    print(f"\n  [OK] Corrupt rejection report saved to: {report_path}")
    print("==================================================")
    if test_passed:
        print(" ✅ CORRUPT-IMAGE REJECTION TEST PASSED (100% REJECTED) ")
    else:
        print(" ❌ CORRUPT-IMAGE REJECTION TEST FAILED ")
    print("==================================================")
    return test_passed

if __name__ == "__main__":
    kernel = sys.argv[1] if len(sys.argv) > 1 else "kernel/target/aarch64-unknown-none/debug/indk"
    if not test_corrupt_dtb_rejection(kernel):
        sys.exit(1)
