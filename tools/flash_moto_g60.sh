#!/usr/bin/env bash
# Ind OS Moto G60 (hanoip) Flashing & Safe Boot Harness
# Standards: Ind OS Development Bible v1.0.0.0 Table 19.2 (Gate G9), Rule N-06, Rule N-11

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$WORKSPACE_ROOT"

# 1. Locate fastboot binary
FASTBOOT_BIN=""
if command -v fastboot &>/dev/null; then
    FASTBOOT_BIN="fastboot"
elif [ -f "$HOME/Library/Android/sdk/platform-tools/fastboot" ]; then
    FASTBOOT_BIN="$HOME/Library/Android/sdk/platform-tools/fastboot"
elif [ -f "/opt/homebrew/bin/fastboot" ]; then
    FASTBOOT_BIN="/opt/homebrew/bin/fastboot"
elif [ -f "/usr/local/bin/fastboot" ]; then
    FASTBOOT_BIN="/usr/local/bin/fastboot"
else
    echo "❌ Error: fastboot binary not found. Please install Android Platform Tools."
    exit 1
fi

echo "=================================================="
echo " Ind OS Moto G60 Hardware Deployment Tool         "
echo " Standards: Gate G9, Rule N-06 (Recovery First)   "
echo " Fastboot Binary: $FASTBOOT_BIN"
echo "=================================================="

# 2. Check device connectivity
DEVICES=$("$FASTBOOT_BIN" devices)
if [ -z "$DEVICES" ]; then
    echo "❌ Error: No fastboot devices detected."
    echo "Please ensure the Moto G60 is connected via USB and in Fastboot mode."
    exit 1
fi
echo "✓ Device Detected:"
echo "$DEVICES"

# 3. Query Device Properties
PRODUCT=$("$FASTBOOT_BIN" getvar product 2>&1 | grep "product:" | awk '{print $2}' || true)
SECURE_STATE=$("$FASTBOOT_BIN" getvar securestate 2>&1 | grep "securestate:" | awk '{print $2}' || true)
CURRENT_SLOT=$("$FASTBOOT_BIN" getvar current-slot 2>&1 | grep "current-slot:" | awk '{print $2}' || true)

echo "  - Product:        $PRODUCT"
echo "  - Security State: $SECURE_STATE"
echo "  - Current Slot:   $CURRENT_SLOT"

if [ "$PRODUCT" != "hanoip" ]; then
    echo "⚠️ Warning: Detected product is '$PRODUCT' (expected 'hanoip'). Proceed with caution."
fi

if [[ "$SECURE_STATE" != *"unlocked"* ]]; then
    echo "❌ Error: Bootloader is not unlocked ($SECURE_STATE). Cannot flash non-stock images."
    exit 1
fi

# 4. Ensure boot image is generated
BOOT_IMG="target/indos-hanoip-boot.img"
if [ ! -f "$BOOT_IMG" ]; then
    echo "Building Moto G60 hardware boot image..."
    make kernel-hanoip
fi

ACTION="${1:-help}"

case "$ACTION" in
    "boot"|"test-boot")
        echo "=================================================="
        echo " [SAFE TEST] Booting Ind OS in RAM via fastboot   "
        echo " Rule N-06 Compliance: Zero flash partitions modified!"
        echo " If the system halts, hold Power to reboot stock."
        echo "=================================================="
        "$FASTBOOT_BIN" boot "$BOOT_IMG"
        ;;

    "flash-inactive")
        INACTIVE_SLOT="a"
        if [ "$CURRENT_SLOT" = "a" ]; then
            INACTIVE_SLOT="b"
        fi
        echo "=================================================="
        echo " Flashing Inactive Slot boot_$INACTIVE_SLOT      "
        echo " Active Slot ($CURRENT_SLOT) remains untouched stock ROM."
        echo "=================================================="
        "$FASTBOOT_BIN" flash "boot_$INACTIVE_SLOT" "$BOOT_IMG"
        echo "✓ Flashed to boot_$INACTIVE_SLOT successfully."
        echo "To test booting into slot $INACTIVE_SLOT:"
        echo "  $FASTBOOT_BIN --set-active=$INACTIVE_SLOT"
        echo "  $FASTBOOT_BIN reboot"
        echo "To revert back to stock:"
        echo "  $FASTBOOT_BIN --set-active=$CURRENT_SLOT"
        ;;

    "flash-active")
        echo "=================================================="
        echo " ⚠️ WARNING: Flashing Active Slot boot_$CURRENT_SLOT "
        echo " This overwrites the currently selected boot partition."
        echo "=================================================="
        read -p "Are you sure you want to proceed? [y/N]: " CONFIRM
        if [[ "$CONFIRM" =~ ^[Yy]$ ]]; then
            "$FASTBOOT_BIN" flash "boot_$CURRENT_SLOT" "$BOOT_IMG"
            echo "✓ Flashed to boot_$CURRENT_SLOT successfully."
        else
            echo "Operation cancelled."
        fi
        ;;

    "boot-slot-a"|"test-slot-a")
        echo "=================================================="
        echo " Switching to Slot A and rebooting into Ind OS... "
        echo "=================================================="
        "$FASTBOOT_BIN" --set-active=a
        "$FASTBOOT_BIN" reboot
        ;;

    "set-slot-a")
        echo "Setting active slot to A..."
        "$FASTBOOT_BIN" --set-active=a
        ;;

    "set-slot-b"|"revert-stock")
        echo "=================================================="
        echo " Reverting back to Stock ROM (Slot B)...         "
        echo "=================================================="
        "$FASTBOOT_BIN" --set-active=b
        echo "Active slot is now B (Stock). Run './tools/flash_moto_g60.sh reboot' when ready."
        ;;

    "build")
        echo "Building Moto G60 hardware image..."
        make kernel-hanoip
        ;;

    "status")
        "$FASTBOOT_BIN" getvar all
        ;;

    "reboot")
        "$FASTBOOT_BIN" reboot
        ;;

    *)
        echo "Usage: ./tools/flash_moto_g60.sh [action]"
        echo ""
        echo "Actions:"
        echo "  build          - Rebuild kernel and package boot.img for Moto G60"
        echo "  boot           - [SAFE TEST] Boot kernel directly in RAM (Rule N-06)"
        echo "  flash-inactive - Flash to inactive slot (boot_a)"
        echo "  boot-slot-a    - Set active slot to A and reboot to test Ind OS"
        echo "  set-slot-a     - Set active slot to A"
        echo "  revert-stock   - Set active slot back to B (Stock ROM)"
        echo "  reboot         - Reboot device"
        echo "  status         - Query all fastboot device variables"
        echo ""
        ;;
esac
