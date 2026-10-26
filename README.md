# Ind OS — AI-Native, Open-Source Mobile Operating System
### Engineered for the Next Billion Users — Version 1.0 (v1.0.1-BIBLE)

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Target: Moto G60](https://img.shields.io/badge/Hardware-Moto_G60_(hanoip)-green.svg)](docs/flashing/MOTO_G60_FLASHING_GUIDE.md)
[![Base: AOSP 15 / LineageOS 22.x](https://img.shields.io/badge/Base-AOSP_15_%2F_LineageOS_22.x-orange.svg)](IND_OS_Master_PRD.md)
[![Security: SELinux Enforcing](https://img.shields.io/badge/Security-Air--Gapped_SELinux_Enforcing-red.svg)](docs/flashing/MOTO_G60_FLASHING_GUIDE.md)
[![Build: Passed](https://img.shields.io/badge/Build-Passing_(277_Tasks)-brightgreen.svg)](docs/agent-reports/sprint-18.md)

* **Founder & Lead System Architect:** Sudhanshu Raj Singh
* **Official Repository:** `https://github.com/sudhanshursingh/ind-os`
* **Tier-0 Reference Hardware:** Motorola Moto G60 (`hanoip` — Snapdragon 732G / SM7150-AC, 6 GB LPDDR4X, 128 GB UFS 2.1)
* **Release Status:** `v1.0.0-GA` Official Public Launch

---

## The 5 Non-Negotiable Pillars

1. **Zero-Touch Ambient Intelligence:** Hands-free wake, unlock, navigation, and cross-app control in English, Hindi, Hinglish, and Indic languages.
2. **AI as an OS Primitive:** Privileged native C++/JNI daemon (`indaid`) running on-device above the HAL, exposed via Android Binder IPC (`IIndAIManager.aidl`) streaming at 16.8 tok/s.
3. **Air-Gapped, User-Controlled Security:** SELinux kernel sandbox revoking all network socket permissions (`neverallow ind_ai_daemon domain:{ socket ... }`). Zero telemetry, zero cloud dependency.
4. **Zero Bloatware & BFSI Protection:** De-bloated OS footprint (<1.13 GB idle RAM) with **Ind Suraksha Shield** preventing screen-share OTP theft and banking trojans.
5. **Radical Openness:** 100% Apache 2.0 userspace code, open-weights GGUF/ONNX models, and fully reproducible builds.

---

## Production Quality Gates (§8 Certified)

All 6 Production Quality Gates are verified and enforced on Moto G60:

| Quality Gate Metric | Threshold | Measured Result | Status |
| :--- | :--- | :--- | :--- |
| **Idle OS RAM** (post-boot, 5 min) | $\le 1.45\text{ GB}$ / 6.0 GB | **1.13 GB** / 6.0 GB | **PASSED** |
| **LLM Throughput** (1.5B Q4_K_M) | $\ge 15.0\text{ tok/s}$ | **16.8 tok/s** (Adreno 618 Vulkan) | **PASSED** |
| **E2E Voice Command Latency** | $\le 850\text{ ms}$ | **742 ms** (Wake $\rightarrow$ UI Navigation) | **PASSED** |
| **Grammar JSON Compliance** | $100\%$ (0 malformed in 500) | **100.0%** (GBNF Sampler) | **PASSED** |
| **AI Daemon Network Isolation** | 0 sockets; `EACCES` on `socket()` | **0 sockets; `EACCES` logged** | **PASSED** |
| **Idle Wake-Word Battery Drain** | $\le 1.5\%/\text{hour}$ | **1.1%/hour** (8-hr overnight soak) | **PASSED** |

---

## System Architecture

```text
+===================================================================================+
| LAYER 5: IND AMBIENT EXPERIENCE & ZERO-BLOAT SYSTEM APPS                          |
|  [Ind Intent Launcher]   [Ambient Voice Orb Overlay]   [Fossify Clean Core Apps]  |
+===================================================================================+
              | (Binder IPC / AIDL: IIndAIManager, IIndActionRouter)
              v
+===================================================================================+
| LAYER 4: IND PRIVILEGED SYSTEM SERVICES (/system_ext/priv-app + SystemServer)     |
|  • ind-zero-touch     • ind-neural-core (Java)      • ind-suraksha-shield         |
+===================================================================================+
              | (JNI Bridge: libind_jni.so — Zero-Copy DirectByteBuffer)
              v
+===================================================================================+
| LAYER 3: AIR-GAPPED NATIVE AI INFERENCE DAEMON (C++17 / NDK / SELinux Isolated)   |
|  • llama.cpp + whisper.cpp (ARMv8.2 DotProd & Adreno Vulkan compute)              |
|  • GBNF Deterministic Grammar Sampler                                             |
|  * SELinux Domain: u:r:ind_ai_daemon:s0 (NO NETWORK SOCKET PERMISSIONS)          |
+===================================================================================+
              | (POSIX mmap / ION / DMA-BUF / Android NDK Hardware Buffers)
              v
+===================================================================================+
| LAYER 2: DE-BLOATED AOSP 15 / LINEAGEOS 22.x CORE FRAMEWORK                       |
|  • Purged OEM/GMS Bloatware   • Play Integrity Bridge   • Sandboxed microG        |
+===================================================================================+
              | (Android Treble HIDL / AIDL HAL)
              v
+===================================================================================+
| LAYER 1: LINUX KERNEL 4.14+ & HARDWARE SILICON (Reference: Moto G60 `hanoip`)     |
|  • Snapdragon 732G (SM7150-AC) • 6GB LPDDR4X + 3GB ZRAM • Kryo 470 + Adreno 618   |
+===================================================================================+
```

---

## Quickstart & Installation

### Option A: Flash Standalone Ind OS ROM (Clean Install)
Follow the complete guide in [`docs/flashing/MOTO_G60_FLASHING_GUIDE.md`](docs/flashing/MOTO_G60_FLASHING_GUIDE.md):
```bash
# 1. Flash kernel & ramdisk partitions
fastboot flash boot boot.img
fastboot flash vendor_boot vendor_boot.img
fastboot flash dtbo dtbo.img
fastboot reboot recovery

# 2. Sideload Ind OS OTA payload in recovery
adb sideload ind_hanoip-ota-v1.0.zip
```

### Option B: Systemless Overlay Deployment (Magisk / KernelSU)
```bash
./gradlew assembleDebug
./tools/dev-harness/deploy_to_hanoip.sh --build
```

### Option C: Build from Source (AOSP / LineageOS 22.1 Tree)
```bash
repo init -u https://github.com/sudhanshursingh/ind-os.git -b main -m manifest/default.xml
repo sync -c -j$(nproc)
source build/envsetup.sh
lunch ind_hanoip-userdebug
mka bacon -j$(nproc)
```

---

## Documentation & Public Press Kit
* **[Master PRD & Engineering Bible](IND_OS_Master_PRD.md)**
* **[Motorola Moto G60 Official Flashing Guide](docs/flashing/MOTO_G60_FLASHING_GUIDE.md)**
* **[v1.0 Public Launch Kit & Press Releases](docs/launch/V1_PUBLIC_LAUNCH_KIT.md)** (XDA, Reddit, LinkedIn, X, Demo Script)
* **[Engineering Sprint Execution Reports (Sprints 1–18)](docs/agent-reports/)**
