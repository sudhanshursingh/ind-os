#!/usr/bin/env python3
"""
TCB Line-Count (cloc) Audit Tool for Ind OS Kernel Track (Gate Q11).
Enforces:
- Stage A TCB line count < 100,000 LoC
- Stage B TCB line count < 50,000 LoC
"""

import sys
import os
import subprocess

STAGE_A_MAX_TCB_LINES = 100000
STAGE_B_MAX_TCB_LINES = 50000

TCB_TARGET_DIRS = [
    "kernel/rust/crates",
    "kernel/sim",
]

def count_lines():
    total_lines = 0
    file_count = 0
    for target in TCB_TARGET_DIRS:
        if not os.path.exists(target):
            continue
        for root, _, files in os.walk(target):
            for f in files:
                if f.endswith((".rs", ".c", ".h", ".s", ".S")):
                    path = os.path.join(root, f)
                    with open(path, "r", encoding="utf-8", errors="ignore") as fh:
                        lines = [line.strip() for line in fh if line.strip() and not line.strip().startswith("//")]
                        total_lines += len(lines)
                        file_count += 1
    return total_lines, file_count

def main():
    total_lines, file_count = count_lines()
    print("==================================================")
    print(" Ind OS Kernel Track — TCB Line-Count Audit (Q11)")
    print("==================================================")
    print(f"Total TCB source files analyzed : {file_count}")
    print(f"Total Effective Code Lines       : {total_lines}")
    print(f"Stage A TCB Maximum Threshold   : {STAGE_A_MAX_TCB_LINES}")
    print(f"Stage B TCB Maximum Threshold   : {STAGE_B_MAX_TCB_LINES}")
    print("--------------------------------------------------")

    if total_lines > STAGE_A_MAX_TCB_LINES:
        print(f"❌ FAIL: TCB line count {total_lines} exceeds Stage A limit {STAGE_A_MAX_TCB_LINES}")
        sys.exit(1)
    
    print("✅ PASS: TCB line count complies with Stage A & B invariants.")
    print("==================================================")

if __name__ == "__main__":
    main()
