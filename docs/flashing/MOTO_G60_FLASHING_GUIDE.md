# Ind OS v1.0 — Motorola Moto G60 (`hanoip`) Official Flashing Guide
**Target Device:** Motorola Moto G60 (`hanoip` / `XT2135-1` / `XT2135-2`)  
**Hardware Specifications:** Qualcomm Snapdragon 732G (SM7150-AC), 6 GB LPDDR4X RAM, 128 GB UFS 2.1, 120 Hz 6.8" FHD+ IPS LCD  
**OS Version:** Ind OS v1.0.1-BIBLE (LineageOS 22.1 / Android 15 Core)  
**Security Status:** SELinux = Enforcing | 100% On-Device Air-Gapped AI Runtime  

---

## 1. Prerequisites & Safety Precautions

> [!IMPORTANT]
> Flashing custom operating systems wipes all user data on the device. Back up all photos, documents, and credentials before proceeding. Ensure battery level is at least 60%.

### Required Tools on Host Machine
* **Android Platform Tools (v35.0.0+):** `fastboot` and `adb` installed and present in your system `PATH`.
* **Motorola USB Drivers (Windows only):** Standard Qualcomm HS-USB Android drivers.
* **USB-C Data Cable:** High-quality cable connected directly to the host machine (avoid unpowered USB hubs).

---

## 2. Bootloader Unlocking (Motorola Official Portal)

1. On your Moto G60, go to **Settings → About Phone** and tap **Build Number** 7 times to enable Developer Options.
2. Go to **Settings → System → Developer Options** and enable:
   * **OEM Unlocking**
   * **USB Debugging**
3. Connect the phone to your computer via USB and execute:
   ```bash
   adb reboot bootloader
   ```
4. Retrieve the device unlock data string:
   ```bash
   fastboot oem get_unlock_data
   ```
5. Paste the output string into the official [Motorola Bootloader Unlock Portal](https://motorola-global-portal.custhelp.com/app/standalone/bootloader/unlock-your-device-b) to obtain your unique Unlock Key.
6. Execute the unlock command:
   ```bash
   fastboot oem unlock <YOUR_UNIQUE_KEY>
   ```
   *(Confirm the warning on the device display using the Volume and Power keys).*

---

## 3. Flashable ROM Installation (Clean Flash via Recovery)

### Step 3.1: Download Ind OS v1.0 Release Artifacts
Download the official signed release package from the [Ind OS GitHub Releases](https://github.com/sudhanshursingh/ind-os/releases/tag/v1.0.0):
* `boot.img` (Kernel 4.14+ with Ind OS Binder IPC & ashmem optimizations)
* `vendor_boot.img` (Treble HAL drivers & ramdisk)
* `dtbo.img` (Device tree overlay)
* `ind_hanoip-ota-v1.0.zip` (Full signed flashable system payload)

### Step 3.2: Flash Core Partitions via Fastboot
Put the phone into Fastboot mode (`Volume Down + Power` or `adb reboot bootloader`):
```bash
# 1. Flash kernel & ramdisk partitions
fastboot flash boot boot.img
fastboot flash vendor_boot vendor_boot.img
fastboot flash dtbo dtbo.img

# 2. Reboot into Ind OS Recovery
fastboot reboot recovery
```

### Step 3.3: Sideload Ind OS Payload via ADB
1. In Ind OS Recovery, use the Volume keys to highlight **Factory Reset → Format Data / Cache** and press the Power key to confirm.
2. Return to the main recovery menu and select **Apply Update → Apply from ADB**.
3. On your host computer, transmit the signed OTA package:
   ```bash
   adb sideload ind_hanoip-ota-v1.0.zip
   ```
4. Once sideloading reaches 100% (or step 2/2 completes), select **Reboot System Now**.

---

## 4. Alternative Method: Systemless Overlay Deployment (Magisk / KernelSU)

For developers and advanced users running existing LineageOS 22.x or AOSP 15 ROMs on `hanoip`, Ind OS can be installed systemlessly without wiping existing data:

```bash
# 1. Clone the Ind OS repository and build APKs
git clone https://github.com/sudhanshursingh/ind-os.git
cd ind-os
./gradlew assembleDebug

# 2. Deploy systemless overlay via automated harness
./tools/dev-harness/deploy_to_hanoip.sh --build

# 3. Or install module zip manually via Magisk / KernelSU
adb push build/artifacts/ind_os_system_overlay.zip /data/local/tmp/
adb shell su -c 'magisk --install-module /data/local/tmp/ind_os_system_overlay.zip'
adb reboot
```

---

## 5. Post-Boot Verification & Health Checks

Once Ind OS boots and you are greeted by the Ind OS glowing startup animation and Voice Orb, open a terminal on your computer to verify system invariants:

### Check 1: SELinux Enforcing
```bash
adb shell getenforce
# Expected output:
# Enforcing
```

### Check 2: Ind OS Identity & Version
```bash
adb shell getprop ro.ind.version
# Expected output:
# v1.0.1-BIBLE

adb shell getprop ro.ind.founder
# Expected output:
# Sudhanshu_Raj_Singh
```

### Check 3: Air-Gap Network Socket Isolation
```bash
adb shell "cat /proc/net/tcp /proc/net/udp | grep -v 'sl' | wc -l"
# Expected output for ind_ai_daemon:
# 0 sockets held
```

### Check 4: Play Integrity & Banking Compatibility
Launch **BHIM UPI**, **PhonePe**, **Google Pay**, or **YONO SBI**. Verify that the app launches seamlessly and passes hardware integrity attestation with zero root/ROM warnings.

---

## 6. Building Ind OS from Source (Reproducible Builds)

For system developers building from source on a Linux build box (16 vCPU, 64 GB RAM, 400 GB NVMe):

```bash
# 1. Initialize repo from Ind OS manifest
mkdir ind-os-tree && cd ind-os-tree
repo init -u https://github.com/sudhanshursingh/ind-os.git -b main -m manifest/default.xml
repo sync -c -j$(nproc) --no-clone-bundle --no-tags

# 2. Setup build environment
source build/envsetup.sh
lunch ind_hanoip-userdebug

# 3. Compile flashable bacon OTA package
mka bacon -j$(nproc)

# 4. Resulting zip will be generated in:
# out/target/product/hanoip/ind_hanoip-ota-v1.0.zip
```
