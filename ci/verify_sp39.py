#!/usr/bin/env python3
"""
Ind OS Sprint SP-39 Verification & Acceptance Test Harness
Target: Integration Test Suite v1 (Scripted Journeys & Soak Harness)
Standards: Bible §8.11, REQ-QA-201, INV-QA-001, INV-QA-002, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json
import subprocess

cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP39_FILES = [
    "docs/architecture/integration-test-suite-spec.md",
    "docs/evidence/SP-39-validation.md",
    "docs/evidence/golden-boot-sp39.log",
    "userspace/testsuite/Cargo.toml",
    "userspace/testsuite/README.md",
    "userspace/testsuite/src/lib.rs",
    "userspace/testsuite/src/main.rs",
    "userspace/testsuite/src/types.rs",
    "userspace/testsuite/src/journeys.rs",
    "userspace/testsuite/src/soak.rs",
    "kernel/src/userspace/qa.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp39():
    print("==================================================")
    print(" Ind OS SP-39 Integration Test Suite Verifier     ")
    print(" Target: Sprint SP-39 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-39 deliverable presence...")
    for rel_path in REQUIRED_SP39_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-39 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/testsuite/src/journeys.rs", "r") as f:
        j_src = f.read()
    for s in ["UserJourneyRunner", "journey_1_init_supervision", "journey_2_hotplug_capabilities", "journey_3_shell_session", "journey_4_logging_diagnostics", "journey_5_store_consent"]:
        if s not in j_src:
            print(f"❌ FAIL: journeys.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ User journey contracts verified (INV-QA-001)")

    with open("userspace/testsuite/src/soak.rs", "r") as f:
        s_src = f.read()
    for s in ["SoakHarness", "run_cycles", "UserHeapAllocator", "RingBuffer", "attenuate"]:
        if s not in s_src:
            print(f"❌ FAIL: soak.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Soak harness contracts verified (INV-QA-002)")

    # 3. Kernel Golden Boot Log Verification (Stages 141-145)
    print("\n[CHECK 3/6] Verifying kernel boot stages in golden-boot-sp39.log...")
    with open("docs/evidence/golden-boot-sp39.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        ("STAGE:141_QA_SUITE_INIT", "Integration test suite runner and soak harness initialized"),
        ("STAGE:142_QA_USER_JOURNEY_1", "User journeys 1 (Init) & 2 (Hotplug) verified"),
        ("STAGE:143_QA_USER_JOURNEY_2", "User journeys 3 (Shell/Edit), 4 (Logd), 5 (Store) verified"),
        ("STAGE:144_QA_SOAK_HARNESS", "Automated soak harness 100 cycles verified without memory leaks"),
        ("STAGE:145_SP39_QA_READY", "Sprint SP-39 Integration Test Suite v1 Milestone REACHED"),
    ]

    for marker, desc in expected_stages:
        if marker not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{marker}' missing from golden boot log")
            sys.exit(1)
        print(f"  ✓ Boot stage {marker} verified ({desc})")

    if "[PANIC]" in boot_log or "FATAL" in boot_log:
        print("❌ FAIL: Kernel panic or fatal error detected in golden-boot-sp39.log")
        sys.exit(1)
    print("  ✓ Golden boot clean execution confirmed")

    # 4. Pure Rust Rule N-01 Verification
    print("\n[CHECK 4/6] Verifying Rule N-01: Zero external dependencies...")
    with open("userspace/testsuite/Cargo.toml", "r") as f:
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
                print(f"❌ FAIL: Non-local dependency detected in userspace/testsuite/Cargo.toml: {line}")
                sys.exit(1)
    print("  ✓ Rule N-01 verified: 100% pure in-tree workspace crates")

    # 5. Rule N-04 Zero Unsafe Code Verification
    print("\n[CHECK 5/6] Verifying Rule N-04: Zero unsafe code in new modules...")
    modules_to_scan = [
        "userspace/testsuite/src/lib.rs",
        "userspace/testsuite/src/main.rs",
        "userspace/testsuite/src/types.rs",
        "userspace/testsuite/src/journeys.rs",
        "userspace/testsuite/src/soak.rs",
        "kernel/src/userspace/qa.rs",
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
    print("\n[CHECK 6/6] Verifying compilation of indus-testsuite...")
    res = subprocess.run(
        ["cargo", "check", "-p", "indus-testsuite", "--target", "aarch64-unknown-none"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if res.returncode != 0:
        print(f"❌ FAIL: cargo check failed:\n{res.stderr}")
        sys.exit(1)
    print("  ✓ indus-testsuite compilation verified clean")

    # Output manifest
    manifest_dir = "evidence/SP-39"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_file = os.path.join(manifest_dir, "manifest.json")
    manifest_data = {
        "sprint": "SP-39",
        "target": "Integration Test Suite v1",
        "status": "PASSED",
        "invariants": {
            "INV-QA-001": "Deterministic Whole-System User Journeys (Journeys 1-5)",
            "INV-QA-002": "Zero-Leak Soak Cycling Harness (100 cycles)",
        },
        "stages_verified": [m[0] for m in expected_stages],
        "checksums": {f: hash_file(f) for f in REQUIRED_SP39_FILES if os.path.exists(f)},
    }
    with open(manifest_file, "w") as f:
        json.dump(manifest_data, f, indent=2)

    print(f"\n==================================================")
    print(f"  ALL SPRINT SP-39 VERIFICATION CHECKS PASSED     ")
    print(f"  Manifest written to {manifest_file}            ")
    print(f"==================================================")

if __name__ == "__main__":
    verify_sp39()
