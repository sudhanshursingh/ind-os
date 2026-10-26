#!/usr/bin/env bash
# Ind OS Headless QEMU Runner Script (Sprint SP-03)
# Runs IndK bare-metal kernel ELF inside headless QEMU virt environment.
# Parameters:
#   --kernel <path_to_kernel_elf>
#   --log <path_to_serial_output_log>
#   --timeout <seconds> (default: 15)

set -euo pipefail

KERNEL_PATH=""
LOG_PATH=""
TIMEOUT_SECS=15
EXTRA_ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        -k|--kernel)
            KERNEL_PATH="$2"
            shift 2
            ;;
        -l|--log)
            LOG_PATH="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT_SECS="$2"
            shift 2
            ;;
        *)
            EXTRA_ARGS+=("$1")
            shift
            ;;
    esac
done

if [[ -z "$KERNEL_PATH" ]]; then
    echo "❌ ERROR: --kernel <path> is required." >&2
    exit 2
fi

if [[ ! -f "$KERNEL_PATH" ]]; then
    echo "❌ ERROR: Kernel ELF not found: $KERNEL_PATH" >&2
    exit 2
fi

QEMU_BIN="qemu-system-aarch64"
if ! command -v "$QEMU_BIN" >/dev/null 2>&1; then
    echo "❌ ERROR: $QEMU_BIN not found in PATH." >&2
    echo "Please use tools/Containerfile or install QEMU system aarch64." >&2
    exit 127
fi

# QEMU virt configuration ratified in docs/architecture/qemu-boot-path-spike.md
# Direct kernel ELF execution, GICv2, Cortex-A57, 512MB RAM, headless serial on stdio
QEMU_CMD=(
    "$QEMU_BIN"
    -machine virt,gic-version=2
    -cpu cortex-a57
    -smp 1
    -m 512M
    -nographic
    -serial mon:stdio
    -no-reboot
    -kernel "$KERNEL_PATH"
)

if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
    QEMU_CMD+=("${EXTRA_ARGS[@]}")
fi

echo "=================================================="
echo " Starting Headless IndK QEMU Session             "
echo " Kernel:  $KERNEL_PATH"
echo " Timeout: ${TIMEOUT_SECS}s"
if [[ -n "$LOG_PATH" ]]; then
    echo " Log:     $LOG_PATH"
fi
echo "=================================================="

if [[ -n "$LOG_PATH" ]]; then
    mkdir -p "$(dirname "$LOG_PATH")"
    python3 -c "
import subprocess, sys
cmd = sys.argv[1:]
with open('$LOG_PATH', 'w', encoding='utf-8') as out_f:
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        stdout, _ = p.communicate(timeout=$TIMEOUT_SECS)
        out_f.write(stdout)
        sys.stdout.write(stdout)
    except subprocess.TimeoutExpired:
        p.kill()
        sys.stderr.write('\n⚠️ QEMU process terminated after timeout ($TIMEOUT_SECS s).\n')
        sys.exit(124)
sys.exit(p.returncode)
" "${QEMU_CMD[@]}"
else
    "${QEMU_CMD[@]}"
fi
