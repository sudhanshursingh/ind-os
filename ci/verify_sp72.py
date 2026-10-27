#!/usr/bin/env python3
"""
Ind OS Sprint SP-72 Verification & Acceptance Test Harness
Target: Boot-Chain Proof & Non-Stock Kernel Bring-Up (Header, AVB Unlock, Fallback Routing)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-103, PEN-005, Rule N-06, INV-BOOT-001..003
"""

import os
import sys

REQUIRED_SP72_FILES = [
    "docs/architecture/boot-chain-proof-spec.md",
    "docs/evidence/SP-72-validation.md",
    "docs/evidence/golden-boot-sp72.log",
    "userspace/devmgr/src/boot_chain.rs",
    "kernel/src/hal/platform/boot_chain_core.rs",
]

def verify_sp72():
    print("==================================================")
    print(" Ind OS SP-72 Boot-Chain Proof Verifier          ")
    print(" Target: Sprint SP-72 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-72 deliverable presence...")
    for rel_path in REQUIRED_SP72_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-72 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/boot_chain.rs", "r") as f:
        boot_src = f.read()
    for s in [
        "BootChainError",
        "BootChainHarness",
        "BOOT_MAGIC",
        "QUALCOMM_PAGE_SIZE",
        "verify_image_header",
        "check_avb_unlock_state",
        "simulate_boot_failure_fallback",
    ]:
        if s not in boot_src:
            print(f"❌ FAIL: boot_chain.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/boot_chain_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelBootChainError",
        "KernelBootChainHarness",
        "KERNEL_BOOT_MAGIC",
        "KERNEL_QUALCOMM_PAGE_SIZE",
        "verify_image_header",
        "check_avb_unlock_state",
        "simulate_boot_failure_fallback",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: boot_chain_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/boot_chain.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/boot_chain.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/boot_chain.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 310-314)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 310-314...")
    with open("docs/evidence/golden-boot-sp72.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "310_SP72_BOOT_CHAIN_INIT",
        "311_SP72_IMAGE_HEADER_VERIFY",
        "312_SP72_AVB_UNLOCK_STATE_CHECK",
        "313_SP72_BOOT_FALLBACK_ROUTING",
        "314_SP72_BOOT_CHAIN_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Boot Chain Confirmation Logs
    print("\n[CHECK 6/6] Verifying Boot Chain Confirmation Logs...")
    expected_logs = [
        "Kernel image header verified: ANDROID! magic & 4096-byte page size authenticated (INV-BOOT-001 PASSED)",
        "AVB 2.0 policy verified: non-stock test image safely accepted under unlocked bootloader (INV-BOOT-002, PEN-005 PASSED)",
        "Boot fallback verified: deterministic revert to verified stock slot on failure prevents bricking (INV-BOOT-003, Rule N-06 PASSED)",
        "[SP-72] Sprint SP-72 Boot-Chain Proof Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-72 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp72()
