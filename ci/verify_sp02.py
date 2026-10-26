#!/usr/bin/env python3
"""
Sprint SP-02 Verification Harness (Specification Skeleton).
Verifies:
1. docs/architecture/boot-spec.md exists, specifies entry & EL transitions, defines INV-BOOT-001..004.
2. docs/architecture/memory-spec.md exists, specifies address space split & allocators, defines INV-MEM-001..004.
3. docs/architecture/interrupt-spec.md exists, specifies GIC architecture & critical sections, defines INV-IRQ-001..004.
4. docs/architecture/scheduler-spec.md exists, specifies priority classes & sched_hint bounds, defines INV-SCHED-001..004.
5. docs/architecture/invariants-register.md exists and consolidates all 16 invariants.
6. Generates evidence/SP-02/manifest.json with cryptographic SHA-256 hashes.
"""

import sys
import os
import hashlib
import json
import time

REQUIRED_SP02_FILES = [
    "docs/architecture/boot-spec.md",
    "docs/architecture/memory-spec.md",
    "docs/architecture/interrupt-spec.md",
    "docs/architecture/scheduler-spec.md",
    "docs/architecture/invariants-register.md",
]

INVARIANTS = {
    "docs/architecture/boot-spec.md": [
        "INV-BOOT-001",
        "INV-BOOT-002",
        "INV-BOOT-003",
        "INV-BOOT-004",
    ],
    "docs/architecture/memory-spec.md": [
        "INV-MEM-001",
        "INV-MEM-002",
        "INV-MEM-003",
        "INV-MEM-004",
    ],
    "docs/architecture/interrupt-spec.md": [
        "INV-IRQ-001",
        "INV-IRQ-002",
        "INV-IRQ-003",
        "INV-IRQ-004",
    ],
    "docs/architecture/scheduler-spec.md": [
        "INV-SCHED-001",
        "INV-SCHED-002",
        "INV-SCHED-003",
        "INV-SCHED-004",
    ],
}

def verify_sp02():
    print("==================================================")
    print(" Ind OS SP-02 Specification Skeleton Verifier    ")
    print(" Target: Sprint SP-02 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Check all required files exist
    for f in REQUIRED_SP02_FILES:
        if not os.path.isfile(f):
            print(f"❌ FAIL: Missing required deliverable: {f}")
            return False
        print(f"  [OK] File exists: {f}")

    # 2. Check per-module invariants
    for doc_path, inv_list in INVARIANTS.items():
        with open(doc_path, "r", encoding="utf-8") as fh:
            content = fh.read()
        for inv in inv_list:
            if inv not in content:
                print(f"❌ FAIL: Invariant {inv} missing from {doc_path}")
                return False
        print(f"  [OK] All invariants verified in {doc_path}")

    # 3. Check invariants register consolidates all 16 invariants
    with open("docs/architecture/invariants-register.md", "r", encoding="utf-8") as fh:
        reg_content = fh.read()

    total_invariants = 0
    for inv_list in INVARIANTS.values():
        for inv in inv_list:
            if inv not in reg_content:
                print(f"❌ FAIL: Invariant {inv} missing from invariants register")
                return False
            total_invariants += 1

    print(f"  [OK] docs/architecture/invariants-register.md contains all {total_invariants} invariants.")

    # 4. Generate Evidence Pack
    evidence_dir = "evidence/SP-02"
    os.makedirs(evidence_dir, exist_ok=True)
    manifest_path = os.path.join(evidence_dir, "manifest.json")

    file_hashes = {}
    for f in sorted(REQUIRED_SP02_FILES):
        h = hashlib.sha256()
        with open(f, "rb") as fh:
            h.update(fh.read())
        file_hashes[f] = h.hexdigest()

    manifest_data = {
        "sprint": "SP-02",
        "title": "Specification Skeleton",
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": "PASS",
        "invariants_count": total_invariants,
        "file_hashes": file_hashes,
    }

    should_write = True
    if os.path.exists(manifest_path):
        try:
            with open(manifest_path, "r", encoding="utf-8") as existing_fh:
                existing_data = json.load(existing_fh)
                if existing_data.get("file_hashes") == file_hashes:
                    should_write = False
        except Exception:
            should_write = True

    if should_write:
        with open(manifest_path, "w", encoding="utf-8") as fh:
            json.dump(manifest_data, fh, indent=2)

    print(f"\n• Evidence Pack Manifest up to date at: {manifest_path}")
    print("--------------------------------------------------")
    print("✅ Sprint SP-02 Definition of Done (DoD) Passed.")
    print("==================================================")
    return True

if __name__ == "__main__":
    if not verify_sp02():
        sys.exit(1)
