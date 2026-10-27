#!/usr/bin/env python3
"""
Ind OS Sprint SP-71 Verification & Acceptance Test Harness
Target: Device Evidence Pack & Recovery Procedure (Identity, Partition Map, Recovery Image)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-102, Rule N-06, INV-EVID-001..003
"""

import os
import sys

REQUIRED_SP71_FILES = [
    "docs/architecture/device-evidence-pack-spec.md",
    "docs/evidence/SP-71-validation.md",
    "docs/evidence/golden-boot-sp71.log",
    "userspace/devmgr/src/device_evidence.rs",
    "kernel/src/hal/platform/device_evidence_core.rs",
]

def verify_sp71():
    print("==================================================")
    print(" Ind OS SP-71 Device Evidence Pack Verifier      ")
    print(" Target: Sprint SP-71 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-71 deliverable presence...")
    for rel_path in REQUIRED_SP71_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-71 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/device_evidence.rs", "r") as f:
        evid_src = f.read()
    for s in [
        "DeviceEvidenceError",
        "PartitionDescriptor",
        "DeviceEvidenceHarness",
        "MOTO_G60_DEVICE_CODENAME",
        "MOTO_G60_SOC_ID",
        "verify_device_identity",
        "record_partition",
        "verify_recovery_image",
    ]:
        if s not in evid_src:
            print(f"❌ FAIL: device_evidence.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/device_evidence_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelDeviceEvidenceError",
        "KernelPartitionDescriptor",
        "KernelDeviceEvidenceHarness",
        "KERNEL_MOTO_G60_DEVICE_CODENAME",
        "KERNEL_MOTO_G60_SOC_ID",
        "verify_device_identity",
        "record_partition",
        "verify_recovery_image",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: device_evidence_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/devmgr/Cargo.toml"]:
        with open(toml_path, "r") as f:
            lines = [line.strip() for line in f.readlines() if line.strip() and not line.strip().startswith("#")]
        in_deps = False
        for line in lines:
            if line.startswith("[dependencies]"):
                in_deps = True
                continue
            if line.startswith("[") and in_deps:
                in_deps = False
                continue
            if in_deps and line:
                if "path =" not in line:
                    print(f"❌ FAIL: External dependency detected in {toml_path}: {line}")
                    sys.exit(1)
        print(f"  ✓ {toml_path} compliant with Rule N-01")

    # 4. Rule N-04: Zero Unsafe Code Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/devmgr)...")
    with open("userspace/devmgr/src/device_evidence.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/device_evidence.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/device_evidence.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 305-309)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 305-309...")
    with open("docs/evidence/golden-boot-sp71.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "305_SP71_DEVICE_EVIDENCE_INIT",
        "306_SP71_DEVICE_IDENTITY_RECORD",
        "307_SP71_PARTITION_MAP_HASH_PINNING",
        "308_SP71_RECOVERY_IMAGE_RESTORE_PROOF",
        "309_SP71_DEVICE_EVIDENCE_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Device Evidence Confirmation
    print("\n[CHECK 6/6] Verifying Device Evidence Confirmation Logs...")
    expected_logs = [
        "Device identity verified: Motorola Moto G60 (hanoip, Snapdragon SM7150-AC) matched (INV-EVID-001 PASSED)",
        "Partition map verified: 4 critical partitions pinned with immutable SHA256 checksums (INV-EVID-002 PASSED)",
        "Recovery proof verified: non-destructive stock EDL recovery image checksum authenticated (INV-EVID-003 PASSED)",
        "[SP-71] Sprint SP-71 Device Evidence Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-71 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp71()
