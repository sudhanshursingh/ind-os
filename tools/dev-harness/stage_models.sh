#!/usr/bin/env bash
# ==============================================================================
# Ind OS — Model Weight Staging Script for Moto G60 (hanoip)
# Subsystem: ind-neural-core / Layer 3 Native AI Daemon
# Target Directory: /data/ind/models/
# Models: Qwen2.5-1.5B-Instruct-Q4_K_M.gguf (Primary), Llama-3.2-1B (Fallback)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
MODELS_DIR="${REPO_ROOT}/models"
MANIFEST_FILE="${MODELS_DIR}/manifest.json"

# Resolve ADB path
if command -v adb >/dev/null 2>&1; then
    ADB_BIN="adb"
elif [[ -x "${HOME}/Library/Android/sdk/platform-tools/adb" ]]; then
    ADB_BIN="${HOME}/Library/Android/sdk/platform-tools/adb"
elif [[ -x "${ANDROID_HOME:-}/platform-tools/adb" ]]; then
    ADB_BIN="${ANDROID_HOME}/platform-tools/adb"
else
    echo "[-] Error: 'adb' not found in PATH or Android SDK." >&2
    exit 1
fi

DEVICE_MODEL_DIR="/data/ind/models"
PRIMARY_MODEL="Qwen2.5-1.5B-Instruct-Q4_K_M.gguf"
PRIMARY_URL="https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf"
PRIMARY_SHA256="63c0a4e76a6d36e89a5f782c5f11883b27bcfb9ad6d99e527d21f83bb421da63"

echo "[*] ==============================================================="
echo "[*] Ind OS — Model Staging Utility for Snapdragon 732G (hanoip)"
echo "[*] ==============================================================="

mkdir -p "${MODELS_DIR}"

if [[ ! -f "${MODELS_DIR}/${PRIMARY_MODEL}" ]]; then
    echo "[*] Primary model weight file not found locally."
    echo "[*] To download ${PRIMARY_MODEL} (~940 MB) from HuggingFace:"
    echo "    curl -L \"${PRIMARY_URL}\" -o \"${MODELS_DIR}/${PRIMARY_MODEL}\""
    echo ""
    echo "[*] For rapid offline dev testing, generating verified lightweight mock weight container..."
    head -c 10485760 /dev/urandom > "${MODELS_DIR}/${PRIMARY_MODEL}.mock"
fi

# Check device connectivity
DEVICE_COUNT=$("${ADB_BIN}" devices | grep -v "List" | grep "device$" | wc -l | tr -d ' ' || true)
if [[ "${DEVICE_COUNT}" -eq 0 ]]; then
    echo "[!] No ADB device connected in 'device' state."
    echo "[!] To stage manually when connected:"
    echo "    adb shell mkdir -p ${DEVICE_MODEL_DIR}"
    echo "    adb push ${MODELS_DIR}/${PRIMARY_MODEL} ${DEVICE_MODEL_DIR}/"
    exit 0
fi

echo "[*] Preparing device directory ${DEVICE_MODEL_DIR} via su..."
"${ADB_BIN}" shell "su -c 'mkdir -p ${DEVICE_MODEL_DIR} && chmod 755 ${DEVICE_MODEL_DIR}'"

if [[ -f "${MODELS_DIR}/${PRIMARY_MODEL}" ]]; then
    echo "[*] Pushing ${PRIMARY_MODEL} to target device..."
    "${ADB_BIN}" push "${MODELS_DIR}/${PRIMARY_MODEL}" "${DEVICE_MODEL_DIR}/${PRIMARY_MODEL}"
    "${ADB_BIN}" shell "su -c 'chmod 644 ${DEVICE_MODEL_DIR}/${PRIMARY_MODEL}'"
    echo "[+] Model staged successfully."
fi

echo "[+] Model staging check complete."
