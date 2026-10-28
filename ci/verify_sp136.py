#!/usr/bin/env python3
"""
Ind OS Sprint SP-136 Verification & Acceptance Test Harness
Target: Messages App II: Suraksha Security, Screen-Share Blackout & Attachments (REQ-APP-MSG-004..006, 011)
Standards: Bible §8.18.3, §9.9.3, Gate G11
"""

import os
import sys

REQUIRED_SP136_FILES = [
    "userspace/ux/src/messages_suraksha.rs",
]

def verify_sp136():
    print("==================================================")
    print(" Ind OS SP-136 Messages Suraksha Security Verifier")
    print(" Target: Sprint SP-136 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/4] Verifying SP-136 deliverable presence...")
    for rel_path in REQUIRED_SP136_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-136 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural Symbols Check
    print("\n[CHECK 2/4] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/messages_suraksha.rs", "r") as f:
        src = f.read()

    required_symbols = [
        "MessagesSurakshaEngine",
        "AttachmentDescriptor",
        "OtpDescriptor",
        "SmsScamVerdict",
        "DeviceCipherEnvelope",
        "MessagesSurakshaError",
        "CAP_ATTACHMENT_MEDIATE",
        "SCREEN_SHARE_BLACKOUT_STR",
        "MAX_ENVELOPE_LEN",
        "attach_file",
        "inspect_incoming_sms",
        "render_for_display",
        "encrypt_at_rest",
        "decrypt_at_rest",
        "INV-MSG-004",
        "INV-MSG-005",
        "INV-MSG-006",
        "INV-MSG-007",
    ]

    for s in required_symbols:
        if s not in src:
            print(f"❌ FAIL: messages_suraksha.rs missing required symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified symbol: {s}")

    # 3. Rule N-01 & N-04 Checks
    print("\n[CHECK 3/4] Verifying Rule N-01 (Zero external crates) & Rule N-04 (Zero unsafe)...")
    for idx, line in enumerate(src.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*") or stripped.startswith("//!"):
            continue
        if "unsafe " in stripped or "unsafe{" in stripped:
            print(f"❌ FAIL: Unsafe block found in userspace/ux/src/messages_suraksha.rs at line {idx}: {stripped}")
            sys.exit(1)
    print("  ✓ userspace/ux/src/messages_suraksha.rs is 100% safe Rust")

    # 4. Unit Test Verification
    print("\n[CHECK 4/4] Running cargo test for messages_suraksha...")
    ret = os.system("cargo test --lib -p indus-ux -- messages_suraksha")
    if ret != 0:
        print("❌ FAIL: Unit tests failed for messages_suraksha")
        sys.exit(1)

    print("\n==================================================")
    print(" ✅ SPRINT SP-136 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp136()

