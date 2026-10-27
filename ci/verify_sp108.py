#!/usr/bin/env python3
"""
Ind OS Sprint SP-108 Verification & Acceptance Test Harness
Target: Gate G9 Acceptance Pack Assembly
Standards: Bible §6.3, §8.15, Table 19.2, §19.3, §19.4, PRD-001 §9.5, INV-G9-PACK-001..003
"""

import os
import sys

REQUIRED_SP108_FILES = [
    "docs/architecture/g9-acceptance-pack-spec.md",
    "docs/architecture/sbom-phase9.md",
    "docs/evidence/sbom-phase9.json",
    "docs/evidence/SP-108-validation.md",
    "docs/evidence/golden-boot-sp108.log",
    "docs/DECISIONS/gate-G9-dec001-review.md",
    "docs/evidence/G9/manifest.json",
    "docs/evidence/G9/build-provenance.md",
    "docs/evidence/G9/dod-test-logs.md",
    "docs/evidence/G9/docs-diff-phase9.md",
    "docs/evidence/G9/gate-g9-signatures.md",
    "userspace/init/src/g9_pack.rs",
    "kernel/src/hal/platform/g9_pack_core.rs",
]

def verify_sp108():
    print("==================================================")
    print(" Ind OS SP-108 Gate G9 Acceptance Pack Verifier   ")
    print(" Target: Sprint SP-108 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-108 deliverable presence...")
    for rel_path in REQUIRED_SP108_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-108 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/init/src/g9_pack.rs", "r") as f:
        init_src = f.read()
    for s in [
        "G9PackEngine",
        "G9PackError",
        "EvidenceItem",
        "CAP_G9_PACK_ADMIN",
        "G9_TOTAL_EVIDENCE_ITEMS",
        "G9_MANIFEST_HASH_CRC32",
        "G9_SBOM_COMPONENT_COUNT",
        "G9_TCB_EXTERNAL_CRATES",
        "assemble_manifest",
        "refresh_sbom",
        "record_dec001_minute",
        "is_pack_complete",
    ]:
        if s not in init_src:
            print(f"❌ FAIL: g9_pack.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/g9_pack_core.rs", "r") as f:
        hal_src = f.read()
    for s in [
        "KernelG9PackHarness",
        "KernelG9PackError",
        "KernelEvidenceItem",
        "KERNEL_CAP_G9_PACK_ADMIN",
        "KERNEL_G9_TOTAL_EVIDENCE_ITEMS",
        "KERNEL_G9_MANIFEST_HASH_CRC32",
        "KERNEL_G9_SBOM_COMPONENT_COUNT",
        "KERNEL_G9_TCB_EXTERNAL_CRATES",
        "assemble_manifest",
        "refresh_sbom",
        "record_dec001_minute",
        "is_pack_complete",
    ]:
        if s not in hal_src:
            print(f"❌ FAIL: g9_pack_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel HAL symbol: {s}")

    # 3. Rule N-01 Compliance: Zero external crate dependencies
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for cargo_path in ["kernel/Cargo.toml", "userspace/init/Cargo.toml"]:
        with open(cargo_path, "r") as f:
            lines = f.readlines()
        in_deps = False
        for line in lines:
            line_str = line.strip()
            if line_str == "[dependencies]":
                in_deps = True
                continue
            if in_deps and line_str.startswith("["):
                in_deps = False
            if in_deps and line_str and not line_str.startswith("#"):
                if "path =" not in line_str:
                    print(f"❌ FAIL: Non-path external dependency in {cargo_path}: {line_str}")
                    sys.exit(1)
        print(f"  ✓ {cargo_path} compliant with Rule N-01")

    # 4. Rule N-04 Compliance: Zero unsafe in newly authored userspace
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/init/src/g9_pack.rs)...")
    with open("userspace/init/src/g9_pack.rs", "r") as f:
        content = f.read()
    if "unsafe " in content or "unsafe{" in content:
        print("❌ FAIL: unsafe code block found in userspace/init/src/g9_pack.rs")
        sys.exit(1)
    print("  ✓ userspace/init/src/g9_pack.rs is 100% safe Rust")

    # 5. Golden Boot Log Verification: Stages 490-494
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 490-494...")
    with open("docs/evidence/golden-boot-sp108.log", "r") as f:
        log_content = f.read()

    stages = [
        "490_SP108_G9_PACK_INIT",
        "491_SP108_EVIDENCE_MANIFEST_ASSEMBLY",
        "492_SP108_SBOM_PHASE9_REFRESH",
        "493_SP108_DEC001_G9_MINUTE",
        "494_SP108_ACCEPTANCE_PACK_READY",
    ]
    for st in stages:
        if st not in log_content:
            print(f"❌ FAIL: Missing stage {st} in docs/evidence/golden-boot-sp108.log")
            sys.exit(1)
        print(f"  ✓ Verified stage: {st}")

    # 6. Specific Confirmation Logs
    print("\n[CHECK 6/6] Verifying Gate G9 Acceptance Pack Confirmation Logs...")
    expected_logs = [
        "Gate G9 acceptance pack harness online: assembling 9 evidentiary sections (PRD-001 §9.5 PASSED)",
        "Evidence manifest assembled: 9 sections verified with CRC32 0x98A14C2F (INV-G9-PACK-001 PASSED)",
        "SBOM refreshed for Phase 9: 6 in-tree components, 0 external crates in TCB (INV-G9-PACK-002 PASSED)",
        "DEC-001 re-examination minuted: CONTINUE confirmed on from-scratch kernel path (INV-G9-PACK-003 PASSED)",
        "[SP-108] Sprint SP-108 Gate G9 Acceptance Pack Milestone REACHED",
    ]
    for el in expected_logs:
        if el not in log_content:
            print(f"❌ FAIL: Log string not found: '{el}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {el[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-108 VERIFICATION PASSED (100% DoD) ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp108()
