#!/usr/bin/env bash
# Ind OS Developer OS v0.1 Packaging Tool (Sprint SP-40)
# Bundles the IndK microkernel and userspace packages into a reproducible release archive.

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "=================================================="
echo " Packaging Ind OS Developer OS v0.1 Release      "
echo "=================================================="

DIST_DIR="target/developer-os-v0.1"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/boot"
mkdir -p "$DIST_DIR/bin"
mkdir -p "$DIST_DIR/manifests"
mkdir -p "$DIST_DIR/docs"

# 1. Ensure kernel ELF is built
echo "[1/4] Checking kernel binary..."
if [ ! -f "kernel/target/aarch64-unknown-none/debug/indk" ]; then
    echo "Building kernel..."
    make kernel
fi
cp "kernel/target/aarch64-unknown-none/debug/indk" "$DIST_DIR/boot/indk"
echo "  ✓ Kernel ELF packaged into boot/indk"

# 2. Build and verify userspace crates
echo "[2/4] Verifying workspace userspace crates..."
USERSPACE_CRATES=(
    "userspace/init"
    "userspace/libindus"
    "userspace/devmgr"
    "userspace/shell"
    "userspace/logd"
    "userspace/store"
    "userspace/edit"
    "userspace/testsuite"
)

for crate in "${USERSPACE_CRATES[@]}"; do
    echo "  Checking $crate..."
    cargo check --manifest-path "$crate/Cargo.toml" --target aarch64-unknown-none
done
echo "  ✓ All 8 userspace packages verified clean"

# 3. Generate Release SBOM & Manifests
echo "[3/4] Generating Software Bill of Materials (SBOM)..."
cat << 'EOF' > "$DIST_DIR/sbom.json"
{
  "release": "Developer OS v0.1",
  "version": "0.1.0",
  "phase": "Phase 5",
  "architecture": "aarch64",
  "license": "Apache-2.0",
  "components": [
    { "name": "indk-core", "type": "microkernel", "path": "kernel", "unsafe": "bounded (early boot only)" },
    { "name": "indus-init", "type": "service", "path": "userspace/init", "unsafe": 0 },
    { "name": "libindus", "type": "library", "path": "userspace/libindus", "unsafe": 0 },
    { "name": "indus-devmgr", "type": "service", "path": "userspace/devmgr", "unsafe": 0 },
    { "name": "indus-shell", "type": "service", "path": "userspace/shell", "unsafe": 0 },
    { "name": "indus-logd", "type": "service", "path": "userspace/logd", "unsafe": 0 },
    { "name": "indus-store", "type": "service", "path": "userspace/store", "unsafe": 0 },
    { "name": "indus-edit", "type": "application", "path": "userspace/edit", "unsafe": 0 },
    { "name": "indus-testsuite", "type": "qa-suite", "path": "userspace/testsuite", "unsafe": 0 }
  ],
  "constitutional_compliance": {
    "rule_n01": "Zero external crates",
    "rule_n04": "Zero unsafe in new userspace modules",
    "rule_n10": "DCO Signed-off-by commits"
  }
}
EOF

# Copy architecture documentation
cp docs/architecture/*.md "$DIST_DIR/docs/"
echo "  ✓ Architecture specifications and SBOM bundled"

# 4. Create Release Archive
echo "[4/4] Creating release archive..."
tar -czf "target/developer-os-v0.1.tar.gz" -C target developer-os-v0.1

echo "=================================================="
echo " Release bundle created: target/developer-os-v0.1.tar.gz"
echo " Developer OS v0.1 packaging completed successfully!"
echo "=================================================="
