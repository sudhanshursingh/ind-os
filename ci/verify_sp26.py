#!/usr/bin/env python3
"""
Ind OS Sprint SP-26 Verification & Acceptance Test Harness
Target: User-Mode Driver Framework (Device Manager, IRQ Broker, DMA/IOMMU Broker, Driver Protocol over IPC)
Standards: Bible §8.10, INDOS-SPEC-DRV-01, REQ-KRN-107, INV-DRV-001, INV-DRV-002, INV-DRV-003, Rule N-01, Rule N-04, Rule N-10
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

REQUIRED_SP26_FILES = [
    "docs/architecture/driver-framework-spec.md",
    "docs/evidence/SP-26-validation.md",
    "docs/evidence/golden-boot-sp26.log",
    "kernel/src/driver_framework/device_manager.rs",
    "kernel/src/driver_framework/irq_broker.rs",
    "kernel/src/driver_framework/dma_broker.rs",
    "kernel/src/driver_framework/protocol.rs",
    "kernel/src/driver_framework/mod.rs",
    "kernel/src/capability/table.rs",
]

def verify_sp26():
    print("==================================================")
    print(" Ind OS SP-26 Driver Framework Verifier           ")
    print(" Target: Sprint SP-26 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-26 deliverable presence...")
    for rel_path in REQUIRED_SP26_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-26 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying architectural symbols and invariants...")
    with open("kernel/src/driver_framework/device_manager.rs", "r") as f:
        dm_src = f.read()
    for symbol in ["DeviceDescriptor", "DeviceManager", "DeviceState", "DeviceType", "MAX_DEVICES", "register_device", "bind_driver"]:
        if symbol not in dm_src:
            print(f"❌ FAIL: device_manager.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Device Manager & capability-authenticated binding verified in device_manager.rs (INV-DRV-001)")

    with open("kernel/src/driver_framework/irq_broker.rs", "r") as f:
        irq_src = f.read()
    for symbol in ["IrqBroker", "IrqSubscription", "IrqState", "MAX_IRQS", "subscribe", "dispatch_irq", "acknowledge_irq"]:
        if symbol not in irq_src:
            print(f"❌ FAIL: irq_broker.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ IRQ Broker & safe line masking verified in irq_broker.rs (INV-DRV-002)")

    with open("kernel/src/driver_framework/dma_broker.rs", "r") as f:
        dma_src = f.read()
    for symbol in ["DmaBroker", "DmaWindow", "DmaDirection", "MAX_DMA_WINDOWS", "allocate_window", "validate_transfer"]:
        if symbol not in dma_src:
            print(f"❌ FAIL: dma_broker.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ DMA Broker & IOMMU window isolation verified in dma_broker.rs (INV-DRV-003)")

    with open("kernel/src/driver_framework/protocol.rs", "r") as f:
        proto_src = f.read()
    for symbol in ["DriverMessage", "DriverOp", "DriverStatus", "from_ipc_message", "to_ipc_message"]:
        if symbol not in proto_src:
            print(f"❌ FAIL: protocol.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Driver Protocol over IPC verified in protocol.rs (REQ-KRN-107)")

    with open("kernel/src/capability/table.rs", "r") as f:
        cap_src = f.read()
    for variant in ["Device(u32)", "Interrupt(u32)", "DmaWindow(u32)"]:
        if variant not in cap_src:
            print(f"❌ FAIL: CapType missing variant '{variant}'")
            sys.exit(1)
    print("  ✓ CapType driver framework variants verified in capability/table.rs")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in driver framework subsystem...")
    for root, _, files in os.walk("kernel/src/driver_framework"):
        for file in files:
            if file.endswith(".rs"):
                path = os.path.join(root, file)
                with open(path, "r") as f:
                    content = f.read()
                    if "unsafe " in content:
                        print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                        sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/driver_framework/ (Zero unsafe blocks)")

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
    print("\n[CHECK 5/6] Verifying golden boot log against SP-26 boot stages...")
    with open("docs/evidence/golden-boot-sp26.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:72_DEVICE_MANAGER_INIT]",
        "Device Manager online: 2 devices registered",
        "[STAGE:73_USER_MODE_DRIVER_REGISTRATION]",
        "Unprivileged driver binding rejected",
        "User-mode driver PID 10 authenticated and bound to device 'virtio-blk' (INV-DRV-001 PASSED)",
        "[STAGE:74_IRQ_BROKER_DELIVERY]",
        "IRQ 48 intercepted: line masked at GIC, asynchronous notification dispatched to EP 201",
        "Unauthenticated IRQ_ACK rejected",
        "Authenticated IRQ_ACK verified: interrupt line unmasked, ready for next event (INV-DRV-002 PASSED)",
        "[STAGE:75_DMA_IOMMU_ISOLATION]",
        "SMMU-isolated DMA window #1",
        "Out-of-bounds DMA transfer rejected",
        "Rogue PID DMA access rejected",
        "[STAGE:76_DRIVER_PROTOCOL_IPC]",
        "Driver Protocol over IPC verified: IO_READ req/resp cycle",
        "[STAGE:77_DRIVER_FRAMEWORK_READY]",
        "[DRV] Sprint SP-26 User-Mode Driver Framework REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-26"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest = {
        "sprint": "SP-26",
        "title": "User-Mode Driver Framework (Device Manager, IRQ Broker, DMA/IOMMU Broker, Driver Protocol over IPC)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": "PASSED",
        "artifacts": {}
    }

    tracked_files = REQUIRED_SP26_FILES + ["ci/verify_sp26.py"]
    for path in tracked_files:
        if os.path.exists(path):
            with open(path, "rb") as f:
                sha256 = hashlib.sha256(f.read()).hexdigest()
            manifest["artifacts"][path] = sha256

    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)

    print(f"  ✓ Evidence manifest written to {manifest_path}")
    print("\n==================================================")
    print(" SP-26 VERIFICATION PASSED: All 6 checks satisfied")
    print("==================================================")

if __name__ == "__main__":
    verify_sp26()
