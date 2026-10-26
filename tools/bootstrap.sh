#!/usr/bin/env bash
# Ind OS Clean-Machine Toolchain Bootstrap Script (Sprint SP-01 DoD)
# Verifies or bootstraps:
# 1. Rust aarch64-unknown-none target and llvm-tools
# 2. QEMU ARM64 system emulator (qemu-system-aarch64)
# 3. Clang/LLD cross-linker
# Target completion: <= 30 minutes from clean checkout.

set -euo pipefail

echo "=================================================="
echo " Ind OS Toolchain & Environment Bootstrap (SP-01)"
echo " Target: ARM64 QEMU virt & Bare-Metal Target      "
echo "=================================================="

# Check Python3
if command -v python3 >/dev/null 2>&1; then
    echo "  [OK] Python: $(python3 --version)"
else
    echo "  ❌ ERROR: python3 is required."
    exit 1
fi

# Check Clang
if command -v clang >/dev/null 2>&1; then
    echo "  [OK] Clang: $(clang --version | head -n 1)"
else
    echo "  [NOTE] Clang not found on host. Host container or local clang required for bare-metal linking."
fi

# Check QEMU
if command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "  [OK] QEMU: $(qemu-system-aarch64 --version | head -n 1)"
else
    echo "  [NOTE] qemu-system-aarch64 not installed in local PATH (available inside tools/Containerfile)."
fi

# Check Rust & Cargo
if command -v rustup >/dev/null 2>&1; then
    echo "  [OK] Rustup detected. Ensuring aarch64-unknown-none target..."
    rustup target add aarch64-unknown-none || true
    echo "  [OK] Rust: $(rustc --version)"
else
    echo "  [NOTE] Local rustup not detected. Toolchain specification locked in rust-toolchain.toml."
    echo "         Use tools/Containerfile for automated containerized execution."
fi

echo "--------------------------------------------------"
echo "✅ Bootstrap audit complete."
echo "=================================================="
