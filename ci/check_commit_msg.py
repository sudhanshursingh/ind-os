#!/usr/bin/env python3
"""
CI Enforcer for Ind OS Commit Messages and Requirements Referencing (Rule N-10).
Enforces:
1. Every commit message must reference at least one valid ID: REQ-*, DEC-*, DEF-*, or PEN-*.
2. Prevents empty or malformed commit summaries.
"""

import sys
import re

ID_PATTERN = re.compile(r"\[(REQ-[A-Z0-9\-]+|DEC-[0-9]{3}|DEF-[0-9]{3}|PEN-[0-9]{3}|SP-[0-9]{2})\]")

def check_commit_message(msg):
    lines = msg.strip().split("\n")
    if not lines or not lines[0].strip():
        print("❌ FAIL: Commit message is empty.")
        return False
    
    first_line = lines[0].strip()
    match = ID_PATTERN.search(msg)
    if not match:
        print("❌ FAIL (Rule N-10): Commit message must reference a requirement or decision ID.")
        print("  Example: feat(kernel): initialize boot vector table [REQ-KRN-101]")
        print("  Allowed ID formats: [REQ-*], [DEC-*], [DEF-*], [PEN-*], [SP-*]")
        return False
    
    print(f"✅ PASS (Rule N-10): Valid reference found: {match.group(0)}")
    return True

if __name__ == "__main__":
    if len(sys.argv) > 1:
        with open(sys.argv[1], "r", encoding="utf-8") as f:
            msg = f.read()
    else:
        msg = sys.stdin.read()
        
    if not check_commit_message(msg):
        sys.exit(1)
    sys.exit(0)
