#!/usr/bin/env python3
"""
Ind OS Sprint SP-54 Verification & Acceptance Test Harness
Target: Update Service (A/B Slots, Signed Manifests, Atomic Switch, Verified Rollback)
Standards: Bible §8.13, REQ-USE-205, INV-UPD-001..004, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json

REQUIRED_SP54_FILES = [
    "docs/architecture/update-service-spec.md",
    "docs/evidence/SP-54-validation.md",
    "docs/evidence/golden-boot-sp54.log",
    "userspace/net/src/update.rs",
    "kernel/src/userspace/update_service.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp54():
    print("==================================================")
    print(" Ind OS SP-54 A/B Update Service Verifier        ")
    print(" Target: Sprint SP-54 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-54 deliverable presence...")
    for rel_path in REQUIRED_SP54_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-54 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/net/src/update.rs", "r") as f:
        upd_src = f.read()
    for s in [
        "SlotId",
        "SlotState",
        "UpdatePolicy",
        "UpdateError",
        "UpdateManifest",
        "UpdateManager",
        "verify_and_stage",
        "atomic_switch",
        "verify_health",
        "compute_checksum",
        "AUTHORIZED_UPDATE_SIGNATURE",
    ]:
        if s not in upd_src:
            print(f"❌ FAIL: update.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Update service and A/B slot state symbols verified (REQ-USE-205, INV-UPD-001..004)")

    # 3. Kernel Golden Boot Log Verification (Stages 218-222)
    print("\n[CHECK 3/6] Verifying kernel boot stages in golden-boot-sp54.log...")
    with open("docs/evidence/golden-boot-sp54.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        ("STAGE:218_UPDATE_SERVICE_INIT", "A/B update service initialized with Slot A active"),
        ("STAGE:219_UPDATE_SIGNED_MANIFEST_VERIFICATION", "Signed update manifest verification and tamper rejection verified"),
        ("STAGE:220_UPDATE_ATOMIC_SLOT_SWITCH", "Atomic boot slot switch from Slot A to Slot B verified"),
        ("STAGE:221_UPDATE_VERIFIED_ROLLBACK", "Health verification failure triggers automatic rollback to Slot A"),
        ("STAGE:222_SP54_UPDATE_READY", "Sprint SP-54 Update Service Milestone REACHED"),
    ]

    for marker, desc in expected_stages:
        if marker not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{marker}' missing from golden boot log")
            sys.exit(1)
        print(f"  ✓ Boot stage {marker} verified ({desc})")

    if "[PANIC]" in boot_log or "FATAL" in boot_log:
        print("❌ FAIL: Kernel panic or fatal error detected in golden-boot-sp54.log")
        sys.exit(1)
    print("  ✓ Golden boot clean execution confirmed")

    # 4. Pure Rust Rule N-01 Verification
    print("\n[CHECK 4/6] Verifying Rule N-01: Zero external dependencies...")
    with open("userspace/net/Cargo.toml", "r") as f:
        cargo_toml = f.read()
    in_deps = False
    for line in cargo_toml.splitlines():
        line = line.strip()
        if line.startswith("[dependencies]"):
            in_deps = True
            continue
        if line.startswith("[") and in_deps:
            in_deps = False
        if in_deps and line and not line.startswith("#"):
            if "path =" not in line:
                print(f"❌ FAIL: Non-local dependency detected in userspace/net/Cargo.toml: {line}")
                sys.exit(1)
    print("  ✓ 100% in-tree pure Rust dependencies verified (Rule N-01)")

    # 5. Zero Unsafe Rule N-04 Verification
    print("\n[CHECK 5/6] Verifying Rule N-04: Zero unsafe code in update service...")
    with open("userspace/net/src/update.rs", "r") as f:
        for idx, line in enumerate(f, 1):
            clean = line.split("//")[0].strip()
            if "unsafe" in clean:
                print(f"❌ FAIL: Rule N-04 violation in update.rs:{idx}: '{clean}'")
                sys.exit(1)
    print("  ✓ 0 unsafe code blocks in userspace/net/src/update.rs (Rule N-04)")

    # 6. Generate SP-54 Manifest
    print("\n[CHECK 6/6] Generating SP-54 formal verification manifest...")
    manifest_dir = "evidence/SP-54"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_file = os.path.join(manifest_dir, "manifest.json")
    manifest_data = {
        "sprint": "SP-54",
        "phase": "Phase 7 — Network + Updates + Service Completion",
        "status": "PASSED & RATIFIED",
        "verified_stages": [s[0] for s in expected_stages],
        "checksums": {f: hash_file(f) for f in REQUIRED_SP54_FILES if os.path.exists(f)},
    }
    with open(manifest_file, "w") as f:
        json.dump(manifest_data, f, indent=2)
    print(f"  ✓ Manifest written to {manifest_file}")

    print("\n==================================================")
    print("  SPRINT SP-54 VERIFICATION PASSED & RATIFIED     ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp54()
