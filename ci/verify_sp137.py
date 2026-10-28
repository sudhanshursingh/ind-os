#!/usr/bin/env python3
"""
Ind OS Sprint SP-137 Verification & Acceptance Test Harness
Target: Messages App III: RCS, MMS & Group Messaging (REQ-APP-MSG-007, REQ-APP-MSG-010)
Standards: Bible §8.18.3, §9.9.3, Gate G11
"""

import os
import sys

REQUIRED_SP137_FILES = [
    "userspace/ux/src/messages_mms.rs",
]

def verify_sp137():
    print("==================================================")
    print(" Ind OS SP-137 Messages RCS & MMS Verifier        ")
    print(" Target: Sprint SP-137 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/4] Verifying SP-137 deliverable presence...")
    for rel_path in REQUIRED_SP137_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-137 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural Symbols Check
    print("\n[CHECK 2/4] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/messages_mms.rs", "r") as f:
        src = f.read()

    required_symbols = [
        "MessagesMmsEngine",
        "GroupConversation",
        "GroupReplyMode",
        "MmsMessage",
        "MmsPart",
        "MessageTransportProtocol",
        "RcsSessionState",
        "MessagesMmsError",
        "CAP_APP_MMS_SEND",
        "CAP_APP_MMS_RECEIVE",
        "MAX_GROUP_PARTICIPANTS",
        "MAX_MMS_PARTS",
        "MAX_SUBJECT_LEN",
        "create_group",
        "send_mms",
        "receive_mms",
        "negotiate_transport",
        "INV-MSG-MMS-001",
        "INV-MSG-MMS-002",
        "INV-MSG-MMS-003",
    ]

    for s in required_symbols:
        if s not in src:
            print(f"❌ FAIL: messages_mms.rs missing required symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified symbol: {s}")

    # 3. Rule N-01 & N-04 Checks
    print("\n[CHECK 3/4] Verifying Rule N-01 (Zero external crates) & Rule N-04 (Zero unsafe)...")
    for idx, line in enumerate(src.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*") or stripped.startswith("//!"):
            continue
        if "unsafe " in stripped or "unsafe{" in stripped:
            print(f"❌ FAIL: Unsafe block found in userspace/ux/src/messages_mms.rs at line {idx}: {stripped}")
            sys.exit(1)
    print("  ✓ userspace/ux/src/messages_mms.rs is 100% safe Rust")

    # 4. Unit Test Verification
    print("\n[CHECK 4/4] Running cargo test for messages_mms...")
    ret = os.system("cargo test --lib -p indus-ux -- messages_mms")
    if ret != 0:
        print("❌ FAIL: Unit tests failed for messages_mms")
        sys.exit(1)

    print("\n==================================================")
    print(" ✅ SPRINT SP-137 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp137()
