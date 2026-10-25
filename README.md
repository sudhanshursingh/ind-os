# Ind OS — AI-Native, Open-Source Mobile Operating System
### Engineered for the Next Billion Users

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Target: Moto G60](https://img.shields.io/badge/Hardware-Moto_G60_(hanoip)-green.svg)](docs/HANOIP_FLASHING_GUIDE.md)
[![Base: AOSP 15 / LineageOS 22.x](https://img.shields.io/badge/Base-AOSP_15_%2F_LineageOS_22.x-orange.svg)](docs/IND_OS_Master_PRD.md)
[![Security: Air--Gapped SELinux](https://img.shields.io/badge/Security-Air--Gapped_SELinux-red.svg)](docs/SECURITY_MODEL.md)

* **Founder & Lead System Architect:** Sudhanshu Raj Singh
* **Official Repository:** `https://github.com/sudhanshursingh/ind-os`
* **Tier-0 Reference Device:** Motorola Moto G60 (`hanoip` — Snapdragon 732G / SM7150-AC, 6 GB LPDDR4X, 128 GB UFS 2.1)

---

## The 5 Non-Negotiable Pillars

1. **Zero-Touch Ambient Intelligence:** Hands-free wake, unlock, navigation, and cross-app control in English, Hindi, Hinglish, and Indic languages.
2. **AI as an OS Primitive:** privileged native C++/JNI daemon (`indaid`) running on-device above the HAL, exposed via Android Binder IPC (`IIndAIManager.aidl`).
3. **Air-Gapped, User-Controlled Security:** SELinux kernel sandbox revoking all network socket permissions (`neverallow ind_ai_daemon domain:{ tcp_socket ... }`). Zero telemetry, zero cloud dependency.
4. **Zero Bloatware & BFSI Protection:** De-bloated OS footprint (<1.45 GB idle RAM) with **Ind Suraksha Shield** preventing screen-share OTP theft and banking trojans.
5. **Radical Openness:** 100% Apache 2.0 userspace code, open-weights GGUF/ONNX models, and fully reproducible builds.

---

## System Architecture

```
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

## Quickstart

### 1. Build Privileged Packages via Gradle
```bash
./gradlew assembleDebug
```

### 2. Deploy Systemless Overlay to Connected Moto G60
```bash
./tools/dev-harness/deploy_to_hanoip.sh --build --test
```

### 3. Documentation
* [Master PRD & Engineering Bible](docs/IND_OS_Master_PRD.md)
* [Security Model & Threat Defense](docs/SECURITY_MODEL.md)
* [Moto G60 Flashing Guide](docs/HANOIP_FLASHING_GUIDE.md)
