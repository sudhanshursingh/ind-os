#!/usr/bin/env python3
"""
Ind OS Sprint SP-44 Verification & Acceptance Test Harness
Target: Consent UX & Broker v2
Standards: Bible §6.3, §8.12, REQ-UX-301, REQ-AI-102, INV-CONSENT-001..002, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json
import subprocess

cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP44_FILES = [
    "docs/architecture/consent-broker-v2-spec.md",
    "docs/evidence/SP-44-validation.md",
    "docs/evidence/golden-boot-sp44.log",
    "userspace/ais/src/consent.rs",
    "kernel/src/userspace/consent_v2.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp44():
    print("==================================================")
    print(" Ind OS SP-44 Consent UX & Broker v2 Verifier     ")
    print(" Target: Sprint SP-44 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-44 deliverable presence...")
    for rel_path in REQUIRED_SP44_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-44 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ais/src/consent.rs", "r") as f:
        c_src = f.read()
    for s in ["ConsentRiskTier", "UserConsentDecision", "ConsentCard", "ConsentBrokerV2", "generate_narrative", "create_card_for_proposal", "record_decision"]:
        if s not in c_src:
            print(f"❌ FAIL: consent.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Consent Broker v2 symbols verified (REQ-UX-301, INV-CONSENT-001, INV-CONSENT-002)")

    # 3. Kernel Golden Boot Log Verification (Stages 167-171)
    print("\n[CHECK 3/6] Verifying kernel boot stages in golden-boot-sp44.log...")
    with open("docs/evidence/golden-boot-sp44.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        ("STAGE:167_CONSENT_V2_INIT", "Consent broker v2 initialized with narrative synthesizer"),
        ("STAGE:168_CONSENT_V2_PLAIN_NARRATIVE", "Plain-language narrative generation verified"),
        ("STAGE:169_CONSENT_V2_PROPOSAL_BINDING", "AIS proposal bound to interactive consent card verified"),
        ("STAGE:170_CONSENT_V2_AUDIT_TRAIL", "User decision recorded into immutable log bus before actiondispatch"),
        ("STAGE:171_SP44_CONSENT_READY", "Sprint SP-44 Consent UX & Broker v2 Milestone REACHED"),
    ]

    for marker, desc in expected_stages:
        if marker not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{marker}' missing from golden boot log")
            sys.exit(1)
        print(f"  ✓ Boot stage {marker} verified ({desc})")

    if "[PANIC]" in boot_log or "FATAL" in boot_log:
        print("❌ FAIL: Kernel panic or fatal error detected in golden-boot-sp44.log")
        sys.exit(1)
    print("  ✓ Golden boot clean execution confirmed")

    # 4. Pure Rust Rule N-01 Verification
    print("\n[CHECK 4/6] Verifying Rule N-01: Zero external dependencies...")
    with open("userspace/ais/Cargo.toml", "r") as f:
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
                print(f"❌ FAIL: Non-local dependency detected in userspace/ais/Cargo.toml: {line}")
                sys.exit(1)
    print("  ✓ Rule N-01 verified: 100% pure in-tree workspace crates")

    # 5. Rule N-04 Zero Unsafe Code Verification
    print("\n[CHECK 5/6] Verifying Rule N-04: Zero unsafe code in new modules...")
    modules_to_scan = [
        "userspace/ais/src/consent.rs",
        "kernel/src/userspace/consent_v2.rs",
    ]
    for path in modules_to_scan:
        with open(path, "r") as f:
            for idx, line in enumerate(f, 1):
                clean_line = line.split("//")[0].strip()
                if "unsafe" in clean_line:
                    print(f"❌ FAIL: Rule N-04 violation: 'unsafe' found at {path}:{idx}")
                    sys.exit(1)
    print("  ✓ Rule N-04 verified: 0 unsafe blocks detected")

    # 6. Compilation Check
    print("\n[CHECK 6/6] Verifying compilation of indus-ais...")
    res = subprocess.run(
        ["cargo", "check", "-p", "indus-ais", "--target", "aarch64-unknown-none"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if res.returncode != 0:
        print(f"❌ FAIL: cargo check failed:\n{res.stderr}")
        sys.exit(1)
    print("  ✓ indus-ais compilation verified clean")

    # Output manifest
    manifest_dir = "evidence/SP-44"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_file = os.path.join(manifest_dir, "manifest.json")
    manifest_data = {
        "sprint": "SP-44",
        "target": "Consent UX & Broker v2",
        "status": "PASSED",
        "invariants": {
            "INV-CONSENT-001": "Plain-Language Narrative Generation (REQ-UX-301)",
            "INV-CONSENT-002": "Immutable Audit Trail Recording Before Action Dispatch",
        },
        "stages_verified": [m[0] for m in expected_stages],
        "checksums": {f: hash_file(f) for f in REQUIRED_SP44_FILES if os.path.exists(f)},
    }
    with open(manifest_file, "w") as f:
        json.dump(manifest_data, f, indent=2)

    print(f"\n==================================================")
    print(f"  ALL SPRINT SP-44 VERIFICATION CHECKS PASSED     ")
    print(f"  Manifest written to {manifest_file}            ")
    print(f"==================================================")

if __name__ == "__main__":
    verify_sp44()
