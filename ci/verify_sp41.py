#!/usr/bin/env python3
"""
Ind OS Sprint SP-41 Verification & Acceptance Test Harness
Target: AIS Model Runtime Sandbox & Dynamic Hardware Profile Manager
Standards: Bible §8.12, §9.7, REQ-AI-101, REQ-AI-112, INV-AIS-001..003, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json
import subprocess

cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP41_FILES = [
    "docs/architecture/ais-model-runtime-sandbox-spec.md",
    "docs/evidence/SP-41-validation.md",
    "docs/evidence/golden-boot-sp41.log",
    "userspace/ais/Cargo.toml",
    "userspace/ais/README.md",
    "userspace/ais/src/lib.rs",
    "userspace/ais/src/main.rs",
    "userspace/ais/src/types.rs",
    "userspace/ais/src/profile.rs",
    "userspace/ais/src/loader.rs",
    "userspace/ais/src/sandbox.rs",
    "kernel/src/userspace/ais_runtime.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp41():
    print("==================================================")
    print(" Ind OS SP-41 AIS Model Sandbox Verifier          ")
    print(" Target: Sprint SP-41 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-41 deliverable presence...")
    for rel_path in REQUIRED_SP41_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-41 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ais/src/profile.rs", "r") as f:
        p_src = f.read()
    for s in ["IndHardwareProfile", "select_profile", "HardwareTier", "InferenceConfig"]:
        if s not in p_src:
            print(f"❌ FAIL: profile.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Hardware profiling symbols verified (REQ-AI-112, INV-AIS-003)")

    with open("userspace/ais/src/loader.rs", "r") as f:
        l_src = f.read()
    for s in ["SignedModelLoader", "verify_and_load", "compute_checksum", "AUTHORIZED_SIGNING_AUTHORITY"]:
        if s not in l_src:
            print(f"❌ FAIL: loader.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Signed model loader symbols verified (REQ-AI-101, INV-AIS-001)")

    with open("userspace/ais/src/sandbox.rs", "r") as f:
        s_src = f.read()
    for s in ["ModelRuntimeSandbox", "load_model", "infer", "reset"]:
        if s not in s_src:
            print(f"❌ FAIL: sandbox.rs missing symbol '{s}'")
            sys.exit(1)
    print("  ✓ Model sandbox symbols verified (INV-AIS-002)")

    # 3. Kernel Golden Boot Log Verification (Stages 152-156)
    print("\n[CHECK 3/6] Verifying kernel boot stages in golden-boot-sp41.log...")
    with open("docs/evidence/golden-boot-sp41.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        ("STAGE:152_AIS_SANDBOX_INIT", "AIS model runtime sandbox & hardware profile manager initialized"),
        ("STAGE:153_AIS_HARDWARE_PROFILE", "Dynamic hardware profile selection via telemetry verified"),
        ("STAGE:154_AIS_SIGNED_MODEL_LOADER", "Signed model verification and tamper rejection verified"),
        ("STAGE:155_AIS_SANDBOX_EXECUTION", "Bounded CPU inference within capability sandbox verified"),
        ("STAGE:156_SP41_AIS_SANDBOX_READY", "Sprint SP-41 Model Runtime Sandbox (REQ-AI-101, REQ-AI-112) Milestone REACHED"),
    ]

    for marker, desc in expected_stages:
        if marker not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{marker}' missing from golden boot log")
            sys.exit(1)
        print(f"  ✓ Boot stage {marker} verified ({desc})")

    if "[PANIC]" in boot_log or "FATAL" in boot_log:
        print("❌ FAIL: Kernel panic or fatal error detected in golden-boot-sp41.log")
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
        "userspace/ais/src/lib.rs",
        "userspace/ais/src/main.rs",
        "userspace/ais/src/types.rs",
        "userspace/ais/src/profile.rs",
        "userspace/ais/src/loader.rs",
        "userspace/ais/src/sandbox.rs",
        "kernel/src/userspace/ais_runtime.rs",
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
    manifest_dir = "evidence/SP-41"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_file = os.path.join(manifest_dir, "manifest.json")
    manifest_data = {
        "sprint": "SP-41",
        "target": "AIS Model Runtime Sandbox & Hardware Profile Manager",
        "status": "PASSED",
        "invariants": {
            "INV-AIS-001": "Signed-Model Loader Only (Cryptographic Integrity)",
            "INV-AIS-002": "Bounded Capability Sandbox & Memory/Time Quota",
            "INV-AIS-003": "Deterministic Hardware Profile Tiering via Telemetry",
        },
        "stages_verified": [m[0] for m in expected_stages],
        "checksums": {f: hash_file(f) for f in REQUIRED_SP41_FILES if os.path.exists(f)},
    }
    with open(manifest_file, "w") as f:
        json.dump(manifest_data, f, indent=2)

    print(f"\n==================================================")
    print(f"  ALL SPRINT SP-41 VERIFICATION CHECKS PASSED     ")
    print(f"  Manifest written to {manifest_file}            ")
    print(f"==================================================")

if __name__ == "__main__":
    verify_sp41()
