#!/usr/bin/env bash
# Ind OS Headless QEMU Runner Script (Sprint SP-03 / SP-09)
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

CONTAINER_NAME="indos-qemu-$$-$RANDOM"
export CONTAINER_NAME
trap 'docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true' EXIT INT TERM

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

QEMU_PREFIX=()
if command -v qemu-system-aarch64 >/dev/null 2>&1; then
    QEMU_PREFIX=("qemu-system-aarch64")
elif command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    sync 2>/dev/null || true
    QEMU_PREFIX=(docker run --rm -i --name "$CONTAINER_NAME" -v "${WORKSPACE_ROOT}:/workspace" -w /workspace indos-qemu qemu-system-aarch64)
else
    echo "❌ ERROR: Neither qemu-system-aarch64 nor docker daemon found." >&2
    echo "Please use tools/Containerfile or install QEMU system aarch64." >&2
    exit 127
fi

# If a .bin counterpart exists for an ELF, prefer .bin for direct Linux-style booting
TARGET_IMAGE="$KERNEL_PATH"
if [[ "$KERNEL_PATH" != *.bin && -f "${KERNEL_PATH}.bin" ]]; then
    TARGET_IMAGE="${KERNEL_PATH}.bin"
fi

QEMU_CMD=(
    "${QEMU_PREFIX[@]}"
    -machine virt,gic-version=2
    -cpu cortex-a57
    -smp 1
    -m 512M
    -nographic
    -monitor none
    -serial stdio
    -no-reboot
    -kernel "$TARGET_IMAGE"
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
fi

python3 -c "
import os, subprocess, sys, time, select

cmd = sys.argv[1:]
log_path = '$LOG_PATH'
out_file = open(log_path, 'w', encoding='utf-8') if log_path else None

p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
start_time = time.time()

while True:
    elapsed = time.time() - start_time
    rem_time = max(0.05, $TIMEOUT_SECS - elapsed)
    if elapsed >= $TIMEOUT_SECS:
        break

    rlist, _, _ = select.select([p.stdout], [], [], min(rem_time, 0.2))
    if rlist:
        try:
            chunk = os.read(p.stdout.fileno(), 4096)
        except Exception:
            chunk = b''
        if not chunk:
            if p.poll() is not None:
                break
            time.sleep(0.05)
            continue
        text = chunk.decode('utf-8', errors='replace')
        sys.stdout.write(text)
        sys.stdout.flush()
        if out_file:
            out_file.write(text)
            out_file.flush()
        if 'SP60_COMPOSITOR_READY' in text or '[PANIC]' in text or 'System halted' in text or 'FDT_ERR' in text:
            time.sleep(0.1)
            while True:
                r, _, _ = select.select([p.stdout], [], [], 0.05)
                if r:
                    try:
                        rem = os.read(p.stdout.fileno(), 4096)
                        if not rem:
                            break
                        t_rem = rem.decode('utf-8', errors='replace')
                        sys.stdout.write(t_rem)
                        sys.stdout.flush()
                        if out_file:
                            out_file.write(t_rem)
                            out_file.flush()
                    except Exception:
                        break
                else:
                    break
            break
    elif p.poll() is not None:
        break

if out_file:
    out_file.close()

try:
    p.kill()
    p.wait(timeout=1)
except Exception:
    pass

c_name = os.environ.get('CONTAINER_NAME')
if c_name:
    try:
        subprocess.run(['docker', 'rm', '-f', c_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass

sys.exit(0)
" "${QEMU_CMD[@]}"
