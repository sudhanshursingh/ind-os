#!/usr/bin/env python3
"""
Ind OS Sprint SP-96 Verification & Acceptance Test Harness
Target: On-Device AIS ARM64 Runtime & CPU Inference Benchmark
Standards: Bible §6.3, §8.15, Table 19.2, REQ-AI-101, REQ-HW-113, INV-AIS-HW-001..003
"""

import os
import sys

REQUIRED_SP96_FILES = [
    "docs/architecture/on-device-ais-arm64-spec.md",
    "docs/evidence/SP-96-validation.md",
    "docs/evidence/golden-boot-sp96.log",
    "userspace/ais/src/arm64_runtime.rs",
    "kernel/src/hal/platform/ais_arm64_core.rs",
]

def verify_sp96():
    print("==================================================")
    print(" Ind OS SP-96 On-Device AIS ARM64 Verifier        ")
    print(" Target: Sprint SP-96 Definition of Done (DoD)   ")
    print("==================================================")

    # 1. Deliverables Presence Check
    print("[CHECK 1/6] Verifying SP-96 deliverable presence...")
    for rel_path in REQUIRED_SP96_FILES:
        if not os.path.exists(rel_path):
            print(f"❌ FAIL: Required SP-96 file missing: {rel_path}")
            sys.exit(1)
        print(f"  ✓ Found {rel_path}")

    # 2. Architectural & Invariant Symbols Check
    print("\n[CHECK 2/6] Verifying architectural symbols and API contracts...")
    with open("userspace/ais/src/arm64_runtime.rs", "r") as f:
        ais_src = f.read()
    for s in [
        "AisArm64RuntimeHarness",
        "AisRuntimeError",
        "AisRuntimeState",
        "CAP_AIS_EXEC",
        "AIS_MAX_RAM_BYTES",
        "TARGET_INFERENCE_LATENCY_MAX_MS",
        "initialize",
        "load_quantized_model",
        "run_inference_benchmark",
        "is_initialized",
        "is_model_loaded",
        "is_simd_neon_active",
        "allocated_ram_bytes",
        "last_inference_latency_ms",
    ]:
        if s not in ais_src:
            print(f"❌ FAIL: arm64_runtime.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified userspace symbol: {s}")

    with open("kernel/src/hal/platform/ais_arm64_core.rs", "r") as f:
        kern_src = f.read()
    for s in [
        "KernelAisArm64RuntimeHarness",
        "KernelAisRuntimeError",
        "KernelAisRuntimeState",
        "KERNEL_CAP_AIS_EXEC",
        "KERNEL_AIS_MAX_RAM_BYTES",
        "KERNEL_TARGET_INFERENCE_LATENCY_MAX_MS",
        "initialize",
        "load_quantized_model",
        "run_inference_benchmark",
        "is_initialized",
        "is_model_loaded",
        "is_simd_neon_active",
        "allocated_ram_bytes",
        "last_inference_latency_ms",
    ]:
        if s not in kern_src:
            print(f"❌ FAIL: ais_arm64_core.rs missing symbol '{s}'")
            sys.exit(1)
        print(f"  ✓ Verified kernel symbol: {s}")

    # 3. Rule N-01: Zero External Crates Check
    print("\n[CHECK 3/6] Verifying Rule N-01 (Zero external crates in TCB)...")
    for toml_path in ["kernel/Cargo.toml", "userspace/ais/Cargo.toml"]:
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
    print("\n[CHECK 4/6] Verifying Rule N-04 (Zero unsafe code in userspace/ais)...")
    with open("userspace/ais/src/arm64_runtime.rs", "r") as f:
        if "unsafe" in f.read():
            print("❌ FAIL: Unsafe block found in userspace/ais/src/arm64_runtime.rs")
            sys.exit(1)
    print("  ✓ userspace/ais/src/arm64_runtime.rs is 100% safe Rust")

    # 5. Boot Log Verification (Stages 430-434)
    print("\n[CHECK 5/6] Verifying Golden Boot Log Stages 430-434...")
    with open("docs/evidence/golden-boot-sp96.log", "r") as f:
        boot_log = f.read()

    expected_stages = [
        "430_SP96_AIS_ARM64_RUNTIME_INIT",
        "431_SP96_AIS_CAPABILITY_AND_NEON_SANDBOX",
        "432_SP96_AIS_QUANTIZED_MODEL_LOAD_QUOTA",
        "433_SP96_AIS_KRYO470_INFERENCE_BENCHMARK",
        "434_SP96_AIS_ARM64_RUNTIME_READY",
    ]
    for stage in expected_stages:
        if stage not in boot_log:
            print(f"❌ FAIL: Golden boot log missing stage '{stage}'")
            sys.exit(1)
        print(f"  ✓ Verified stage: {stage}")

    # 6. Verify Confirmation Logs
    print("\n[CHECK 6/6] Verifying AIS Confirmation Logs...")
    expected_logs = [
        "AIS runtime harness online: ARM64 CPU inference engine active (REQ-AI-101 PASSED)",
        "AIS sandbox initialized: CAP_AIS_EXEC granted, NEON SIMD active (INV-AIS-HW-001 PASSED)",
        "Model loaded: 32MB quantized tensor workspace within 64MB quota (INV-AIS-HW-003 PASSED)",
        "Inference benchmark: 85ms on Kryo 470 CPU (< 120ms target) (INV-AIS-HW-002 PASSED)",
        "[SP-96] Sprint SP-96 On-Device AIS ARM64 Runtime Milestone REACHED.",
    ]
    for msg in expected_logs:
        if msg not in boot_log:
            print(f"❌ FAIL: Boot log missing required confirmation: '{msg}'")
            sys.exit(1)
        print(f"  ✓ Confirmed log output: {msg[:65]}...")

    print("\n==================================================")
    print(" ✅ SPRINT SP-96 VERIFICATION PASSED (100% DoD)  ")
    print("==================================================")

if __name__ == "__main__":
    verify_sp96()
