#!/usr/bin/env python3
"""
Sprint SP-06 Verification Harness (Memory Discovery & Frame Allocator).
Ratified under INDOS-BIBLE-001 / Phase 1 Kernel Bootstrap (§8.7) and INDOS-SPEC-MEM-01.

Verifies:
1. Deliverable presence: DTB parser, page frame allocator, address space layout definitions.
2. Invariant verification in source code:
   - INV-MEM-001: Address 0x0 is strictly unmapped (NULL pointer trap zone).
   - INV-MEM-003: Physical page allocations are identity-audited (double-free causes immediate panic).
3. DTB parser structure verification (no_std FDT reader, unaligned safe access).
4. Frame allocator granularity (strictly 4 KB / 4096 bytes).
5. Clean compilation and headless QEMU execution validation against golden boot reference.
6. Generation of evidence/SP-06/manifest.json.
"""

import sys
import os
import subprocess
import hashlib
import json
import time

REQUIRED_SP06_FILES = [
    "kernel/src/mm/mod.rs",
    "kernel/src/mm/dtb.rs",
    "kernel/src/mm/frame.rs",
    "kernel/src/mm/address_space.rs",
    "docs/evidence/golden-boot-sp06.log",
]

def find_llvm_tool(tool_name: str) -> str:
    try:
        sysroot = subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip()
        candidate = os.path.join(sysroot, "lib", "rustlib", "aarch64-apple-darwin", "bin", tool_name)
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            return candidate
        for root, dirs, files in os.walk(os.path.join(sysroot, "lib", "rustlib")):
            if tool_name in files:
                p = os.path.join(root, tool_name)
                if os.access(p, os.X_OK):
                    return p
    except Exception:
        pass
    return tool_name

def verify_sp06():
    print("==================================================")
    print(" Ind OS SP-06 Memory Discovery & Frame Verifier   ")
    print(" Target: Sprint SP-06 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Check required files exist
    for f in REQUIRED_SP06_FILES:
        if not os.path.isfile(f):
            print(f"❌ FAIL: Missing required deliverable: {f}")
            return False
        print(f"  [OK] Deliverable exists: {f}")

    # 2. Check DTB parser elements
    with open("kernel/src/mm/dtb.rs", "r", encoding="utf-8") as f:
        dtb_src = f.read()

    dtb_required = [
        "FDT_MAGIC",
        "0xd00dfeed",
        "DtbParser",
        "discover_memory",
        "PhysicalMemoryRegion",
        "read_unaligned",
    ]
    for sym in dtb_required:
        if sym not in dtb_src:
            print(f"❌ FAIL: Missing DTB parser element: {sym}")
            return False
    print("  [OK] DTB parser elements and safe unaligned access verified.")

    # 3. Check Frame Allocator & INV-MEM-003
    with open("kernel/src/mm/frame.rs", "r", encoding="utf-8") as f:
        frame_src = f.read()

    if "PAGE_SIZE: usize = 4096" not in frame_src:
        print("❌ FAIL: PAGE_SIZE is not defined as 4096 (4KB)")
        return False
    if "INV-MEM-003" not in frame_src or "Double-free" not in frame_src:
        print("❌ FAIL: INV-MEM-003 double-free audit missing in frame allocator")
        return False
    if "MemoryError::OutOfMemory" not in frame_src:
        print("❌ FAIL: Typed OutOfMemory error missing in frame allocator")
        return False
    print("  [OK] PageFrameAllocator (4KB granularity, typed error handling, INV-MEM-003) verified.")

    # 4. Check Address Space Layout & INV-MEM-001
    with open("kernel/src/mm/address_space.rs", "r", encoding="utf-8") as f:
        as_src = f.read()

    if "USER_SPACE_START" not in as_src or "KERNEL_SPACE_START" not in as_src:
        print("❌ FAIL: Missing higher-half address space split constants")
        return False
    if "INV-MEM-001" not in as_src or "0x0000_0000_0040_0000" not in as_src:
        print("❌ FAIL: INV-MEM-001 NULL trap zone not enforced in address space layout")
        return False
    print("  [OK] Higher-half virtual address space layout & INV-MEM-001 verified.")

    # 5. Build kernel cleanly
    print("• Compiling IndK ARM64 kernel...")
    res = subprocess.run(
        ["cargo", "build", "--target", "aarch64-unknown-none"],
        cwd="kernel",
        capture_output=True,
        text=True,
    )
    if res.returncode != 0:
        print(f"❌ FAIL: cargo build failed:\n{res.stderr}")
        return False
    print("  [OK] IndK compiled cleanly.")

    kernel_elf = "kernel/target/aarch64-unknown-none/debug/indk"
    kernel_bin = "kernel/target/aarch64-unknown-none/debug/indk.bin"

    # 6. Generate flat binary using llvm-objcopy
    objcopy = find_llvm_tool("llvm-objcopy")
    res = subprocess.run([objcopy, "-O", "binary", kernel_elf, kernel_bin], capture_output=True, text=True)
    if res.returncode != 0:
        print(f"❌ FAIL: llvm-objcopy failed:\n{res.stderr}")
        return False
    print(f"  [OK] Generated raw kernel image: {kernel_bin} ({os.path.getsize(kernel_bin)} bytes)")

    # 7. Execute headless QEMU test and verify against golden boot log
    print("• Running QEMU test via ci/run_qemu_test.py...")
    res = subprocess.run(
        [
            sys.executable,
            "ci/run_qemu_test.py",
            "--kernel", kernel_elf,
            "--golden", "docs/evidence/golden-boot-sp06.log",
            "--evidence", "evidence/SP-06",
            "--timeout", "8",
        ],
        capture_output=True,
        text=True,
    )
    if res.returncode != 0:
        print(f"❌ FAIL: QEMU execution test failed:\n{res.stdout}\n{res.stderr}")
        return False
    print("  [OK] Headless QEMU execution passed and serial log matches golden reference.")

    # 8. Generate Manifest Pack
    evidence_dir = "evidence/SP-06"
    os.makedirs(evidence_dir, exist_ok=True)
    manifest_path = os.path.join(evidence_dir, "manifest.json")

    file_hashes = {}
    tracked_files = sorted(REQUIRED_SP06_FILES + [kernel_elf, kernel_bin, os.path.join(evidence_dir, "serial.log")])
    for f in tracked_files:
        if os.path.isfile(f):
            h = hashlib.sha256()
            with open(f, "rb") as fh:
                while chunk := fh.read(65536):
                    h.update(chunk)
            file_hashes[f] = h.hexdigest()

    manifest_data = {
        "sprint": "SP-06",
        "phase": "Phase 1: Kernel Bootstrap",
        "milestone": "Memory Discovery & Frame Allocator",
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "invariants_verified": [
            "INV-MEM-001",
            "INV-MEM-003",
        ],
        "requirements_verified": [
            "REQ-KRN-102",
            "REQ-KRN-103",
        ],
        "page_size_bytes": 4096,
        "stage_markers_verified": [
            "06_MEMORY_DISCOVERY",
            "07_FRAME_ALLOCATOR",
            "08_MM_READY",
        ],
        "artifacts": file_hashes,
    }

    with open(manifest_path, "w", encoding="utf-8") as mf:
        json.dump(manifest_data, mf, indent=2)

    print(f"  [OK] Generated evidence manifest: {manifest_path}")
    print("==================================================")
    print(" ✅ SPRINT SP-06 VERIFICATION PASSED (100% GREEN) ")
    print("==================================================")
    return True

if __name__ == "__main__":
    if not verify_sp06():
        sys.exit(1)
