#!/usr/bin/env bash
# Ind OS Virtual Device Simulator Launcher
# Starts a local web server hosting the Motorola Moto G60 Virtual Device Emulator.

set -euo pipefail

PORT="${1:-8085}"
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "=================================================="
echo " Starting Ind OS Moto G60 Virtual Device Simulator"
echo " URL: http://localhost:${PORT}"
echo "=================================================="

# Open default browser on macOS if available
if command -v open >/dev/null 2>&1; then
    (sleep 1 && open "http://localhost:${PORT}") &
fi

exec python3 -m http.server "$PORT" --directory simulator
