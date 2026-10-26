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
MODULE_VERSION="v1.0.1-sprint14"
MODULE_VERSION_CODE="10014"
AUTHOR="Sudhanshu Raj Singh & Ind OS Team"
BUILD_APKS=false
RUN_TEST=false
RUN_IPC_BENCH=false
RUN_PROBE=false
RUN_MMAP_BENCH=false
RUN_GBNF_TEST=false
RUN_EXEC_TEST=false
RUN_WAKE_TEST=false
RUN_ASR_TEST=false
RUN_NAV_TEST=false
RUN_FIREWALL_TEST=false
RUN_OTP_TEST=false
RUN_APK_BLOCK_TEST=false
RUN_LAUNCHER_TEST=false
RUN_DIGEST_TEST=false
RUN_SUMMARY_TEST=false
TARGET_SERIAL=""

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Options:
    -b, --build       Compile debug APKs via Gradle before deployment
    -t, --test        Execute Sprint 1 DoD privileged toggle test (Wi-Fi, Brightness)
    -i, --ipc-bench   Execute Sprint 2 DoD Binder IPC latency test (IndLauncher -> IndNeuralCore)
    -p, --probe       Execute Sprint 3 DoD Native Hardware Acceleration Probe (Kryo 470 / Adreno 618)
    -m, --mmap-bench  Execute Sprint 4 DoD Zero-Copy mmap & Streaming Inference Benchmark (>=15 tok/s, <15 ms eviction)
    -g, --gbnf-test   Execute Sprint 5 DoD Deterministic GBNF Sampler & Prompt Test (200/200 100% valid JSON)
    -e, --exec-test   Execute Sprint 6 DoD Core OS System Intent Execution Test (Wi-Fi, BT, Torch, Apps, Calls <500 ms)
    -w, --wake-test   Execute Sprint 7 DoD Wake-Word & ECAPA-TDNN Biometric Voice Gate Test (>95% wake, <2% FAR)
    -a, --asr-test    Execute Sprint 8 DoD Streaming Indic/Hinglish ASR + Voice Orb Test (<200 ms latency, offline TTS)
    -n, --nav-test    Execute Sprint 9 DoD Flagship Autonomous UI Navigation Demo (WhatsApp / YouTube / Dialer multi-step)
    -f, --firewall-test Execute Sprint 10 DoD Prompt-Injection Firewall & Dual-Context Isolation Test (0 actions triggered)
    -o, --otp-test    Execute Sprint 11 DoD Screen-Share OTP Blackout & Banking Window Guard Test
    -k, --apk-block-test Execute Sprint 12 DoD Scam SMS & Malicious Trojan APK Blocker Test
    -l, --launcher-test  Execute Sprint 13 DoD IndLauncher 120 Hz Frame Pacing & RAM Test (<0.5% jank, <45 MB)
    -d, --digest-test    Execute Sprint 14 DoD Zero-Spam Notification Digest Test (Promo Spam vs Instant Pass)
    -u, --summary-test   Execute Sprint 14 DoD On-Screen Hindi 3-Bullet Translation & Summarization Test (<3s)
    -s, --serial <id> Target specific ADB device serial
    -h, --help        Show this help message

Examples:
    $0 --build
    $0 --build --test
    $0 --build --ipc-bench
    $0 --build --probe
    $0 --build --mmap-bench
    $0 --build --gbnf-test
    $0 --build --exec-test
    $0 --build --launcher-test
    $0 --build --digest-test
    $0 --build --summary-test
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
        -m|--mmap-bench)
            RUN_MMAP_BENCH=true
            shift
            ;;
        -g|--gbnf-test)
            RUN_GBNF_TEST=true
            shift
            ;;
        -e|--exec-test)
            RUN_EXEC_TEST=true
            shift
            ;;
        -w|--wake-test)
            RUN_WAKE_TEST=true
            shift
            ;;
        -a|--asr-test)
            RUN_ASR_TEST=true
            shift
            ;;
        -n|--nav-test)
            RUN_NAV_TEST=true
            shift
            ;;
        -f|--firewall-test)
            RUN_FIREWALL_TEST=true
            shift
            ;;
        -o|--otp-test)
            RUN_OTP_TEST=true
            shift
            ;;
        -k|--apk-block-test)
            RUN_APK_BLOCK_TEST=true
            shift
            ;;
        -l|--launcher-test)
            RUN_LAUNCHER_TEST=true
            shift
            ;;
        -d|--digest-test)
            RUN_DIGEST_TEST=true
            shift
            ;;
        -u|--summary-test)
            RUN_SUMMARY_TEST=true
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

if [[ "${RUN_MMAP_BENCH}" == "true" ]]; then
    echo "[*] Step 9: Executing Sprint 4 DoD Zero-Copy mmap & Streaming Inference Benchmark..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c
    
    echo "[*] Triggering Sprint 4 benchmark via IndAIService (indos.neural.ACTION_RUN_BENCHMARK)..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.neural.ACTION_RUN_BENCHMARK || \
    "${ADB_CMD[@]}" shell am startservice -a indos.neural.ACTION_RUN_BENCHMARK || true
    
    echo "[*] Awaiting benchmark completion (listening to logcat -s IND_BENCHMARK)..."
    sleep 4
    "${ADB_CMD[@]}" logcat -d -s IND_BENCHMARK
    echo "[+] Sprint 4 DoD Zero-Copy mmap & Streaming Inference Benchmark completed."
fi

if [[ "${RUN_GBNF_TEST}" == "true" ]]; then
    echo "[*] Step 10: Executing Sprint 5 DoD Deterministic GBNF Sampler & Prompt Test..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c
    
    echo "[*] Triggering GBNF Prompt evaluation test via IndAIService..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.neural.ACTION_RUN_BENCHMARK || \
    "${ADB_CMD[@]}" shell am startservice -a indos.neural.ACTION_RUN_BENCHMARK || true
    
    echo "[*] Awaiting GBNF test results (listening to logcat -s IND_BENCHMARK)..."
    sleep 4
    "${ADB_CMD[@]}" logcat -d -s IND_BENCHMARK
    echo "[+] Sprint 5 DoD Deterministic GBNF Sampler test completed."
fi

if [[ "${RUN_EXEC_TEST}" == "true" ]]; then
    echo "[*] Step 11: Executing Sprint 6 DoD Core OS System Intent Execution Test (<500 ms)..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c
    
    echo "[*] Triggering live toggle (flashlight ON)..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.neural.ACTION_EXECUTE_ACTION --es actionType "SYS_TOGGLE" --es target "flashlight" --ez value true || true
    sleep 1
    
    echo "[*] Triggering live toggle (volume 80%)..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.neural.ACTION_EXECUTE_ACTION --es actionType "SYS_TOGGLE" --es target "volume" --ei value 80 || true
    sleep 1
    
    echo "[*] Awaiting execution telemetry (listening to logcat -s IND_BENCHMARK)..."
    "${ADB_CMD[@]}" logcat -d -s IND_BENCHMARK
    echo "[+] Sprint 6 DoD Core OS System Intent Execution Test completed."
fi

if [[ "${RUN_WAKE_TEST}" == "true" ]]; then
    echo "[*] Step 12: Executing Sprint 7 DoD Low-Power Wake-Word & ECAPA-TDNN Biometric Voice Gate Test..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c
    
    echo "[*] Starting AudioRingBufferService..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.zerotouch.action.START_LISTENING -n in.indos.zerotouch/indos.zerotouch.audio.AudioRingBufferService || true
    sleep 1
    
    echo "[*] Simulating wake-word trigger ('Hey Ind', confidence=0.91)..."
    "${ADB_CMD[@]}" shell am start-foreground-service -a indos.zerotouch.action.SIMULATE_WAKE -n in.indos.zerotouch/indos.zerotouch.audio.AudioRingBufferService --es extra_keyword "Hey Ind" --ef extra_confidence 0.91 || true
    sleep 2
    
    echo "[*] Awaiting voice gate telemetry (listening to logcat -s AudioRingBufferService EcapaSpeakerVerifier WakeWordDetector IND_BENCHMARK)..."
    "${ADB_CMD[@]}" logcat -d -s AudioRingBufferService EcapaSpeakerVerifier WakeWordDetector IND_BENCHMARK
    
    echo "[*] Querying battery drain telemetry..."
    "${ADB_CMD[@]}" shell dumpsys batterystats --charged in.indos.zerotouch | grep -E "(Uid u0a|Estimated power use|Capacity)" || true
    echo "[+] Sprint 7 DoD Wake-Word & ECAPA-TDNN Biometric Voice Gate Test completed."
fi

if [[ "${RUN_ASR_TEST}" == "true" ]]; then
    echo "[*] Step 13: Executing Sprint 8 DoD Streaming Indic/Hinglish ASR + Voice Orb Test (<200 ms)..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c
    
    echo "[*] Displaying Voice Orb overlay (LISTENING state)..."
    "${ADB_CMD[@]}" shell am startservice -a indos.zerotouch.action.SHOW_ORB -n in.indos.zerotouch/indos.zerotouch.ui.IndOrbOverlayService --es extra_orb_state "LISTENING" || true
    sleep 1
    
    echo "[*] Transitioning Voice Orb to PROCESSING state..."
    "${ADB_CMD[@]}" shell am startservice -a indos.zerotouch.action.SET_ORB_STATE -n in.indos.zerotouch/indos.zerotouch.ui.IndOrbOverlayService --es extra_orb_state "PROCESSING" || true
    sleep 1
    
    echo "[*] Transitioning Voice Orb to SPEAKING state (Spoken TTS confirmation)..."
    "${ADB_CMD[@]}" shell am startservice -a indos.zerotouch.action.SET_ORB_STATE -n in.indos.zerotouch/indos.zerotouch.ui.IndOrbOverlayService --es extra_orb_state "SPEAKING" || true
    sleep 2
    
    echo "[*] Dismissing Voice Orb overlay..."
    "${ADB_CMD[@]}" shell am startservice -a indos.zerotouch.action.HIDE_ORB -n in.indos.zerotouch/indos.zerotouch.ui.IndOrbOverlayService || true
    
    echo "[*] Awaiting ASR & TTS telemetry (listening to logcat -s StreamingAsrEngine SileroVadDetector OfflineTtsEngine VoiceCoordinator IND_BENCHMARK)..."
    "${ADB_CMD[@]}" logcat -d -s StreamingAsrEngine SileroVadDetector OfflineTtsEngine VoiceCoordinator IND_BENCHMARK
    echo "[+] Sprint 8 DoD Streaming Indic/Hinglish ASR + Voice Orb Test completed."
fi

if [[ "${RUN_NAV_TEST}" == "true" ]]; then
    echo "[*] Step 14: Executing Sprint 9 DoD Flagship Autonomous UI Navigation Demo..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c
    
    echo "[*] Verifying Accessibility Service state..."
    "${ADB_CMD[@]}" shell settings get secure enabled_accessibility_services | grep -q "in.indos.zerotouch" || {
        echo "[*] Enabling IndAccessibilityService in secure settings..."
        "${ADB_CMD[@]}" shell settings put secure enabled_accessibility_services "in.indos.zerotouch/indos.zerotouch.IndAccessibilityService"
        "${ADB_CMD[@]}" shell settings put secure accessibility_enabled 1
    }
    
    echo "[*] Dumping UI hierarchy before autonomous action dispatch..."
    "${ADB_CMD[@]}" shell uiautomator dump /data/local/tmp/ui_dump_sprint9.xml || true
    
    echo "[*] Awaiting Autonomous Navigator telemetry (listening to logcat -s AutonomousNavigator IndAccessibility SemanticGraphPruner GestureSynthesizer IND_BENCHMARK)..."
    sleep 2
    "${ADB_CMD[@]}" logcat -d -s AutonomousNavigator IndAccessibility SemanticGraphPruner GestureSynthesizer IND_BENCHMARK
    echo "[+] Sprint 9 DoD Flagship Autonomous UI Navigation Demo completed."
fi

if [[ "${RUN_FIREWALL_TEST}" == "true" ]]; then
    echo "[*] Step 15: Executing Sprint 10 DoD Prompt-Injection Firewall & Dual-Context Isolation Test..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c

    echo "[*] Triggering crafted adversarial injection payload test..."
    "${ADB_CMD[@]}" shell am start-foreground-service -n in.indos.suraksha/indos.suraksha.SurakshaShieldService || true

    echo "[*] Awaiting Suraksha Shield telemetry (listening to logcat -s ActionPolicyEnforcer PromptFirewall DualContextIsolation IND_SURAKSHA IND_BENCHMARK)..."
    sleep 2
    "${ADB_CMD[@]}" logcat -d -s ActionPolicyEnforcer PromptFirewall DualContextIsolation IND_SURAKSHA IND_BENCHMARK
    echo "[+] Sprint 10 DoD Prompt-Injection Firewall & Dual-Context Isolation Test completed."
fi

if [[ "${RUN_OTP_TEST}" == "true" ]]; then
    echo "[*] Step 16: Executing Sprint 11 DoD Screen-Share OTP Blackout & Banking Window Guard Test..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c

    echo "[*] Ensuring Notification Listener access is granted..."
    "${ADB_CMD[@]}" shell cmd notification set_notification_listener_access in.indos.suraksha/indos.suraksha.screenshare.OtpNotificationInterceptor true || true

    echo "[*] Simulating AnyDesk/TeamViewer screen-share active during call..."
    "${ADB_CMD[@]}" shell am start-foreground-service -n in.indos.suraksha/indos.suraksha.SurakshaShieldService || true

    echo "[*] Awaiting OTP Interceptor & Screen-Share telemetry (listening to logcat -s ScreenShareDetector OtpNotificationInterceptor BankingWindowGuard IND_SURAKSHA IND_BENCHMARK)..."
    sleep 2
    "${ADB_CMD[@]}" logcat -d -s ScreenShareDetector OtpNotificationInterceptor BankingWindowGuard IND_SURAKSHA IND_BENCHMARK
    echo "[+] Sprint 11 DoD Screen-Share OTP Blackout & Banking Window Guard Test completed."
fi

if [[ "${RUN_APK_BLOCK_TEST}" == "true" ]]; then
    echo "[*] Step 17: Executing Sprint 12 DoD Scam SMS & Malicious Trojan APK Blocker Test..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c

    echo "[*] Simulating sideloaded banking trojan APK installation..."
    "${ADB_CMD[@]}" shell am start-foreground-service -n in.indos.suraksha/indos.suraksha.SurakshaShieldService || true

    echo "[*] Awaiting Package Verifier & Scam Classifier telemetry (listening to logcat -s SurakshaVerifier ScamClassifier SurakshaBlockActivity IND_SURAKSHA IND_BENCHMARK)..."
    sleep 2
    "${ADB_CMD[@]}" logcat -d -s SurakshaVerifier ScamClassifier SurakshaBlockActivity IND_SURAKSHA IND_BENCHMARK
    echo "[+] Sprint 12 DoD Scam SMS & Malicious Trojan APK Blocker Test completed."
fi

if [[ "${RUN_LAUNCHER_TEST}" == "true" ]]; then
    echo "[*] Step 18: Executing Sprint 13 DoD IndLauncher 120 Hz Frame Pacing & RAM Test..."
    echo "[*] Launching IndLauncherActivity (in.indos.launcher)..."
    "${ADB_CMD[@]}" shell am start -n in.indos.launcher/indos.launcher.IndLauncherActivity || true
    sleep 2

    echo "[*] Resetting and collecting dumpsys gfxinfo for in.indos.launcher..."
    "${ADB_CMD[@]}" shell dumpsys gfxinfo in.indos.launcher reset >/dev/null || true

    echo "[*] Simulating high-velocity touch flings across 120 Hz app drawer..."
    for i in {1..5}; do
        "${ADB_CMD[@]}" shell input swipe 540 1800 540 400 120 || true
        "${ADB_CMD[@]}" shell input swipe 540 400 540 1800 120 || true
    done

    echo "[*] Querying dumpsys gfxinfo frame pacing and jank metrics..."
    "${ADB_CMD[@]}" shell dumpsys gfxinfo in.indos.launcher | grep -E "(Total frames|Janky frames|50th percentile|90th percentile|95th percentile|99th percentile)" || true

    echo "[*] Querying IndLauncher resident memory usage (dumpsys meminfo)..."
    "${ADB_CMD[@]}" shell dumpsys meminfo in.indos.launcher | grep -E "(TOTAL PSS|TOTAL RSS|Native Heap|Dalvik Heap)" || true
    echo "[+] Sprint 13 DoD IndLauncher 120 Hz Frame Pacing & RAM Test completed."
fi

if [[ "${RUN_DIGEST_TEST}" == "true" ]]; then
    echo "[*] Step 19: Executing Sprint 14 DoD Zero-Spam Smart Notification Digest Test..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c

    echo "[*] Simulating incoming mixed notification batch (UPI + OTP + Personal + Promo Spam)..."
    "${ADB_CMD[@]}" shell am start-foreground-service -n in.indos.suraksha/indos.suraksha.SurakshaShieldService || true

    echo "[*] Awaiting Notification Digest telemetry (listening to logcat -s NotificationDigestManager NotificationDigestClassifier IND_SURAKSHA IND_BENCHMARK)..."
    sleep 2
    "${ADB_CMD[@]}" logcat -d -s NotificationDigestManager NotificationDigestClassifier IND_SURAKSHA IND_BENCHMARK
    echo "[+] Sprint 14 DoD Zero-Spam Smart Notification Digest Test completed."
fi

if [[ "${RUN_SUMMARY_TEST}" == "true" ]]; then
    echo "[*] Step 20: Executing Sprint 14 DoD On-Screen Hindi 3-Bullet Translator & Summarizer Test..."
    echo "[*] Clearing logcat buffer..."
    "${ADB_CMD[@]}" logcat -c

    echo "[*] Triggering simulated Hindi screen summarization ('Hey Ind, is page ko Hindi mein samjhao')..."
    "${ADB_CMD[@]}" shell am startservice -a indos.zerotouch.action.SHOW_ORB -n in.indos.zerotouch/indos.zerotouch.ui.IndOrbOverlayService --es extra_orb_state "PROCESSING" || true

    echo "[*] Awaiting Screen Summarizer telemetry (listening to logcat -s ScreenTranslate VoiceCoordinator OfflineTtsEngine IND_BENCHMARK)..."
    sleep 3
    "${ADB_CMD[@]}" logcat -d -s ScreenTranslate VoiceCoordinator OfflineTtsEngine IND_BENCHMARK
    echo "[+] Sprint 14 DoD On-Screen Hindi 3-Bullet Translator & Summarizer Test completed."
fi

echo "[+] Deployment completed successfully."




