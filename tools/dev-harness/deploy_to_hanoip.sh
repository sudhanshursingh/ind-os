#!/usr/bin/env bash
# ==============================================================================
# Ind OS — Moto G60 (hanoip) Rapid Dev & Privileged Deploy Harness
# Version: 1.0.0
# Reference Target: Motorola Moto G60 (hanoip - SM7150-AC, Snapdragon 732G)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Resolve ADB path
if command -v adb >/dev/null 2>&1; then
    ADB_BIN="adb"
elif [[ -x "${HOME}/Library/Android/sdk/platform-tools/adb" ]]; then
    ADB_BIN="${HOME}/Library/Android/sdk/platform-tools/adb"
elif [[ -x "${ANDROID_HOME:-}/platform-tools/adb" ]]; then
    ADB_BIN="${ANDROID_HOME}/platform-tools/adb"
else
    echo "[-] Error: 'adb' not found in PATH, ~/Library/Android/sdk/platform-tools/adb, or ANDROID_HOME." >&2
    exit 1
fi

MODULE_ID="ind_os_system_overlay"
MODULE_NAME="Ind OS Privileged Subsystems Overlay"
MODULE_VERSION="v1.0.1-sprint3"
MODULE_VERSION_CODE="10003"
AUTHOR="Sudhanshu Raj Singh & Ind OS Team"
BUILD_APKS=false
RUN_TEST=false
RUN_IPC_BENCH=false
RUN_PROBE=false
TARGET_SERIAL=""

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Options:
    -b, --build       Compile debug APKs via Gradle before deployment
    -t, --test        Execute Sprint 1 DoD privileged toggle test (Wi-Fi, Brightness)
    -i, --ipc-bench   Execute Sprint 2 DoD Binder IPC latency test (IndLauncher -> IndNeuralCore)
    -p, --probe       Execute Sprint 3 DoD Native Hardware Acceleration Probe (Kryo 470 / Adreno 618)
    -s, --serial <id> Target specific ADB device serial
    -h, --help        Show this help message

Examples:
    $0 --build
    $0 --build --test
    $0 --build --ipc-bench
    $0 --build --probe
EOF
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -b|--build)
            BUILD_APKS=true
            shift
            ;;
        -t|--test)
            RUN_TEST=true
            shift
            ;;
        -i|--ipc-bench)
            RUN_IPC_BENCH=true
            shift
            ;;
        -p|--probe)
            RUN_PROBE=true
            shift
            ;;
        -s|--serial)
            TARGET_SERIAL="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "[-] Unknown option: $1" >&2
            usage
            ;;
    esac
done

ADB_CMD=("${ADB_BIN}")
if [[ -n "${TARGET_SERIAL}" ]]; then
    ADB_CMD+=("-s" "${TARGET_SERIAL}")
fi

echo "[*] ==========================================================="
echo "[*] Ind OS Rapid Deploy Harness — Target: Moto G60 (hanoip)"
echo "[*] ==========================================================="

# Build APKs if requested
if [[ "${BUILD_APKS}" == "true" ]]; then
    echo "[*] Step 1: Compiling Ind OS Core Privileged Packages..."
    (cd "${REPO_ROOT}" && ./gradlew assembleDebug)
fi

# Ensure device is connected
echo "[*] Step 2: Checking ADB target device connectivity..."
DEVICE_COUNT=$("${ADB_CMD[@]}" devices | grep -v "List" | grep "device$" | wc -l | tr -d ' ' || true)

if [[ "${DEVICE_COUNT}" -eq 0 ]]; then
    echo "[!] No ADB device connected in 'device' state."
    echo "[!] Showing current devices:"
    "${ADB_CMD[@]}" devices
    echo ""
    echo "[!] Offline mode: Generating local Magisk/KernelSU deployable module ZIP in build/artifacts/..."
    GENERATE_LOCAL_ZIP_ONLY=true
else
    GENERATE_LOCAL_ZIP_ONLY=false
    DEVICE_MODEL=$("${ADB_CMD[@]}" shell getprop ro.product.model 2>/dev/null || echo "Unknown")
    DEVICE_CODENAME=$("${ADB_CMD[@]}" shell getprop ro.product.device 2>/dev/null || echo "Unknown")
    echo "[+] Target Connected: ${DEVICE_MODEL} (${DEVICE_CODENAME})"
fi

# Prepare staging overlay directory
WORK_DIR="${REPO_ROOT}/build/dev-harness/module_staging"
MODULE_ZIP="${REPO_ROOT}/build/artifacts/${MODULE_ID}.zip"
rm -rf "${WORK_DIR}"
mkdir -p "${WORK_DIR}/system/system_ext/priv-app"
mkdir -p "${WORK_DIR}/system/etc/permissions"
mkdir -p "$(dirname "${MODULE_ZIP}")"

echo "[*] Step 3: Assembling Magisk / KernelSU systemless overlay..."

# Write module.prop
cat <<EOF > "${WORK_DIR}/module.prop"
id=${MODULE_ID}
name=${MODULE_NAME}
version=${MODULE_VERSION}
versionCode=${MODULE_VERSION_CODE}
author=${AUTHOR}
description=Systemless /system_ext/priv-app overlay for Ind OS on Moto G60 (hanoip). Grants privileged whitelisted permissions.
EOF

# Copy privapp permissions XML
PRIVAPP_XML="${REPO_ROOT}/vendor/ind/permissions/privapp-permissions-ind.xml"
if [[ -f "${PRIVAPP_XML}" ]]; then
    cp "${PRIVAPP_XML}" "${WORK_DIR}/system/etc/permissions/privapp-permissions-ind.xml"
    echo "[+] Copied privapp-permissions-ind.xml"
else
    echo "[-] Error: ${PRIVAPP_XML} not found!" >&2
    exit 1
fi

# Package apps into staging
PACKAGES=("IndNeuralCore" "IndZeroTouch" "IndSurakshaShield" "IndLauncher" "IndDiagnosticClient")
for PKG in "${PACKAGES[@]}"; do
    TARGET_DIR="${WORK_DIR}/system/system_ext/priv-app/${PKG}"
    mkdir -p "${TARGET_DIR}"
    
    APK_CANDIDATE="${REPO_ROOT}/packages/${PKG}/build/outputs/apk/debug/${PKG}-debug.apk"
    if [[ -f "${APK_CANDIDATE}" ]]; then
        cp "${APK_CANDIDATE}" "${TARGET_DIR}/${PKG}.apk"
        echo "[+] Staged APK: ${PKG}.apk"
    else
        echo "[!] APK candidate not found at: ${APK_CANDIDATE}. Creating placeholder container for module assembly."
        touch "${TARGET_DIR}/${PKG}.apk"
    fi
done

# Create ZIP artifact
(cd "${WORK_DIR}" && zip -r -q "${MODULE_ZIP}" .)
echo "[+] Module package built at: ${MODULE_ZIP}"

if [[ "${GENERATE_LOCAL_ZIP_ONLY}" == "true" ]]; then
    echo "[*] Build completed in offline mode. Deploy to Moto G60 via:"
    echo "    adb push ${MODULE_ZIP} /data/local/tmp/"
    echo "    adb shell su -c 'magisk --install-module /data/local/tmp/${MODULE_ID}.zip'"
    exit 0
fi

# Deploying to live device
echo "[*] Step 4: Pushing systemless overlay to device..."
"${ADB_CMD[@]}" push "${MODULE_ZIP}" "/data/local/tmp/${MODULE_ID}.zip"

# Attempt direct injection to /data/adb/modules if rooted, or magisk install
echo "[*] Step 5: Installing module via su..."
"${ADB_CMD[@]}" shell "su -c 'magisk --install-module /data/local/tmp/${MODULE_ID}.zip || ksu_install /data/local/tmp/${MODULE_ID}.zip || (mkdir -p /data/adb/modules/${MODULE_ID} && unzip -o /data/local/tmp/${MODULE_ID}.zip -d /data/adb/modules/${MODULE_ID})'"

echo "[+] Overlay installed into systemless priv-app hierarchy."

if [[ "${RUN_TEST}" == "true" ]]; then
    echo "[*] Step 6: Executing Sprint 1 DoD Verification Test..."
    echo "[*] Testing Wi-Fi toggle without permission prompts..."
    INITIAL_WIFI=$("${ADB_CMD[@]}" shell settings get global wifi_on)
    echo "[*] Current wifi_on state: ${INITIAL_WIFI}"
    
    # Toggle Wi-Fi via Ind OS Service
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.neural.ACTION_TEST_TOGGLE --es toggle "wifi" || true
    sleep 2
    NEW_WIFI=$("${ADB_CMD[@]}" shell settings get global wifi_on)
    echo "[*] wifi_on state after toggle: ${NEW_WIFI}"
    
    echo "[*] Testing brightness toggle..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.neural.ACTION_TEST_TOGGLE --es toggle "brightness" --ei value 180 || true
    sleep 1
    BRIGHTNESS=$("${ADB_CMD[@]}" shell settings get system screen_brightness)
    echo "[*] screen_brightness: ${BRIGHTNESS}"
    
    echo "[+] Sprint 1 DoD verification finished on device."
fi

if [[ "${RUN_IPC_BENCH}" == "true" ]]; then
    echo "[*] Step 7: Executing Sprint 2 DoD Binder IPC Latency Benchmark..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c
    
    echo "[*] Triggering 1,000 round-trip calls (IndLauncher -> IndNeuralCore)..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.launcher.ACTION_RUN_BENCHMARK || \
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.diagnostic.ACTION_RUN_BENCHMARK || true
    
    echo "[*] Awaiting benchmark completion (listening to logcat -s IND_IPC_BENCH)..."
    sleep 3
    "${ADB_CMD[@]}" logcat -d -s IND_IPC_BENCH
    echo "[+] Sprint 2 DoD Binder IPC latency benchmark completed."
fi

if [[ "${RUN_PROBE}" == "true" ]]; then
    echo "[*] Step 8: Executing Sprint 3 DoD Native Hardware Acceleration Probe..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c
    
    echo "[*] Triggering hardware probe via IndAIService (indos.neural.ACTION_RUN_PROBE)..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.neural.ACTION_RUN_PROBE || \
    "${ADB_CMD[@]}" shell am startservice -a indos.neural.ACTION_RUN_PROBE || true
    
    echo "[*] Awaiting probe completion (listening to logcat -s IND_NATIVE)..."
    sleep 3
    "${ADB_CMD[@]}" logcat -d -s IND_NATIVE
    echo "[+] Sprint 3 DoD Native Hardware Acceleration Probe completed."
fi

echo "[+] Deployment completed successfully."

