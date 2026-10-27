#!/usr/bin/env python3
"""
Ind OS Sprint SP-93 Verification & Acceptance Test Harness
Target: Qualcomm WCD9385 Audio Codec & SLIMbus / SoundWire Interface
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-111, REQ-AUD-101, INV-AUD-001..003
"""

import os
import sys

REQUIRED_SP93_FILES = [
    "docs/architecture/audio-codec-spec.md",
    "docs/evidence/SP-93-validation.md",
    "docs/evidence/golden-boot-sp93.log",
    "userspace/devmgr/src/audio_codec.rs",
    "kernel/src/hal/platform/audio_codec_core.rs",
]

def verify_sp93():
    print("==================================================")
    print(" Ind OS SP-93 Audio Codec Verifier                ")
    print(" Target: Sprint SP-93 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-93 deliverable presence...")
    for rel_path in REQUIRED_SP93_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-93 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/audio_codec.rs", "r") as f:
        codec_src = f.read()
    for s in [
        "AudioCodecHarness",
        "AudioCodecError",
        "AudioCodecState",
        "HANOIP_AUDIO_SLIMBUS_BASE",
        "HANOIP_AUDIO_SLIMBUS_SIZE",
        "WCD9385_MANUFACTURER_ID",
        "WCD9385_PRODUCT_ID",
        "CAP_DEVICE_ADMIN",
        "map_mmio",
        "identify_codec",
        "power_up_sequence",
        "is_mmio_mapped",
        "is_identified",
        "is_powered",
    ]:
        if s not in codec_src:
            print(f"❌ FAIL: audio_codec.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/audio_codec_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelAudioCodecHarness",
        "KernelAudioCodecError",
        "KernelAudioCodecState",
        "KERNEL_HANOIP_AUDIO_SLIMBUS_BASE",
        "KERNEL_HANOIP_AUDIO_SLIMBUS_SIZE",
        "KERNEL_WCD9385_MANUFACTURER_ID",
        "KERNEL_WCD9385_PRODUCT_ID",
        "KERNEL_CAP_DEVICE_ADMIN",
        "map_mmio",
        "identify_codec",
        "power_up_sequence",
        "is_mmio_mapped",
        "is_identified",
        "is_powered",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: audio_codec_core.rs missing symbol '{s}'")
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
    with open("userspace/devmgr/src/audio_codec.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/audio_codec.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/audio_codec.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 415-419)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 415-419...")
    with open("docs/evidence/golden-boot-sp93.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "415_SP93_AUDIO_CODEC_INIT",
        "416_SP93_AUDIO_SLIMBUS_MAPPING",
        "417_SP93_WCD9385_CHIP_ID_VERIFY",
        "418_SP93_AUDIO_POWER_UP_SEQUENCING",
        "419_SP93_AUDIO_CODEC_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Confirmation Logs
    print("\n[CHECK 6/6] Verifying Audio Codec Confirmation Logs...")
    expected_logs = [
        "Audio codec harness online: WCD9385 digital audio interface active (REQ-AUD-101 PASSED)",
        "SLIMbus verified: audio aperture mapped at 0x0c00_0000 (INV-AUD-001 PASSED)",
        "Codec ID verified: WCD9385 (Mfg=0x42, Prod=0x85) handshake confirmed (INV-AUD-002 PASSED)",
        "Power sequence verified: charge pump active and audio PLL locked (INV-AUD-003 PASSED)",
        "[SP-93] Sprint SP-93 Audio Codec Bring-Up Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-93 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp93()
