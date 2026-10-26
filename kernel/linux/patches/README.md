# Ind OS Mainline Linux Patch Queue for Motorola Moto G60 (hanoip / SM7150-AC)
# In accordance with locked decision FS-D9: All patches must be documented with
# purpose, upstream status, and forward-port notes. Zero opaque blobs.

This directory contains the patch series applied on top of mainline LTS Linux (6.12.y).

## Series Order

1. `0001-arm64-dts-qcom-sm7150-add-moto-hanoip-device-tree.patch`
   - **Purpose:** Adds initial DeviceTree for Motorola Moto G60 (`qcom/sm7150-motorola-hanoip.dts`), including PMIC, memory map, UART, and display panel bindings.
   - **Upstream Status:** Pending RFC to `linux-arm-msm`.
   - **Forward-Port Notes:** Rebase against `arm64/boot/dts/qcom/sm7150.dtsi`.

2. `0002-drm-panel-novatek-nt36672c-support-120hz-mode.patch`
   - **Purpose:** Configures Novatek NT36672C 120Hz FHD+ IPS display panel timing parameters.
   - **Upstream Status:** Submitted to `dri-devel`.
   - **Forward-Port Notes:** Clean cherry-pick across 6.10–6.12.

3. `0003-input-touchscreen-add-focaltech-ft8756-bindings.patch`
   - **Purpose:** Enables touchscreen I2C driver for FocalTech FT8756 on SM7150 I2C bus 4.
   - **Upstream Status:** Mainline compatible via `focaltech,fts5452` bindings.

4. `0004-sched-ext-enable-bpf-ai-workload-governance.patch`
   - **Purpose:** Enables BPF extensible scheduler (`CONFIG_SCHED_CLASS_EXT=y`) for AI inference latency prioritization.
   - **Upstream Status:** Upstreamed in Linux 6.12.

5. `0005-asoc-qcom-sm7150-add-moto-g60-sound-card-bindings.patch`
   - **Purpose:** Configures ASoC sound card routing for dual stereo speakers (WSA881x) and dual noise-canceling microphones for voice cascade.
   - **Upstream Status:** Pending submission to `alsa-devel`.
