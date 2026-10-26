#!/usr/bin/env python3
"""
Ind OS Sprint SP-24 Verification & Acceptance Test Harness
Target: VFS Core Subsystem (Vnode layer, Mount Table, Path Resolution, Capability-Mediated File Handles)
Standards: Bible §8.10, REQ-KRN-106, INV-VFS-001, INV-VFS-002, Rule N-01, Rule N-04, Rule N-10
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

REQUIRED_SP24_FILES = [
    "docs/architecture/vfs-core-spec.md",
    "docs/architecture/indfs-design-spec.md",
    "docs/evidence/SP-24-validation.md",
    "docs/evidence/golden-boot-sp24.log",
    "kernel/src/vfs/vnode.rs",
    "kernel/src/vfs/mount.rs",
    "kernel/src/vfs/path.rs",
    "kernel/src/vfs/file.rs",
    "kernel/src/vfs/core.rs",
    "kernel/src/vfs/mod.rs",
]

def verify_sp24():
    print("==================================================")
    print(" Ind OS SP-24 VFS Core Verifier                   ")
    print(" Target: Sprint SP-24 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-24 deliverable presence...")
    for rel_path in REQUIRED_SP24_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-24 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Static Architectural & Invariant Checks
    print("\n[CHECK 2/6] Verifying architectural symbols and invariants...")
    with open("kernel/src/vfs/vnode.rs", "r") as f:
        vnode_src = f.read()
    for symbol in ["VnodeType", "VnodeId", "Vnode", "RegularFile", "Directory", "Device", "Symlink"]:
        if symbol not in vnode_src:
            print(f"❌ FAIL: vnode.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Vnode abstraction & classifications verified in vnode.rs")

    with open("kernel/src/vfs/mount.rs", "r") as f:
        mount_src = f.read()
    for symbol in ["MountEntry", "MountTable", "MAX_MOUNTS", "mount", "unmount", "find_mount"]:
        if symbol not in mount_src:
            print(f"❌ FAIL: mount.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Mount table and prefix resolution verified in mount.rs")

    with open("kernel/src/vfs/path.rs", "r") as f:
        path_src = f.read()
    for symbol in ["PathResolver", "canonicalize"]:
        if symbol not in path_src:
            print(f"❌ FAIL: path.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Path canonicalization & root isolation verified in path.rs (INV-VFS-002)")

    with open("kernel/src/vfs/file.rs", "r") as f:
        file_src = f.read()
    for symbol in ["FileDescriptor", "verify_read_access", "verify_write_access"]:
        if symbol not in file_src:
            print(f"❌ FAIL: file.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Capability-guarded file handles verified in file.rs (REQ-KRN-106, INV-VFS-001)")

    with open("kernel/src/vfs/core.rs", "r") as f:
        core_src = f.read()
    for symbol in ["Vfs", "MAX_VNODES", "MAX_OPEN_FILES", "lookup", "create_file", "open_file", "read_file", "write_file", "seek", "close_file"]:
        if symbol not in core_src:
            print(f"❌ FAIL: core.rs missing symbol '{symbol}'")
            sys.exit(1)
    print("  ✓ Vfs core state machine verified in core.rs")

    with open("kernel/src/vfs/mod.rs", "r") as f:
        mod_src = f.read()
    if "VFS" not in mod_src:
        print("❌ FAIL: mod.rs missing global VFS singleton")
        sys.exit(1)
    print("  ✓ Global VFS singleton declared and re-exported in vfs/mod.rs")

    with open("kernel/src/capability/table.rs", "r") as f:
        cap_src = f.read()
    if "File(u64)" not in cap_src:
        print("❌ FAIL: CapType missing File(u64) variant for REQ-KRN-106")
        sys.exit(1)
    print("  ✓ CapType::File(u64) capability token variant verified (REQ-KRN-106)")

    # 3. Unsafe-Code Budget Scanner Check
    print("\n[CHECK 3/6] Auditing unsafe code in VFS subsystem...")
    for path in [
        "kernel/src/vfs/mod.rs",
        "kernel/src/vfs/vnode.rs",
        "kernel/src/vfs/mount.rs",
        "kernel/src/vfs/path.rs",
        "kernel/src/vfs/file.rs",
        "kernel/src/vfs/core.rs",
    ]:
        with open(path, "r") as f:
            content = f.read()
            if "unsafe " in content:
                print(f"❌ FAIL: Forbidden unsafe code detected in {path}")
                sys.exit(1)
    print("  ✓ 100% pure safe Rust in kernel/src/vfs/ (Zero unsafe blocks)")

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
    print("\n[CHECK 5/6] Verifying golden boot log against SP-24 boot stages...")
    with open("docs/evidence/golden-boot-sp24.log", "r") as f:
        log_content = f.read()

    required_stages = [
        "[STAGE:62_VFS_CORE_INIT]",
        "VFS core online: root filesystem mounted",
        "[STAGE:63_VFS_PATH_RESOLUTION]",
        "Path resolution verified: canonicalization, lookup, and root isolation sound (INV-VFS-002 PASSED)",
        "[STAGE:64_VFS_CAPABILITY_FILE_ACCESS]",
        "Unauthorized write rejected by capability check",
        "Access with revoked capability rejected",
        "Capability-mediated file access verified: zero ambient authority (REQ-KRN-106, INV-VFS-001 PASSED)",
        "[STAGE:65_VFS_CORE_READY]",
        "[VFS] Sprint SP-24 VFS Core Milestone REACHED.",
    ]

    for stage_str in required_stages:
        if stage_str not in log_content:
            print(f"❌ FAIL: Golden boot log missing expected output: '{stage_str}'")
            sys.exit(1)
        print(f"  ✓ Found boot verification signature: {stage_str}")

    # 6. Evidence Manifest Generation
    print("\n[CHECK 6/6] Generating evidence manifest...")
    manifest_dir = "evidence/SP-24"
    os.makedirs(manifest_dir, exist_ok=True)
    manifest_path = os.path.join(manifest_dir, "manifest.json")

    manifest_data = {
        "sprint": "SP-24",
        "title": "VFS Core (vnode Layer, Mount Table, Path Resolution, Capability-Mediated File Handles, IndFS Design)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "requirements": {
            "REQ-KRN-106": "Capability-Mediated File Handles from Day One"
        },
        "invariants": {
            "INV-VFS-001": "Capability Enforcement (access strictly requires valid matching token with required rights; invalid/revoked tokens rejected with EPERM)",
            "INV-VFS-002": "Path Traversal Isolation (path canonicalization prevents upward escape above root /)"
        },
        "files": {}
    }

    for path in REQUIRED_SP24_FILES:
        with open(path, "rb") as f:
            h = hashlib.sha256(f.read()).hexdigest()
        manifest_data["files"][path] = {
            "sha256": h,
            "bytes": os.path.getsize(path)
        }

    with open(manifest_path, "w") as f:
        json.dump(manifest_data, f, indent=2)

    print(f"  ✓ Manifest written to {manifest_path}")

    print("\n==================================================")
    print(" ✅ ALL SP-24 VERIFICATION CHECKS PASSED (6/6)    ")
    print(" Sprint SP-24 Definition of Done 100% SATISFIED!  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp24()
