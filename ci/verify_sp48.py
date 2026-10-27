#!/usr/bin/env python3
"""
Ind OS Sprint SP-48 Verification & Acceptance Test Harness
Target: Anomaly Sentinels v1
Standards: Bible §6.3, §8.12, REQ-AI-107, INV-SENTINEL-001..002, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json
import subprocess

cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP48_FILES = [
    "docs/architecture/anomaly-sentinels-v1-spec.md",
    "docs/evidence/SP-48-validation.md",
    "docs/evidence/golden-boot-sp48.log",
    "userspace/ais/src/sentinel.rs",
    "kernel/src/userspace/sentinel_v1.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp48():
    print("==================================================")
    print(" Ind OS SP-48 Anomaly Sentinels v1 Verifier       ")
    print(" Target: Sprint SP-48 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-48 deliverable presence...")
    for rel_path in REQUIRED_SP48_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-48 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ais/src/sentinel.rs", "r") as f:
        s_src = f.read()
    for s in ["AnomalyType", "AnomalyAlert", "AnomalySentinelV1", "inspect_permission_rate", "inspect_storage_rate", "DEFAULT_BASELINE_THRESHOLD_FACTOR", "MAX_SENTINEL_ALERTS"]:
        if s not in s_src:
            print(f"❌ FAIL: sentinel.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Anomaly Sentinel symbols verified (REQ-AI-107, INV-SENTINEL-001, INV-SENTINEL-002)")

    # 3. Kernel Golden Boot Log Verification (Stages 187-191)
    print("\n[CHECK 3/6] Verifying kernel boot stages in golden-boot-sp48.log...")
    with open("docs/evidence/golden-boot-sp48.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        ("STAGE:187_SENTINEL_V1_INIT", "Anomaly sentinel engine v1 initialized"),
        ("STAGE:188_SENTINEL_V1_BASELINE_TRACKING", "Statistical permission & storage baseline tracking verified"),
        ("STAGE:189_SENTINEL_V1_ANOMALY_DETECTION", "Rapid permission spike & excessive access detection verified"),
        ("STAGE:190_SENTINEL_V1_TRUSTED_UI_ALARM", "Out-of-band alarm dispatch to trusted UI only verified"),
        ("STAGE:191_SP48_SENTINEL_READY", "Sprint SP-48 Anomaly Sentinels v1 Milestone REACHED"),
    ]

    for marker, desc in expected_stages:
        if marker not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{marker}' missing from golden boot log")
            sys.exit(1)
        print(f"  ✓ Boot stage {marker} verified ({desc})")

    if "[PANIC]" in boot_log or "FATAL" in boot_log:
        print("❌ FAIL: Kernel panic or fatal error detected in golden-boot-sp48.log")
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
        "userspace/ais/src/sentinel.rs",
        "kernel/src/userspace/sentinel_v1.rs",
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
    manifest_dir = "evidence/SP-48"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_file = os.path.join(manifest_dir, "manifest.json")
    manifest_data = {
        "sprint": "SP-48",
        "target": "Anomaly Sentinels v1",
        "status": "PASSED",
        "invariants": {
            "INV-SENTINEL-001": "Statistical Permission & Storage Baselines (REQ-AI-107)",
            "INV-SENTINEL-002": "Trusted UI Alert Path Only",
        },
        "stages_verified": [m[0] for m in expected_stages],
        "checksums": {f: hash_file(f) for f in REQUIRED_SP48_FILES if os.path.exists(f)},
    }
    with open(manifest_file, "w") as f:
        json.dump(manifest_data, f, indent=2)

    print(f"\n==================================================")
    print(f"  ALL SPRINT SP-48 VERIFICATION CHECKS PASSED     ")
    print(f"  Manifest written to {manifest_file}            ")
    print(f"==================================================")

if __name__ == "__main__":
    verify_sp48()
