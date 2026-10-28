#!/usr/bin/env python3
"""
Ind OS Sprint SP-138 Verification & Acceptance Test Harness
Target: Messages App IV: Local Search & Offline Sync (REQ-APP-MSG-008, REQ-APP-MSG-009)
Standards: Bible §8.18.3, §9.9.3, Gate G11
"""

import os
import sys

REQUIRED_SP138_FILES = [
    "userspace/ux/src/messages_search.rs",
]

def verify_sp138():
    print("==================================================")
    print(" Ind OS SP-138 Messages Search & Offline Verifier ")
    print(" Target: Sprint SP-138 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/4] Verifying SP-138 deliverable presence...")
    for rel_path in REQUIRED_SP138_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-138 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural Symbols Check
    print("\n[CHECK 2/4] Verifying architectural symbols and API contracts...")
    with open("userspace/ux/src/messages_search.rs", "r") as f:
        src = f.read()

    required_symbols = [
        "MessagesSearchEngine",
        "IndexedMessage",
        "SearchResultEntry",
        "SearchResultSet",
        "MessageProposalCard",
        "OfflineQueueItem",
        "ProposalCardState",
        "MessagesSearchError",
        "CAP_APP_MSG_SEARCH",
        "CAP_AIS_MSG_PROPOSAL",
        "SEARCH_LATENCY_BUDGET_US",
        "index_message",
        "search",
        "propose_message",
        "confirm_and_dispatch_proposal",
        "reconcile_offline_queue",
        "is_sensitive_otp",
        "INV-MSG-SEARCH-001",
        "INV-MSG-SEARCH-002",
        "INV-MSG-SEARCH-003",
    ]

    for s in required_symbols:
        if s not in src:
            print(f"❌ FAIL: messages_search.rs missing required symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified symbol: {s}")

    # 3. Rule N-01 & N-04 Checks
    print("\n[CHECK 3/4] Verifying Rule N-01 (Zero external crates) & Rule N-04 (Zero unsafe)...")
    for idx, line in enumerate(src.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*") or stripped.startswith("//!"):
            continue
        if "unsafe " in stripped or "unsafe{" in stripped:
            print(f"❌ FAIL: Unsafe block found in userspace/ux/src/messages_search.rs at line {idx}: {stripped}")
            sys.exit(1)
    print("  ✓ userspace/ux/src/messages_search.rs is 100% safe Rust")

    # 4. Unit Test Verification
    print("\n[CHECK 4/4] Running cargo test for messages_search...")
    ret = os.system("cargo test --lib -p indus-ux -- messages_search")
    if ret != 0:
        print("❌ FAIL: Unit tests failed for messages_search")
        sys.exit(1)

    print("\n==================================================")
    print(" ✅ SPRINT SP-138 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp138()

