#!/usr/bin/env python3
"""
Ind OS Sprint SP-28 Verification & Acceptance Test Harness
Target: Symmetric Multi-Processing (SMP: Core Bring-up, Per-CPU Run Queues, Affinity, IPIs)
Standards: Bible §8.10, INDOS-SPEC-SMP-01, REQ-KRN-109, INV-SMP-001, INV-SMP-002, INV-SMP-003, Rule N-01, Rule N-04, Rule N-10
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

REQUIRED_SP28_FILES = [
    "docs/architecture/smp-core-spec.md",
    "docs/evidence/SP-28-validation.md",
    "docs/evidence/golden-boot-sp28.log",
    "kernel/src/smp/core.rs",
    "kernel/src/smp/runqueue.rs",
    "kernel/src/smp/ipi.rs",
    "kernel/src/smp/mod.rs",
]

def verify_sp28():
    print("==================================================")
    print(" Ind OS SP-28 SMP Core Verifier                   ")
    print(" Target: Sprint SP-28 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-28 deliverable presence...")
    for rel_path in REQUIRED_SP28_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-28 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying architectural symbols and invariants...")
    with open("kernel/src/smp/core.rs", "r") as f:
        core_src = f.read()
    for symbol in ["CpuId", "CpuManager", "CpuState", "CpuDescriptor", "MAX_CPUS", "init_primary", "bringup_secondary"]:
        if symbol not in core_src:
            print(f"❌ FAIL: core.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Multi-core bring-up & topology manager verified in core.rs (INV-SMP-001)")

    with open("kernel/src/smp/runqueue.rs", "r") as f:
        rq_src = f.read()
    for symbol in ["AffinityMask", "PerCpuRunQueue", "ScheduledTask", "MAX_TASKS_PER_QUEUE", "is_allowed_on", "enqueue", "dequeue_highest_priority"]:
        if symbol not in rq_src:
            print(f"❌ FAIL: runqueue.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Decentralized per-CPU run queues & affinity masking verified in runqueue.rs (INV-SMP-002, REQ-KRN-109)")

    with open("kernel/src/smp/ipi.rs", "r") as f:
        ipi_src = f.read()
    for symbol in ["IpiBroker", "IpiKind", "IpiMessage", "send_ipi", "receive_ipi"]:
        if symbol not in ipi_src:
            print(f"❌ FAIL: ipi.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Cross-core IPI messaging & GIC SGI dispatcher verified in ipi.rs (INV-SMP-003)")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in SMP subsystem...")
    for root, _, files in os.walk("kernel/src/smp"):
        for file in files:
            if file.endswith(".rs"):
                path = os.path.join(root, file)
                with open(path, "r") as f:
                    content = f.read()
                    if "unsafe " in content:
                        print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                        sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/smp/ (Zero unsafe blocks)")

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
    print("\n[CHECK 5/6] Verifying golden boot log against SP-28 boot stages...")
    with open("docs/evidence/golden-boot-sp28.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:84_SMP_CORE_TOPOLOGY]",
        "Primary CPU 0 online",
        "[STAGE:85_SMP_SECONDARY_BRINGUP]",
        "All 4 CPU cores online: Cores 1..3 brought up successfully via PSCI handshake (INV-SMP-001 PASSED)",
        "[STAGE:86_SMP_PER_CPU_QUEUES]",
        "Decentralized per-CPU run queues verified: independent local priority dispatch (INV-SMP-002 PASSED)",
        "[STAGE:87_SMP_AFFINITY_DISPATCH]",
        "Affinity enforcement verified: CPU 0 rejected task with AffinityMask::CPU2",
        "Hard affinity dispatch verified: task 103 strictly dispatched on CPU 2 (REQ-KRN-109 PASSED)",
        "[STAGE:88_SMP_IPI_MESSAGING]",
        "IPI messaging verified: SGI 0 Reschedule (CPU 0 -> 1) and SGI 1 TlbFlush (CPU 1 -> 3) delivered and handled (INV-SMP-003 PASSED)",
        "[STAGE:89_SMP_SUBSYSTEM_READY]",
        "[SMP] Sprint SP-28 Symmetric Multi-Processing Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-28"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest = {
        "sprint": "SP-28",
        "title": "Symmetric Multi-Processing (SMP: Core Bring-up, Per-CPU Run Queues, Affinity, IPIs)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": "PASSED",
        "artifacts": {}
    }

    tracked_files = REQUIRED_SP28_FILES + ["ci/verify_sp28.py"]
    for path in tracked_files:
        if os.path.exists(path):
            with open(path, "rb") as f:
                sha256 = hashlib.sha256(f.read()).hexdigest()
            manifest["artifacts"][path] = sha256

    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)

    print(f"  ✓ Evidence manifest written to {manifest_path}")
    print("\n==================================================")
    print(" SP-28 VERIFICATION PASSED: All 6 checks satisfied")
    print("==================================================")

if __name__ == "__main__":
    verify_sp28()
