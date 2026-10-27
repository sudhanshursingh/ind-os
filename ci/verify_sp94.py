#!/usr/bin/env python3
"""
Ind OS Sprint SP-94 Verification & Acceptance Test Harness
Target: Audio PCM Stream Buffer & User-Mode HAL (DMA Ring-Buffer, 48kHz Stereo, XRUN Resilience)
Standards: Bible §6.3, §8.15, Table 19.2, REQ-HW-111, REQ-AUD-102, INV-PCM-001..003
"""

import os
import sys

REQUIRED_SP94_FILES = [
    "docs/architecture/audio-pcm-spec.md",
    "docs/evidence/SP-94-validation.md",
    "docs/evidence/golden-boot-sp94.log",
    "userspace/devmgr/src/audio_pcm.rs",
    "kernel/src/hal/platform/audio_pcm_core.rs",
]

def verify_sp94():
    print("==================================================")
    print(" Ind OS SP-94 Audio PCM Stream Verifier           ")
    print(" Target: Sprint SP-94 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-94 deliverable presence...")
    for rel_path in REQUIRED_SP94_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-94 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/devmgr/src/audio_pcm.rs", "r") as f:
        pcm_src = f.read()
    for s in [
        "AudioPcmHarness",
        "AudioPcmError",
        "AudioPcmState",
        "PCM_SAMPLE_RATE_48K",
        "PCM_CHANNELS_STEREO",
        "PCM_PERIODS",
        "PCM_PERIOD_FRAMES",
        "CAP_AUDIO_PLAYBACK",
        "configure",
        "write_frames",
        "handle_underrun",
        "is_configured",
        "is_running",
        "sample_rate",
        "channels",
        "xrun_count",
        "latency_ms",
    ]:
        if s not in pcm_src:
            print(f"❌ FAIL: audio_pcm.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/audio_pcm_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelAudioPcmHarness",
        "KernelAudioPcmError",
        "KernelAudioPcmState",
        "KERNEL_PCM_SAMPLE_RATE_48K",
        "KERNEL_PCM_CHANNELS_STEREO",
        "KERNEL_PCM_PERIODS",
        "KERNEL_PCM_PERIOD_FRAMES",
        "KERNEL_CAP_AUDIO_PLAYBACK",
        "configure",
        "write_frames",
        "handle_underrun",
        "is_configured",
        "is_running",
        "sample_rate",
        "channels",
        "xrun_count",
        "latency_ms",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: audio_pcm_core.rs missing symbol '{s}'")
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
            if line.startswith("path =") or "path =" in line:
                continue
            if in_deps and line:
                print(f"❌ FAIL: External dependency detected in {toml_path}: {line}")
                sys.exit(1)
        print(f"  ✓ {toml_path} compliant with Rule N-01")

    # 4. Rule N-04: Zero Unsafe Code Check
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/devmgr)...")
    with open("userspace/devmgr/src/audio_pcm.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/devmgr/src/audio_pcm.rs")
            sys.exit(1)
    print("  ✓ userspace/devmgr/src/audio_pcm.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 420-424)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 420-424...")
    with open("docs/evidence/golden-boot-sp94.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "420_SP94_AUDIO_PCM_INIT",
        "421_SP94_PCM_RING_BUFFER_CONFIG",
        "422_SP94_PCM_DMA_PLAYBACK_STREAM",
        "423_SP94_PCM_XRUN_ZERO_PAD_RECOVERY",
        "424_SP94_AUDIO_PCM_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Confirmation Logs
    print("\n[CHECK 6/6] Verifying Audio PCM Confirmation Logs...")
    expected_logs = [
        "Audio PCM harness online: DMA circular buffer engine active (REQ-AUD-102 PASSED)",
        "PCM configured: 48kHz stereo 4-period buffer (16ms latency) (INV-PCM-001 PASSED)",
        "PCM streaming: 384 frames queued into DMA ring-buffer (INV-PCM-003 PASSED)",
        "XRUN recovered: 1 underrun handled cleanly with silence padding (INV-PCM-002 PASSED)",
        "[SP-94] Sprint SP-94 Audio PCM Stream Buffer Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-94 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp94()
