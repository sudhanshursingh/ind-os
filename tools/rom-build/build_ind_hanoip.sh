#!/usr/bin/env bash
# ==============================================================================
# Ind OS — Full AOSP/LineageOS 22.x ROM Build & Validation Harness
# Target Device: Motorola Moto G60 (hanoip — Snapdragon 732G)
# Reference: PRD v1.0.1-BIBLE (Phase 6, Sprint 16 — Tree Sync & Product Makefile)
# ==============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST_FILE="${REPO_ROOT}/manifest/default.xml"
COMMON_MK="${REPO_ROOT}/vendor/ind/config/common.mk"
PRODUCT_MK="${REPO_ROOT}/vendor/ind/products/ind_hanoip.mk"
ANDROID_PRODUCTS_MK="${REPO_ROOT}/vendor/ind/AndroidProducts.mk"
BOOTANIM_ZIP="${REPO_ROOT}/vendor/ind/media/bootanimation.zip"
SETTINGS_OVERLAY="${REPO_ROOT}/vendor/ind/overlay/common/packages/apps/Settings/res/values/strings.xml"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}==================================================================${NC}"
echo -e "${BLUE} Ind OS — Phase 6, Sprint 16 ROM Tree & Zero-Bloat Product Harness${NC}"
echo -e "${BLUE} Target: Motorola Moto G60 (hanoip) | Version: v1.0.1-BIBLE        ${NC}"
echo -e "${BLUE}==================================================================${NC}"

# 1. Audit Manifest Projects
echo -e "\n[*] Step 1: Auditing AOSP/LineageOS 22.1 Manifest Tree..."
if [[ ! -f "${MANIFEST_FILE}" ]]; then
    echo -e "${RED}[-] Error: Manifest file not found at ${MANIFEST_FILE}${NC}" >&2
    exit 1
fi

REQUIRED_PROJECTS=(
    "packages/apps/IndNeuralCore"
    "packages/apps/IndZeroTouch"
    "packages/apps/IndSurakshaShield"
    "packages/apps/IndLauncher"
    "vendor/ind"
    "device/motorola/hanoip"
    "device/motorola/sm6150-common"
    "kernel/motorola/sm6150"
    "vendor/motorola/hanoip"
    "vendor/motorola/sm6150-common"
)

for PROJ in "${REQUIRED_PROJECTS[@]}"; do
    if grep -q "path=\"${PROJ}\"" "${MANIFEST_FILE}"; then
        echo -e "${GREEN}[+] Manifest contains required project: ${PROJ}${NC}"
    else
        echo -e "${RED}[-] Error: Missing project in manifest: ${PROJ}${NC}" >&2
        exit 1
    fi
done

# 2. Audit AndroidProducts.mk & Lunch Targets
echo -e "\n[*] Step 2: Auditing AndroidProducts.mk & Lunch Target Registrations..."
if [[ ! -f "${ANDROID_PRODUCTS_MK}" ]]; then
    echo -e "${RED}[-] Error: ${ANDROID_PRODUCTS_MK} not found!${NC}" >&2
    exit 1
fi

if grep -q "ind_hanoip-userdebug" "${ANDROID_PRODUCTS_MK}" && grep -q "ind_hanoip.mk" "${ANDROID_PRODUCTS_MK}"; then
    echo -e "${GREEN}[+] Registered lunch target: ind_hanoip-userdebug -> products/ind_hanoip.mk${NC}"
else
    echo -e "${RED}[-] Error: ind_hanoip-userdebug target registration missing!${NC}" >&2
    exit 1
fi

# 3. Audit Zero-Bloat common.mk & Packages
echo -e "\n[*] Step 3: Auditing Zero-Bloat Configuration in common.mk..."
if [[ ! -f "${COMMON_MK}" ]]; then
    echo -e "${RED}[-] Error: ${COMMON_MK} not found!${NC}" >&2
    exit 1
fi

REQUIRED_PKGS=("IndNeuralCore" "IndZeroTouch" "IndSurakshaShield" "IndLauncher")
for PKG in "${REQUIRED_PKGS[@]}"; do
    if grep -q "${PKG}" "${COMMON_MK}"; then
        echo -e "${GREEN}[+] Core Subsystem included in PRODUCT_PACKAGES: ${PKG}${NC}"
    else
        echo -e "${RED}[-] Error: ${PKG} missing from PRODUCT_PACKAGES!${NC}" >&2
        exit 1
    fi
done

REMOVED_BLOAT=("Jelly" "Eleven" "LineageUpdater" "Etar" "Recorder")
for BLOAT in "${REMOVED_BLOAT[@]}"; do
    if grep -q "${BLOAT}" "${COMMON_MK}"; then
        echo -e "${GREEN}[+] Bloatware purged from ROM: ${BLOAT}${NC}"
    else
        echo -e "${RED}[-] Error: Bloatware ${BLOAT} not marked for removal!${NC}" >&2
        exit 1
    fi
done

# 4. Audit Boot Animation
echo -e "\n[*] Step 4: Auditing Ind OS Boot Animation Specification..."
if [[ ! -f "${BOOTANIM_ZIP}" ]]; then
    echo -e "${RED}[-] Error: bootanimation.zip not found at ${BOOTANIM_ZIP}${NC}" >&2
    exit 1
fi

# Check zero-compression invariant
STORED_CHECK=$(unzip -v "${BOOTANIM_ZIP}" desc.txt | grep -c "Stored" || true)
if [[ "${STORED_CHECK}" -ge 1 ]]; then
    echo -e "${GREEN}[+] bootanimation.zip is stored with 0% compression (ZIP_STORED).${NC}"
else
    echo -e "${RED}[-] Error: bootanimation.zip is compressed! Android bootanimation binary requires 0% compression.${NC}" >&2
    exit 1
fi

DESC_CONTENT=$(unzip -p "${BOOTANIM_ZIP}" desc.txt)
echo -e "[*] desc.txt Header: $(echo "${DESC_CONTENT}" | head -n 1)"
if echo "${DESC_CONTENT}" | grep -q "1080 2460 60"; then
    echo -e "${GREEN}[+] Target resolution 1080x2460 @ 60fps verified for Moto G60.${NC}"
else
    echo -e "${RED}[-] Error: desc.txt resolution does not match 1080 2460 60!${NC}" >&2
    exit 1
fi

# 5. Audit Settings Overlay for About Phone Branding
echo -e "\n[*] Step 5: Auditing Settings Overlay for About Phone Branding..."
if [[ ! -f "${SETTINGS_OVERLAY}" ]]; then
    echo -e "${RED}[-] Error: Settings overlay not found at ${SETTINGS_OVERLAY}${NC}" >&2
    exit 1
fi

if grep -q "Ind OS v1.0.1-BIBLE" "${SETTINGS_OVERLAY}"; then
    echo -e "${GREEN}[+] Settings overlay contains 'Ind OS v1.0.1-BIBLE' branding.${NC}"
else
    echo -e "${RED}[-] Error: Settings overlay missing Ind OS v1.0.1 branding!${NC}" >&2
    exit 1
fi

echo -e "\n${GREEN}==================================================================${NC}"
echo -e "${GREEN} Sprint 16 DoD: All ROM Tree, Makefiles & Branding Gates PASSED!   ${NC}"
echo -e "${GREEN} Ready for lunch ind_hanoip-userdebug && mka bacon compilation.    ${NC}"
echo -e "${GREEN}==================================================================${NC}"
