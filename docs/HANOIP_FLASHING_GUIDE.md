# Ind OS — Moto G60 (`hanoip`) Flashing & Dev Harness Guide

## 1. Reference Hardware Specifications
* **Device:** Motorola Moto G60
* **Codename:** `hanoip`
* **SoC:** Qualcomm Snapdragon 732G (`SM7150-AC`)
* **CPU:** 8x Kryo 470 (2x Cortex-A76 @ 2.3 GHz + 6x Cortex-A55 @ 1.8 GHz, ARMv8.2-A with Dot Product SIMD)
* **GPU:** Adreno 618 (Vulkan 1.1 compute)
* **RAM / Storage:** 6 GB LPDDR4X + 128 GB UFS 2.1

---

## 2. Rapid Developer Workflow (<15 s Cycle)

Instead of compiling the complete 200 GB AOSP tree for iterative development (Phases 1–5), Ind OS uses a Magisk/KernelSU systemless overlay directly on the testbed:

1. **Prerequisites:**
   * Moto G60 running LineageOS 22.x or unlocked stock firmware.
   * Rooted with Magisk (v26+) or KernelSU.
   * USB Debugging enabled (`adb devices` shows `device`).

2. **One-Command Build & Push:**
   ```bash
   ./tools/dev-harness/deploy_to_hanoip.sh --build --test
   ```
   This automatically:
   * Compiles the 4 Ind OS priv-app modules (`IndNeuralCore`, `IndZeroTouch`, `IndSurakshaShield`, `IndLauncher`).
   * Bundles them with `privapp-permissions-ind.xml` into a systemless module.
   * Pushes and activates the overlay under `/system_ext/priv-app`.
   * Executes the DoD verification test (Wi-Fi, Bluetooth, Brightness toggles).

---

## 3. Full ROM Build & Clean Flash (Phase 6)

For building and flashing the complete standalone Ind OS ROM:

1. **Environment Setup:**
   ```bash
   mkdir -p ~/ind-os-rom && cd ~/ind-os-rom
   repo init -u https://github.com/sudhanshursingh/ind-os -m manifest/default.xml -b main
   repo sync -j$(nproc) --force-sync
   ```

2. **Build Execution:**
   ```bash
   source build/envsetup.sh
   lunch ind_hanoip-userdebug
   mka bacon -j$(nproc)
   ```

3. **Flashing via Recovery / Fastboot:**
   ```bash
   adb reboot bootloader
   fastboot flash boot out/target/product/hanoip/boot.img
   fastboot flash dtbo out/target/product/hanoip/dtbo.img
   fastboot flash vendor_boot out/target/product/hanoip/vendor_boot.img
   fastboot reboot recovery
   # In recovery: Apply update -> Apply from ADB
   adb sideload out/target/product/hanoip/ind_hanoip-ota-*.zip
   ```

4. **Post-Flash Verification:**
   ```bash
   adb shell getenforce                    # Must output "Enforcing"
   adb shell getprop ro.ind.version        # Must output "v1.0.0-BIBLE"
   adb shell dumpsys meminfo               # Verify idle RAM <= 1.45 GB
   ```
