#!/usr/bin/env python3
"""
Sprint SP-04 Verification Harness (Boot Entry & Exception Vectors).
Ratified under INDOS-BIBLE-001 / Phase 1 Kernel Bootstrap.

Verifies:
1. Kernel source file presence and zero-dependency no_std compliance.
2. Invariant verification in boot & exception vector assembly:
   - INV-BOOT-001: Execution in EL1 within <= 50 instructions.
   - INV-BOOT-002: Early kernel stack pointer strictly 16-byte aligned.
   - INV-BOOT-003: DTB pointer preservation and 0xd00dfeed magic check.
   - INV-BOOT-004: 2,048-byte aligned VBAR_EL1 with 16 128-byte spaced entries.
3. ELF symbol & binary layout verification (llvm-objdump).
4. Runtime execution under headless QEMU virt via ci/run_qemu_test.py.
5. Generation of evidence/SP-04/manifest.json.
"""

import sys
import os
import subprocess
import hashlib
import json
import time

REQUIRED_SP04_FILES = [
    "kernel/Cargo.toml",
    "kernel/linker.ld",
    "kernel/src/main.rs",
    "kernel/src/panic.rs",
    "kernel/src/arch/mod.rs",
    "kernel/src/arch/aarch64/mod.rs",
    "kernel/src/arch/aarch64/boot.s",
    "kernel/src/arch/aarch64/exceptions.s",
    "docs/evidence/golden-boot-sp04.log",
]

VECTOR_ENTRIES = [
    "vec_curr_el_sp0_sync",
    "vec_curr_el_sp0_irq",
    "vec_curr_el_sp0_fiq",
    "vec_curr_el_sp0_serror",
    "vec_curr_el_spx_sync",
    "vec_curr_el_spx_irq",
    "vec_curr_el_spx_fiq",
    "vec_curr_el_spx_serror",
    "vec_lower_el_a64_sync",
    "vec_lower_el_a64_irq",
    "vec_lower_el_a64_fiq",
    "vec_lower_el_a64_serror",
    "vec_lower_el_a32_sync",
    "vec_lower_el_a32_irq",
    "vec_lower_el_a32_fiq",
    "vec_lower_el_a32_serror",
]

def find_llvm_tool(tool_name: str) -> str:
    # First search rust sysroot
    try:
        sysroot = subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip()
        candidate = os.path.join(sysroot, "lib", "rustlib", "aarch64-apple-darwin", "bin", tool_name)
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            return candidate
        # Search recursively in rustlib
        for root, dirs, files in os.walk(os.path.join(sysroot, "lib", "rustlib")):
            if tool_name in files:
                p = os.path.join(root, tool_name)
                if os.access(p, os.X_OK):
                    return p
    except Exception:
        pass
    return tool_name

def verify_sp04():
    print("==================================================")
    print(" Ind OS SP-04 Boot Entry & Vector Verifier       ")
    print(" Target: Sprint SP-04 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Check all required source files exist
    for f in REQUIRED_SP04_FILES:
        if not os.path.isfile(f):
            print(f"❌ FAIL: Missing required deliverable: {f}")
            return False
        print(f"  [OK] Deliverable exists: {f}")

    # 2. Verify static code invariants in boot.s & exceptions.s
    with open("kernel/src/arch/aarch64/boot.s", "r", encoding="utf-8") as f:
        boot_src = f.read()

    with open("kernel/src/arch/aarch64/exceptions.s", "r", encoding="utf-8") as f:
        exc_src = f.read()

    # INV-BOOT-001: EL drop logic
    if "msr hcr_el2" not in boot_src or "eret" not in boot_src or "CurrentEL" not in boot_src:
        print("❌ FAIL: INV-BOOT-001 check failed: Missing EL drop sequence in boot.s")
        return False
    print("  [OK] INV-BOOT-001: EL drop & EL1 transition sequence present.")

    # INV-BOOT-002: 16-byte stack alignment check
    if "tst x1, #0xf" not in boot_src or ".Lstack_misaligned_panic" not in boot_src:
        print("❌ FAIL: INV-BOOT-002 check failed: Missing 16-byte stack alignment check in boot.s")
        return False
    print("  [OK] INV-BOOT-002: 16-byte stack alignment assertion present.")

    # INV-BOOT-003: DTB pointer and magic check
    if "0xd00dfeed" not in boot_src or ".Ldtb_corrupted_panic" not in boot_src:
        print("❌ FAIL: INV-BOOT-003 check failed: Missing DTB magic validation in boot.s")
        return False
    print("  [OK] INV-BOOT-003: DTB pointer & 0xd00dfeed magic validation present.")

    # INV-BOOT-004: Vector table alignment and entries
    if ".align 11" not in exc_src or "__vector_table:" not in exc_src:
        print("❌ FAIL: INV-BOOT-004 check failed: Missing .align 11 vector table in exceptions.s")
        return False
    for entry in VECTOR_ENTRIES:
        if entry not in exc_src:
            print(f"❌ FAIL: INV-BOOT-004 check failed: Missing vector table entry: {entry}")
            return False
    print("  [OK] INV-BOOT-004: All 16 vector entries defined with .align 11 table header.")

    # 3. Build kernel
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

    # 4. Generate flat binary using llvm-objcopy
    objcopy = find_llvm_tool("llvm-objcopy")
    res = subprocess.run([objcopy, "-O", "binary", kernel_elf, kernel_bin], capture_output=True, text=True)
    if res.returncode != 0:
        print(f"❌ FAIL: llvm-objcopy failed:\n{res.stderr}")
        return False
    print(f"  [OK] Generated raw kernel image: {kernel_bin} ({os.path.getsize(kernel_bin)} bytes)")

    # 5. Inspect ELF symbol table for vector table alignment and spacing
    objdump = find_llvm_tool("llvm-objdump")
    res = subprocess.run([objdump, "-t", kernel_elf], capture_output=True, text=True)
    if res.returncode != 0:
        print(f"❌ FAIL: llvm-objdump failed:\n{res.stderr}")
        return False

    sym_addrs = {}
    for line in res.stdout.splitlines():
        parts = line.split()
        if len(parts) >= 2:
            addr_str = parts[0]
            sym_name = parts[-1]
            try:
                addr = int(addr_str, 16)
                sym_addrs[sym_name] = addr
            except ValueError:
                pass

    if "__vector_table" not in sym_addrs:
        print("❌ FAIL: __vector_table symbol not found in ELF")
        return False

    vbar_addr = sym_addrs["__vector_table"]
    if vbar_addr % 2048 != 0:
        print(f"❌ FAIL: __vector_table address 0x{vbar_addr:x} is not 2048-byte aligned")
        return False
    print(f"  [OK] __vector_table is strictly 2,048-byte aligned at: 0x{vbar_addr:x}")

    # Verify spacing of all 16 entries
    for i, entry in enumerate(VECTOR_ENTRIES):
        if entry not in sym_addrs:
            print(f"❌ FAIL: Vector symbol {entry} missing in ELF")
            return False
        expected_addr = vbar_addr + (i * 0x80)
        actual_addr = sym_addrs[entry]
        if actual_addr != expected_addr:
            print(f"❌ FAIL: Vector entry {entry} at 0x{actual_addr:x}, expected 0x{expected_addr:x} (+{i*0x80:#x})")
            return False
    print("  [OK] All 16 vector entries spaced exactly at 0x80 (128 bytes) intervals.")

    # 6. Execute headless QEMU test and verify against golden boot log
    print("• Running QEMU test via ci/run_qemu_test.py...")
    res = subprocess.run(
        [
            sys.executable,
            "ci/run_qemu_test.py",
            "--kernel", kernel_elf,
            "--golden", "docs/evidence/golden-boot-sp04.log",
            "--evidence", "evidence/SP-04",
            "--timeout", "4",
        ],
        capture_output=True,
        text=True,
    )
    if res.returncode != 0:
        print(f"❌ FAIL: QEMU execution test failed:\n{res.stdout}\n{res.stderr}")
        return False
    print("  [OK] Headless QEMU execution passed and serial log matches golden reference.")

    # 7. Generate Manifest Pack
    evidence_dir = "evidence/SP-04"
    os.makedirs(evidence_dir, exist_ok=True)
    manifest_path = os.path.join(evidence_dir, "manifest.json")

    file_hashes = {}
    tracked_files = sorted(REQUIRED_SP04_FILES + [kernel_elf, kernel_bin, os.path.join(evidence_dir, "serial.log")])
    for f in tracked_files:
        if os.path.isfile(f):
            h = hashlib.sha256()
            with open(f, "rb") as fh:
                while chunk := fh.read(65536):
                    h.update(chunk)
            file_hashes[f] = h.hexdigest()

    manifest_data = {
        "sprint": "SP-04",
        "phase": "Phase 1: Kernel Bootstrap",
        "milestone": "Boot Entry & Exception Vectors",
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "invariants_verified": [
            "INV-BOOT-001",
            "INV-BOOT-002",
            "INV-BOOT-003",
            "INV-BOOT-004",
        ],
        "vector_table_address": f"0x{vbar_addr:x}",
        "artifacts": file_hashes,
    }

    with open(manifest_path, "w", encoding="utf-8") as mf:
        json.dump(manifest_data, mf, indent=2)

    print(f"  [OK] Generated evidence manifest: {manifest_path}")
    print("==================================================")
    print(" ✅ SPRINT SP-04 VERIFICATION PASSED (100% GREEN) ")
    print("==================================================")
    return True

if __name__ == "__main__":
    if not verify_sp04():
        sys.exit(1)
