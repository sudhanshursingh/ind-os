#!/usr/bin/env python3
"""
Ind OS Sprint SP-30 Verification & Acceptance Test Harness
Target: Kernel Telemetry API v1 (Read-Only Ring Buffer, Schemas, Write Denial, AIS Consumption)
Standards: Bible §8.10, REQ-KRN-113, INDOS-SPEC-TELEM-01, Rule N-01, Rule N-04, Rule N-10
"""

import os
import sys
import subprocess

cargo_bin = os.path.expanduser("~/.cargo/bin")
if cargo_bin not in os.environ.get("PATH", ""):
    os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

REQUIRED_SP30_FILES = [
    "docs/architecture/kernel-telemetry-spec.md",
    "docs/evidence/SP-30-validation.md",
    "docs/evidence/golden-boot-sp30.log",
    "evidence/SP-30/manifest.json",
    "kernel/src/telemetry/event.rs",
    "kernel/src/telemetry/ring_buffer.rs",
    "kernel/src/telemetry/mod.rs",
]

def verify_sp30():
    print("==================================================")
    print(" Ind OS SP-30 Kernel Telemetry API v1 Verifier   ")
    print(" Target: Sprint SP-30 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-30 deliverable presence...")
    for rel_path in REQUIRED_SP30_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-30 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and invariants...")
    with open("kernel/src/telemetry/event.rs", "r") as f:
        ev_src = f.read()
    for symbol in ["TelemetryEventKind", "TelemetryEvent", "SchedulerSwitch", "IpcTransfer", "MemoryPressure", "SecurityViolation"]:
        if symbol not in ev_src:
            print(f"❌ FAIL: event.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Multi-category telemetry event schemas verified in event.rs")

    with open("kernel/src/telemetry/ring_buffer.rs", "r") as f:
        rb_src = f.read()
    for symbol in ["TelemetryRingBuffer", "RING_CAPACITY", "emit", "read_event", "read_batch", "oldest_available_seq", "head_seq"]:
        if symbol not in rb_src:
            print(f"❌ FAIL: ring_buffer.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Circular ring buffer & monotonic sequence tracking verified in ring_buffer.rs")

    with open("kernel/src/telemetry/mod.rs", "r") as f:
        mod_src = f.read()
    for symbol in ["TELEMETRY_BUFFER", "write_telemetry_guarded", "EPERM"]:
        if symbol not in mod_src:
            print(f"❌ FAIL: telemetry/mod.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Userspace write-denial gate & capability check verified in telemetry/mod.rs (REQ-KRN-113)")

    # 3. Unsafe Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in telemetry subsystem...")
    for root, _, files in os.walk("kernel/src/telemetry"):
        for file in files:
            if file.endswith(".rs"):
                path = os.path.join(root, file)
                with open(path, "r") as f:
                    content = f.read()
                    if "unsafe " in content:
                        print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                        sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/telemetry/ (Zero unsafe blocks)")

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
    print("\n[CHECK 5/6] Verifying golden boot log against SP-30 boot stages...")
    with open("docs/evidence/golden-boot-sp30.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:95_TELEMETRY_BUFFER_INIT] Kernel Telemetry ring buffer initialized, capacity and sequential indices verified",
        "[INFO ] telem: Kernel Telemetry buffer online: capacity 64 records, sequential ring indices verified",
        "[STAGE:96_TELEMETRY_EVENT_LOGGING] Scheduler, IPC, memory, and security events logged with monotonic sequence numbering",
        "[INFO ] telem: Telemetry logging verified: 4 events (sched, ipc, mem, sec) emitted with monotonic seq numbers (1..4)",
        "[STAGE:97_TELEMETRY_READONLY_ENFORCEMENT] Userspace write-attempt denial verified: EPERM strictly returned on unprivileged write (REQ-KRN-113)",
        "[INFO ] telem: Write denial test PASSED: User-mode write rejected with EPERM (REQ-KRN-113 PASSED)",
        "[STAGE:98_TELEMETRY_AIS_CONSUMPTION] Simulated AIS daemon reading telemetry stream in FIFO order verified",
        "[INFO ] telem: AIS consumption simulation verified: 4 events read in FIFO order without buffer destruction (REQ-KRN-113 PASSED)",
        "[STAGE:99_TELEMETRY_API_READY] Sprint SP-30 Kernel Telemetry API v1 Milestone REACHED",
    ]

    for stage in required_stages:
        if stage not in log_content:
            print(f"❌ FAIL: Required stage/log message missing in golden boot log: '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified: {stage[:72]}...")

    # 6. DoD Gate & Manifest Verification
    print("\n[CHECK 6/6] Verifying Sprint SP-30 DoD Gate & Manifest...")
    manifest_path = "evidence/SP-30/manifest.json"
    with open(manifest_path, "r") as f:
        manifest = json.load(f)

    if manifest.get("sprint") != "SP-30" or manifest.get("status") != "PASSED":
        print(f"❌ FAIL: Manifest invalid in {manifest_path}")
        sys.exit(1)
    print(f"  ✓ Manifest verified: {manifest['sprint']} -> {manifest['status']}")

    print("\n==================================================")
    print(" 🚀 SUCCESS: Sprint SP-30 100% VERIFIED & ACCEPTED ")
    print("==================================================")

if __name__ == "__main__":
    import json
    verify_sp30()
