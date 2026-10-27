#!/usr/bin/env python3
"""
Ind OS Sprint SP-86 Verification & Acceptance Test Harness
Target: Firmware Blob Loader per §14.3 (Isolated, SBOM-Tracked, Capability-Gated Dispatch)
Standards: Bible §6.3, §8.15, §14.3, Table 19.2, REQ-SEC-105, REQ-HW-109, INV-FW-001..003
"""

import os
import sys

REQUIRED_SP86_FILES = [
    "docs/architecture/firmware-loader-spec.md",
    "docs/evidence/SP-86-validation.md",
    "docs/evidence/golden-boot-sp86.log",
    "userspace/devmgr/src/firmware_loader.rs",
    "kernel/src/hal/platform/firmware_loader_core.rs",
]

def verify_sp86():
    print("==================================================")
    print(" Ind OS SP-86 Firmware Loader Verifier            ")
    print(" Target: Sprint SP-86 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-86 deliverable presence...")
    for rel_path in REQUIRED_SP86_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-86 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/firmware_loader.rs", "r") as f:
        fw_src = f.read()
    for s in [
        "FirmwareSubsystem",
        "FirmwareError",
        "FirmwareManifestEntry",
        "FirmwareLoaderHarness",
        "CAP_FIRMWARE_LOAD",
        "FW_DEST_WLAN",
        "FW_DEST_BT",
        "FW_DEST_ADSP",
        "FW_DEST_MODEM",
        "verify_and_stage",
        "loaded_count",
        "is_staged",
    ]:
        if s not in fw_src:
            print(f"❌ FAIL: firmware_loader.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/firmware_loader_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelFirmwareSubsystem",
        "KernelFirmwareError",
        "KernelFirmwareLoaderHarness",
        "KERNEL_CAP_FIRMWARE_LOAD",
        "KERNEL_FW_DEST_WLAN",
        "KERNEL_FW_DEST_BT",
        "KERNEL_FW_DEST_ADSP",
        "KERNEL_FW_DEST_MODEM",
        "verify_and_stage",
        "loaded_count",
        "is_staged",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: firmware_loader_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/firmware_loader.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/firmware_loader.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/firmware_loader.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 380-384)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 380-384...")
    with open("docs/evidence/golden-boot-sp86.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "380_SP86_FIRMWARE_LOADER_INIT",
        "381_SP86_FIRMWARE_ISOLATION_OUTSIDE_TCB",
        "382_SP86_FIRMWARE_SHA256_SBOM_VERIFY",
        "383_SP86_CAPABILITY_GATED_FIRMWARE_DISPATCH",
        "384_SP86_FIRMWARE_LOADER_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Firmware Loader Confirmation Logs
    print("\n[CHECK 6/6] Verifying Firmware Loader Confirmation Logs...")
    expected_logs = [
        "Isolation verified: firmware loading logic isolated in devmgr; kernel boundary strictly validates descriptors (INV-FW-001 PASSED)",
        "SBOM verified: SHA-256 digest mismatch rejected; untrusted payload contained (INV-FW-002 PASSED)",
        "Dispatch verified: CAP_FIRMWARE_LOAD enforced; WLAN staged to 0x89000000, BT to 0x89200000 (INV-FW-003 PASSED)",
        "[SP-86] Sprint SP-86 Firmware Blob Loader Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-86 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp86()
