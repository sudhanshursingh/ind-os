# Ind OS Hardware Compatibility Matrix v1 (Motorola Moto G60 `hanoip`)

**Document ID:** MATRIX-HW-001  
**Target Device:** Motorola Moto G60 (Qualcomm Snapdragon 732G SM7150-AC)  
**Standard:** PRD-001 §8.3, Ind OS Development Bible §8.15 Table 19.2  
**Status:** PUBLISHED & RATIFIED (Sprint SP-104)  

---

| # | Subsystem / Component | Hardware Device | Status | Rationale / Verification Evidence |
| :-: | :--- | :--- | :-: | :--- |
| 1 | **Boot-Chain & Recovery** | Qualcomm PBL / SBL / Fastboot / EDL | **WORKS** | Stages 300–304: 115200 8N1 UART console, EDL unbrick pinout validated. |
| 2 | **CPU Core & Architecture** | Octa-core Kryo 470 (2x Gold + 6x Silver) | **WORKS** | Stages 315–319: ARM64 v8.2-A MMU 4KB translation, GICv3 interrupt distributor. |
| 3 | **System Memory (DRAM)** | 6GB LPDDR4X @ 1866MHz | **WORKS** | Stages 316–319: Zero memory corruption across 4GB continuous mapping test. |
| 4 | **Mass Storage (UFS)** | 128GB Qualcomm UFS 2.2 Gear 3 | **WORKS** | Stages 320–324 & 445–449: IndFS root mount, clean journal replay, A/B slots. |
| 5 | **Display Scanout Panel** | Tianma FT8726 6.8" DSI LCD (1080x2460 @ 120Hz) | **WORKS** | Stages 330–334 & 450–454: 120Hz double-buffered zero-copy swapchain. |
| 6 | **Touchscreen Controller** | FocalTech FT3518 I2C Multi-Touch | **WORKS** | Stages 345–349 & 452: Multi-finger coordinate acquisition with <16ms latency. |
| 7 | **Power Management (PMIC)** | Qualcomm PM6150 / PM6150L & SMB1395 | **WORKS** | Stages 350–354 & 455–459: Voltage rails, fuel-gauge battery telemetry, thermal governor. |
| 8 | **USB Interface** | Synopsys DWC3 USB 2.0/3.1 Type-C | **WORKS** | Stages 375–379: USB peripheral enumeration, ADB serial transfer. |
| 9 | **Wi-Fi Connectivity** | Qualcomm WCN3990 802.11a/b/g/n/ac 2x2 MIMO | **WORKS** | Stages 385–389: BSSID scan, WPA2/WPA3 handshake frame parser. |
| 10 | **Bluetooth Connectivity** | Qualcomm WCN3990 Bluetooth 5.0 LE | **WORKS** | Stages 390–394: HCI UART packet transport, advertising packet reception. |
| 11 | **Sensor Subsystem** | InvenSense ICM-42607 (IMU) + AK09918 + ALS/Prox | **WORKS** | Stages 395–414: Microsecond timestamp synchronization, event multiplexing hub. |
| 12 | **Audio Codec & Output** | Qualcomm WCD9385 & SLIMbus DMA | **WORKS** | Stages 415–424: 48kHz stereo PCM streaming, zero-pad buffer underrun recovery. |
| 13 | **Neural Accelerator (NPU)** | Qualcomm Hexagon 688 DSP | **OUT-OF-SCOPE** | **Excluded per Rule N-01:** Closed-source binary firmware blobs forbidden in TCB. Replaced by ARM64 CPU NEON quantized inference (SP-96). |
| 14 | **Camera ISP** | Qualcomm Spectra 350L Dual ISP | **PARTIAL** | MIPI CSI-2 bus probing verified; full image pipeline scheduled for Phase 10. |
| 15 | **Biometric Fingerprint** | Fingerprint Cards FPC1020 SPI Sensor | **PARTIAL** | SPI communication and register read verified; biometric enclave scheduled for Phase 10. |

---

### Summary Statistics
- **Total Components Audited:** 15
- **WORKS (Fully Functional):** 12 (80.0%)
- **PARTIAL (Phase 10 Target):** 2 (13.3%)
- **OUT-OF-SCOPE (Rule N-01 Excluded):** 1 (6.7%)
- **BLOCKED:** 0 (0.0%)
