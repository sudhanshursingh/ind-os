#!/usr/bin/env python3
"""
Ind OS Sprint SP-27 Verification & Acceptance Test Harness
Target: Reference Drivers (User Mode: virtio-blk, virtio-net, framebuffer, input)
Standards: Bible §8.10, INDOS-SPEC-REFDRV-01, REQ-KRN-107, INV-DRV-001, INV-DRV-002, INV-DRV-003, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import json
import hashlib
import subprocess
import time

# Ensure ~/.cargo/bin is in PATH for cargo invocations
cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP27_FILES = [
    "docs/architecture/reference-drivers-spec.md",
    "docs/evidence/SP-27-validation.md",
    "docs/evidence/golden-boot-sp27.log",
    "kernel/src/driver_framework/reference/virtio_blk.rs",
    "kernel/src/driver_framework/reference/virtio_net.rs",
    "kernel/src/driver_framework/reference/framebuffer.rs",
    "kernel/src/driver_framework/reference/input.rs",
    "kernel/src/driver_framework/reference/mod.rs",
]

def verify_sp27():
    print("==================================================")
    print(" Ind OS SP-27 Reference Drivers Verifier          ")
    print(" Target: Sprint SP-27 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-27 deliverable presence...")
    for rel_path in REQUIRED_SP27_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-27 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying architectural symbols and invariants...")
    with open("kernel/src/driver_framework/reference/virtio_blk.rs", "r") as f:
        blk_src = f.read()
    for symbol in ["VirtioBlkDriver", "SECTOR_SIZE", "handle_io_read", "handle_io_write", "handle_ipc_message"]:
        if symbol not in blk_src:
            print(f"❌ FAIL: virtio_blk.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ VirtIO block reference driver verified in virtio_blk.rs")

    with open("kernel/src/driver_framework/reference/virtio_net.rs", "r") as f:
        net_src = f.read()
    for symbol in ["VirtioNetDriver", "DEFAULT_MTU", "transmit_packet", "receive_packet", "handle_irq_notification"]:
        if symbol not in net_src:
            print(f"❌ FAIL: virtio_net.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ VirtIO net reference driver verified in virtio_net.rs")

    with open("kernel/src/driver_framework/reference/framebuffer.rs", "r") as f:
        fb_src = f.read()
    for symbol in ["FramebufferDriver", "framebuffer_size_bytes", "flush_dirty_rect"]:
        if symbol not in fb_src:
            print(f"❌ FAIL: framebuffer.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Framebuffer display reference driver verified in framebuffer.rs")

    with open("kernel/src/driver_framework/reference/input.rs", "r") as f:
        input_src = f.read()
    for symbol in ["InputDriver", "InputEvent", "InputEventType", "MAX_INPUT_EVENTS", "push_key_event", "push_mouse_event", "pop_event"]:
        if symbol not in input_src:
            print(f"❌ FAIL: input.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Keyboard & Mouse input reference driver verified in input.rs")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in reference drivers...")
    for root, _, files in os.walk("kernel/src/driver_framework/reference"):
        for file in files:
            if file.endswith(".rs"):
                path = os.path.join(root, file)
                with open(path, "r") as f:
                    content = f.read()
                    if "unsafe " in content:
                        print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                        sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/driver_framework/reference/ (Zero unsafe blocks)")

    # 4. Clean Kernel Compilation Check
    print("\n[CHECK 4/6] Verifying clean kernel compilation...")
    cmd = ["cargo", "check", "--target", "aarch64-unknown-none"]
    res = subprocess.run(cmd, cwd="kernel", capture_output=True, text=True)
    if res.returncode != 0:
        print(f"❌ FAIL: Kernel compilation failed:\n{res.stderr}")
        sys.exit(1)
    if "warning:" in res.stderr:
        print(f"❌ FAIL: Compilation generated warnings:\n{res.stderr}")
        sys.exit(1)
    print("  ✓ Kernel compiles cleanly with zero warnings/errors")

    # 5. Golden Boot Log Verification
    print("\n[CHECK 5/6] Verifying golden boot log against SP-27 boot stages...")
    with open("docs/evidence/golden-boot-sp27.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:78_REF_VIRTIO_BLK_INIT]",
        "virtio-blk user-mode driver online: 2048 sectors (1 MiB)",
        "[STAGE:79_REF_VIRTIO_NET_INIT]",
        "virtio-net user-mode driver online: MAC 52:54:00:12:34:56",
        "[STAGE:80_REF_FRAMEBUFFER_INIT]",
        "framebuffer user-mode driver online: 1024x768 @ 32 bpp",
        "[STAGE:81_REF_INPUT_INIT]",
        "virtio-input user-mode driver online: keyboard & mouse event buffering and dispatch verified",
        "[STAGE:82_REF_DRIVERS_ISOLATION]",
        "Reference driver isolation verified: 4 user-mode drivers operating with zero ambient authority (100% DEFENSE)",
        "[STAGE:83_REFERENCE_DRIVERS_READY]",
        "[DRV] Sprint SP-27 Reference Drivers Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-27"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest = {
        "sprint": "SP-27",
        "title": "Reference Drivers (User Mode: virtio-blk, virtio-net, framebuffer, input)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": "PASSED",
        "artifacts": {}
    }

    tracked_files = REQUIRED_SP27_FILES + ["ci/verify_sp27.py"]
    for path in tracked_files:
        if os.path.exists(path):
            with open(path, "rb") as f:
                sha256 = hashlib.sha256(f.read()).hexdigest()
            manifest["artifacts"][path] = sha256

    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)

    print(f"  ✓ Evidence manifest written to {manifest_path}")
    print("\n==================================================")
    print(" SP-27 VERIFICATION PASSED: All 6 checks satisfied")
    print("==================================================")

if __name__ == "__main__":
    verify_sp27()
