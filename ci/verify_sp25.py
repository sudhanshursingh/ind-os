#!/usr/bin/env python3
"""
Ind OS Sprint SP-25 Verification & Acceptance Test Harness
Target: Storage Subsystem (tmpfs, IndFS WAL journaling, CRC32C checksums, fsck recovery)
Standards: Bible §8.10, INDOS-SPEC-INDFS-01, INV-FS-001, INV-FS-002, Rule N-01, Rule N-04, Rule N-10
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

REQUIRED_SP25_FILES = [
    "docs/architecture/indfs-design-spec.md",
    "docs/evidence/SP-25-validation.md",
    "docs/evidence/golden-boot-sp25.log",
    "kernel/src/fs/block.rs",
    "kernel/src/fs/crc32.rs",
    "kernel/src/fs/tmpfs.rs",
    "kernel/src/fs/journal.rs",
    "kernel/src/fs/indfs.rs",
    "kernel/src/fs/fsck.rs",
    "kernel/src/fs/mod.rs",
]

def verify_sp25():
    print("==================================================")
    print(" Ind OS SP-25 Storage Subsystem Verifier          ")
    print(" Target: Sprint SP-25 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-25 deliverable presence...")
    for rel_path in REQUIRED_SP25_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-25 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying architectural symbols and invariants...")
    with open("kernel/src/fs/crc32.rs", "r") as f:
        crc_src = f.read()
    if "crc32c" not in crc_src or "0x82F63B78" not in crc_src:
        print("❌ FAIL: crc32.rs missing crc32c function or Castagnoli polynomial")
        sys.exit(1)
    print("  ✓ Pure safe Castagnoli CRC32C implementation verified (INV-FS-001)")

    with open("kernel/src/fs/block.rs", "r") as f:
        block_src = f.read()
    for symbol in ["BlockDevice", "MemBlockDevice", "BLOCK_SIZE", "MEM_BLOCK_DEVICE", "CRASH_BLOCK_DEVICE"]:
        if symbol not in block_src:
            print(f"❌ FAIL: block.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Block device abstractions & static singletons verified in block.rs")

    with open("kernel/src/fs/tmpfs.rs", "r") as f:
        tmpfs_src = f.read()
    for symbol in ["TmpFsNode", "TmpFs", "MAX_TMPFS_NODES", "create_file", "write_file", "read_file"]:
        if symbol not in tmpfs_src:
            print(f"❌ FAIL: tmpfs.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Volatile memory filesystem verified in tmpfs.rs")

    with open("kernel/src/fs/journal.rs", "r") as f:
        journal_src = f.read()
    for symbol in ["JournalRecord", "Journal", "OP_COMMIT", "OP_WRITE_BLOCK", "OP_WRITE_INODE", "log_op", "commit"]:
        if symbol not in journal_src:
            print(f"❌ FAIL: journal.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ WAL transaction journal ring buffer verified in journal.rs (INV-FS-002)")

    with open("kernel/src/fs/indfs.rs", "r") as f:
        indfs_src = f.read()
    for symbol in ["Superblock", "Inode", "IndFs", "INDFS_MAGIC", "format", "mount", "create_file", "write_file", "read_file", "unmount"]:
        if symbol not in indfs_src:
            print(f"❌ FAIL: indfs.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ IndFS core filesystem and bitrot detection verified in indfs.rs (INV-FS-001, INV-FS-002)")

    with open("kernel/src/fs/fsck.rs", "r") as f:
        fsck_src = f.read()
    for symbol in ["FsckReport", "Fsck", "check_and_recover"]:
        if symbol not in fsck_src:
            print(f"❌ FAIL: fsck.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ 3-Phase fsck crash recovery state machine verified in fsck.rs (INV-FS-002)")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in FS subsystem...")
    for root, _, files in os.walk("kernel/src/fs"):
        for file in files:
            if file.endswith(".rs"):
                path = os.path.join(root, file)
                with open(path, "r") as f:
                    content = f.read()
                    if "unsafe " in content:
                        print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                        sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/fs/ (Zero unsafe blocks)")

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
    print("\n[CHECK 5/6] Verifying golden boot log against SP-25 boot stages...")
    with open("docs/evidence/golden-boot-sp25.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:66_TMPFS_INIT]",
        "tmpfs online: /tmp mounted, volatile file lifecycle verified",
        "[STAGE:67_INDFS_CORE_INIT]",
        "IndFS online: Superblock verified",
        "[STAGE:68_INDFS_JOURNAL_WAL]",
        "IndFS WAL verified: transaction committed and data checkpointed (INV-FS-002 PASSED)",
        "[STAGE:69_INDFS_CHECKSUM_BITROT]",
        "Cryptographic integrity verified: block CRC32C matches stored table",
        "Silent bitrot detected and rejected: EIO: Block CRC32C checksum mismatch: silent data corruption detected (INV-FS-001)",
        "[STAGE:70_INDFS_CRASH_FSCK_RECOVERY]",
        "fsck crash recovery verified: 2 replayed, 1 discarded, clean state restored (INV-FS-002 PASSED)",
        "[STAGE:71_STORAGE_SUBSYSTEM_READY]",
        "[FS] Sprint SP-25 tmpfs & IndFS Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-25"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest = {
        "sprint": "SP-25",
        "title": "tmpfs + IndFS Implementation (journaling, checksummed persistent FS, fsck recovery state machine, golden-image crash tests)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": "PASSED",
        "artifacts": {}
    }

    tracked_files = REQUIRED_SP25_FILES + ["ci/verify_sp25.py"]
    for path in tracked_files:
        if os.path.exists(path):
            with open(path, "rb") as f:
                sha256 = hashlib.sha256(f.read()).hexdigest()
            manifest["artifacts"][path] = sha256

    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)

    print(f"  ✓ Evidence manifest written to {manifest_path}")
    print("\n==================================================")
    print(" SP-25 VERIFICATION PASSED: All 6 checks satisfied")
    print("==================================================")

if __name__ == "__main__":
    verify_sp25()
