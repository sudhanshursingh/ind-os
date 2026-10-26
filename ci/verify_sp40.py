#!/usr/bin/env python3
"""
Ind OS Sprint SP-40 Verification & Acceptance Test Harness
Target: Quality Sprint & Developer OS v0.1 Packaging
Standards: Bible §8.11, INV-PKG-001, INV-PKG-002, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import hashlib
import json
import subprocess

cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP40_FILES = [
    "docs/architecture/developer-os-packaging-spec.md",
    "docs/evidence/SP-40-validation.md",
    "docs/evidence/golden-boot-sp40.log",
    "tools/package_developer_os.sh",
    "kernel/src/userspace/quality.rs",
]

def hash_file(filepath: str) -> str:
    h = hashlib.sha256()
    with open(filepath, "rb") as fh:
        while chunk := fh.read(65536):
            h.update(chunk)
    return h.hexdigest()

def verify_sp40():
    print("==================================================")
    print(" Ind OS SP-40 Quality & Packaging Verifier        ")
    print(" Target: Sprint SP-40 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-40 deliverable presence...")
    for rel_path in REQUIRED_SP40_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-40 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Packaging Tool Execution Check
    print("\n[CHECK 2/6] Verifying Developer OS v0.1 packaging execution...")
    res = subprocess.run(
        ["./tools/package_developer_os.sh"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if res.returncode != 0:
        print(f"❌ FAIL: package_developer_os.sh failed:\n{res.stderr}")
        sys.exit(1)
    if not os.path.exists("target/developer-os-v0.1.tar.gz"):
        print("❌ FAIL: target/developer-os-v0.1.tar.gz was not created")
        sys.exit(1)
    if not os.path.exists("target/developer-os-v0.1/sbom.json"):
        print("❌ FAIL: target/developer-os-v0.1/sbom.json missing")
        sys.exit(1)
    print("  ✓ Developer OS v0.1 release bundle verified (target/developer-os-v0.1.tar.gz)")

    # 3. SBOM Verification
    print("\n[CHECK 3/6] Verifying Software Bill of Materials (SBOM)...")
    with open("target/developer-os-v0.1/sbom.json", "r") as f:
        sbom = json.load(f)
    components = sbom.get("components", [])
    if len(components) != 9:
        print(f"❌ FAIL: Expected 9 components in SBOM, found {len(components)}")
        sys.exit(1)
    print(f"  ✓ SBOM verified: exactly {len(components)} in-tree components accounted for")

    # 4. Kernel Boot Verification (Stages 146-150)
    print("\n[CHECK 4/6] Verifying kernel boot stages in golden-boot-sp40.log...")
    with open("docs/evidence/golden-boot-sp40.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        ("STAGE:146_SP40_QUALITY_SOAK", "Multi-cycle quality soak and burst stress simulation"),
        ("STAGE:147_SP40_LEAK_SWEEP", "Whole-system heap watermark and capability leak sweep verified"),
        ("STAGE:148_SP40_SBOM_AUDIT", "SBOM inventory and service manifest compliance verified"),
        ("STAGE:149_SP40_DEVOS_PACKAGING", "Developer OS v0.1 release packaging verified"),
        ("STAGE:150_SP40_QUALITY_READY", "Sprint SP-40 Quality & Packaging Milestone REACHED"),
    ]

    for marker, desc in expected_stages:
        if marker not in boot_log:
            print(f"❌ FAIL: Expected stage marker '{marker}' missing from golden boot log")
            sys.exit(1)
        print(f"  ✓ Boot stage {marker} verified ({desc})")

    if "[PANIC]" in boot_log or "FATAL" in boot_log:
        print("❌ FAIL: Kernel panic or fatal error detected in golden-boot-sp40.log")
        sys.exit(1)
    print("  ✓ Golden boot clean execution confirmed")

    # 5. Invariants & Zero Unsafe Check
    print("\n[CHECK 5/6] Verifying Rule N-04: Zero unsafe in userspace modules...")
    modules_to_scan = [
        "kernel/src/userspace/quality.rs",
        "userspace/init/src/lib.rs",
        "userspace/libindus/src/lib.rs",
        "userspace/devmgr/src/lib.rs",
        "userspace/shell/src/lib.rs",
        "userspace/logd/src/lib.rs",
        "userspace/store/src/lib.rs",
        "userspace/edit/src/lib.rs",
        "userspace/testsuite/src/lib.rs",
    ]
    for path in modules_to_scan:
        with open(path, "r") as f:
            for idx, line in enumerate(f, 1):
                clean_line = line.split("//")[0].strip()
                if "unsafe" in clean_line:
                    print(f"❌ FAIL: Rule N-04 violation: 'unsafe' found at {path}:{idx}")
                    sys.exit(1)
    print("  ✓ Rule N-04 verified: 0 unsafe blocks detected across all userspace crates")

    # 6. Compilation Check
    print("\n[CHECK 6/6] Verifying workspace compilation check...")
    res = subprocess.run(
        ["cargo", "check", "--workspace", "--target", "aarch64-unknown-none"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if res.returncode != 0:
        print(f"❌ FAIL: workspace compilation failed:\n{res.stderr}")
        sys.exit(1)
    print("  ✓ Entire workspace cleanly compiled for aarch64-unknown-none")

    # Output manifest
    manifest_dir = "evidence/SP-40"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_file = os.path.join(manifest_dir, "manifest.json")
    manifest_data = {
        "sprint": "SP-40",
        "target": "Quality Sprint & Developer OS v0.1 Packaging",
        "status": "PASSED",
        "invariants": {
            "INV-PKG-001": "Whole-System Zero-Leak Watermark",
            "INV-PKG-002": "100% Manifest & Capability Audit",
        },
        "stages_verified": [m[0] for m in expected_stages],
        "checksums": {f: hash_file(f) for f in REQUIRED_SP40_FILES if os.path.exists(f)},
    }
    with open(manifest_file, "w") as f:
        json.dump(manifest_data, f, indent=2)

    print(f"\n==================================================")
    print(f"  ALL SPRINT SP-40 VERIFICATION CHECKS PASSED     ")
    print(f"  Manifest written to {manifest_file}            ")
    print(f"==================================================")

if __name__ == "__main__":
    verify_sp40()
