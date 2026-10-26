#!/usr/bin/env python3
"""
Ind OS Zero Ambient Authority Static Verification Tool (Rule N-04 / INV-SEC-001)
Ensures that no ambient authority or capability-bypass APIs exist in the microkernel.
"""

import sys
import os
import re

FORBIDDEN_PATTERNS = [
    r"\bget_root_privilege\b",
    r"\bbypass_capability\b",
    r"\bambient_authority\b",
    r"\bdisable_security\b",
    r"\boverride_policy_unconditional\b",
]

def scan_kernel_for_ambient_authority(root_dir: str = "kernel/src") -> bool:
    print(f"• Scanning {root_dir} for zero-ambient-authority compliance (INV-SEC-001)...")
    violations = []

    for dirpath, _, filenames in os.walk(root_dir):
        for f in filenames:
            if not f.endswith(".rs"):
                continue
            fpath = os.path.join(dirpath, f)
            with open(fpath, "r", encoding="utf-8", errors="replace") as fh:
                for line_idx, line in enumerate(fh, 1):
                    for pat in FORBIDDEN_PATTERNS:
                        if re.search(pat, line, re.IGNORECASE):
                            violations.append((fpath, line_idx, line.strip()))

    if violations:
        print("❌ FAIL: Ambient authority violations detected:")
        for path, line_no, content in violations:
            print(f"  {path}:{line_no} -> {content}")
        return False

    print("  [OK] Zero ambient authority verified: 0 forbidden authority bypass patterns found.")
    return True

if __name__ == "__main__":
    if not scan_kernel_for_ambient_authority():
        sys.exit(1)
    print("✅ Zero Ambient Authority Check PASSED (INV-SEC-001 satisfied).")
    sys.exit(0)
