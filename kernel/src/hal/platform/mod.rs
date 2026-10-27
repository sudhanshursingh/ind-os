//! Platform Backends for Ind OS Hardware Abstraction Layer (HAL)
//!
//! Constitutional References:
//! - REQ-HAL-102: At least two backends: QEMU virt (first) and hanoip (second)
//! - REQ-HAL-103: Backend-specific code isolated behind compile-time modules

pub mod qemu_virt;
pub mod hanoip;
pub mod lab_prep_core;
pub mod device_evidence_core;
pub mod boot_chain_core;
pub mod hardware_core;
pub mod ufs_core;
pub mod ufs_resilience_core;
pub mod display_panel_core;
pub mod cpu_compositor_core;
pub mod touch_controller_core;
pub mod input_pipeline_core;
pub mod pmic_spmi_core;
pub mod battery_gauge_core;
pub mod thermal_mgmt_core;
pub mod power_domains_core;
pub mod usb_dwc3_core;
pub mod usb_role_core;
pub mod firmware_loader_core;
pub mod wifi_wcn3990_core;
pub mod bluetooth_hci_core;
pub mod sensor_imu_core;
pub mod sensor_als_prox_core;
pub mod sensor_mag_core;
pub mod sensor_hub_core;
pub mod audio_codec_core;
pub mod audio_pcm_core;
pub mod concurrency_suite_core;
pub mod ais_arm64_core;
pub mod ais_thermal_core;
pub mod intent_hw_core;
pub mod ufs_mount_core;

pub use lab_prep_core::{
    KernelHardwareLabHarness, KernelLabPrepError, KERNEL_EXPECTED_UART_BAUD,
    KERNEL_SAFE_VOLTAGE_MAX_MV, KERNEL_SAFE_VOLTAGE_MIN_MV,
    KERNEL_THERMAL_CUTOFF_MILLICELSIUS,
};
pub use device_evidence_core::{
    KernelDeviceEvidenceError, KernelDeviceEvidenceHarness, KernelPartitionDescriptor,
    KERNEL_MAX_PARTITION_RECORDS, KERNEL_MOTO_G60_DEVICE_CODENAME, KERNEL_MOTO_G60_SOC_ID,
};
pub use boot_chain_core::{
    KernelBootChainError, KernelBootChainHarness, KERNEL_BOOT_MAGIC,
    KERNEL_QUALCOMM_PAGE_SIZE,
};
pub use hardware_core::{
    KernelHardwareCoreError, KernelHardwareCoreHarness, KERNEL_HANOIP_DRAM_BASE,
    KERNEL_HANOIP_DRAM_SIZE_BYTES, KERNEL_HANOIP_GENI_UART_BASE, KERNEL_HANOIP_PDC_GIC_BASE,
};
pub use ufs_core::{
    KernelUfsControllerHarness, KernelUfsDriverError, KERNEL_CAP_STORAGE_ADMIN,
    KERNEL_HANOIP_UFS_BASE,
};
pub use ufs_resilience_core::{
    KernelUfsJournalEntry, KernelUfsJournalError, KernelUfsJournalHarness,
    KERNEL_CAP_STORAGE_RECOVERY, KERNEL_SCSI_CMD_SYNCHRONIZE_CACHE,
};
pub use display_panel_core::{
    KernelDisplayError, KernelDisplayPanelHarness, KERNEL_DCS_CMD_DISPLAY_ON,
    KERNEL_DCS_CMD_PIXEL_FORMAT, KERNEL_DCS_CMD_SLEEP_OUT, KERNEL_HANOIP_MDSS_DSI0_BASE,
    KERNEL_PANEL_HEIGHT, KERNEL_PANEL_REFRESH_HZ, KERNEL_PANEL_WIDTH,
};
pub use cpu_compositor_core::{
    KernelCompositorError, KernelCpuCompositorHarness, KernelDirtyRect,
    KERNEL_CAP_SURFACE_COMPOSITOR, KERNEL_MAX_PANEL_HEIGHT, KERNEL_MAX_PANEL_WIDTH,
};
pub use touch_controller_core::{
    KernelTouchAction, KernelTouchControllerHarness, KernelTouchError, KernelTouchPoint,
    KERNEL_HANOIP_GENI_I2C1_BASE, KERNEL_TOUCH_I2C_SLAVE_ADDR, KERNEL_TOUCH_QUEUE_CAPACITY,
};
pub use input_pipeline_core::{
    KernelGesture, KernelInputPipelineError, KernelInputPipelineHarness,
    KERNEL_CAP_INPUT_SHELL,
};
pub use pmic_spmi_core::{
    KernelPmicError, KernelPmicSpmiHarness, KERNEL_CAP_POWER_MGMT,
    KERNEL_HANOIP_SPMI_BASE, KERNEL_PMIC_PM7150L_SID, KERNEL_PMIC_PM7150_SID,
};
pub use battery_gauge_core::{
    KernelBatteryError, KernelBatteryGaugeHarness, KernelChargingState,
    KERNEL_BATT_CRITICAL_SOC_PCT, KERNEL_BATT_MAX_VOLTAGE_MV, KERNEL_BATT_MIN_VOLTAGE_MV,
};
pub use thermal_mgmt_core::{
    KernelThermalError, KernelThermalMgmtHarness, KernelThermalZone,
    KERNEL_HANOIP_TSENS0_BASE, KERNEL_THERMAL_SHUTDOWN_MILLICELSIUS,
    KERNEL_THERMAL_THROTTLE_MILLICELSIUS, KERNEL_THERMAL_WARN_MILLICELSIUS,
};
pub use power_domains_core::{
    KernelPowerDomainError, KernelPowerDomainState, KernelPowerDomainsHarness,
    KERNEL_CAP_POWER_DOMAINS, KERNEL_HANOIP_RPMH_RSC_BASE,
    KERNEL_PSCI_FN64_CPU_SUSPEND, KERNEL_PSCI_FN_CPU_OFF, KERNEL_PSCI_FN_SYSTEM_RESET,
};
pub use usb_dwc3_core::{
    KernelUsbDwc3Harness, KernelUsbEndpointState, KernelUsbError, KernelUsbSpeed,
    KERNEL_CAP_USB_ADMIN, KERNEL_HANOIP_QUSB2_PHY_BASE, KERNEL_HANOIP_USB_DWC3_BASE,
    KERNEL_USB_DESC_TYPE_DEVICE, KERNEL_USB_REQ_GET_DESCRIPTOR, KERNEL_USB_REQ_SET_ADDRESS,
};
pub use usb_role_core::{
    KernelUsbRole, KernelUsbRoleError, KernelUsbRoleHarness,
    KERNEL_CAP_USB_ROLE_SWITCH, KERNEL_CDC_ACM_BAUD_DEFAULT, KERNEL_CDC_ACM_LINE_CODING_LEN,
    KERNEL_USB_ROLE_DEVICE, KERNEL_USB_ROLE_HOST,
};
pub use firmware_loader_core::{
    KernelFirmwareError, KernelFirmwareLoaderHarness, KernelFirmwareSubsystem,
    KERNEL_CAP_FIRMWARE_LOAD, KERNEL_FW_DEST_ADSP, KERNEL_FW_DEST_BT,
    KERNEL_FW_DEST_MODEM, KERNEL_FW_DEST_WLAN,
};
pub use wifi_wcn3990_core::{
    KernelWifiError, KernelWifiState, KernelWifiWcn3990Harness,
    KERNEL_CAP_NET_ADMIN, KERNEL_HANOIP_WCN3990_BASE,
};
pub use bluetooth_hci_core::{
    KernelBluetoothHciHarness, KernelBtError, KernelBtState,
    KERNEL_CAP_BT_ADMIN, KERNEL_HANOIP_BT_UART_BASE,
    KERNEL_HCI_CMD_LE_SET_SCAN_ENABLE, KERNEL_HCI_CMD_RESET,
};
pub use sensor_imu_core::{
    KernelImuError, KernelImuSample, KernelSensorImuHarness,
    KERNEL_BMI260_CHIP_ID, KERNEL_BMI260_I2C_ADDR, KERNEL_CAP_SENSOR_READ,
    KERNEL_HANOIP_IMU_I2C_BASE,
};
pub use sensor_als_prox_core::{
    KernelAlsGain, KernelAlsProxError, KernelProximityState, KernelSensorAlsProxHarness,
    KERNEL_ALS_PROX_I2C_ADDR, KERNEL_HANOIP_ALS_I2C_BASE, KERNEL_PROX_THRESHOLD_HIGH,
    KERNEL_PROX_THRESHOLD_LOW,
};
pub use sensor_mag_core::{
    kernel_compute_heading_deg, KernelMagError, KernelMagReading, KernelSensorMagHarness,
    KERNEL_AK09918_I2C_ADDR, KERNEL_AK09918_WIA1_COMPANY, KERNEL_AK09918_WIA2_DEVICE,
    KERNEL_HANOIP_MAG_I2C_BASE,
};
pub use sensor_hub_core::{
    KernelSensorChannel, KernelSensorEvent, KernelSensorHubError, KernelSensorHubHarness,
    KernelSensorSubscriber, KERNEL_CHANNEL_MASK_ACCEL, KERNEL_CHANNEL_MASK_ALL,
    KERNEL_CHANNEL_MASK_ALS, KERNEL_CHANNEL_MASK_GYRO, KERNEL_CHANNEL_MASK_MAG,
    KERNEL_CHANNEL_MASK_PROX, KERNEL_MAX_SENSOR_SUBSCRIBERS,
    KERNEL_SENSOR_EVENT_RING_CAPACITY,
};
pub use audio_codec_core::{
    KernelAudioCodecError, KernelAudioCodecHarness, KernelAudioCodecState,
    KERNEL_CAP_DEVICE_ADMIN, KERNEL_HANOIP_AUDIO_SLIMBUS_BASE,
    KERNEL_HANOIP_AUDIO_SLIMBUS_SIZE, KERNEL_WCD9385_MANUFACTURER_ID,
    KERNEL_WCD9385_PRODUCT_ID,
};
pub use audio_pcm_core::{
    KernelAudioPcmError, KernelAudioPcmHarness, KernelAudioPcmState,
    KERNEL_CAP_AUDIO_PLAYBACK, KERNEL_PCM_CHANNELS_STEREO,
    KERNEL_PCM_PERIODS, KERNEL_PCM_PERIOD_FRAMES, KERNEL_PCM_SAMPLE_RATE_48K,
};
pub use concurrency_suite_core::{
    KernelConcurrencyError, KernelConcurrencySuiteHarness,
    KERNEL_SUBSYSTEM_ALL, KERNEL_SUBSYSTEM_AUDIO, KERNEL_SUBSYSTEM_BLUETOOTH,
    KERNEL_SUBSYSTEM_SENSORS, KERNEL_SUBSYSTEM_WIFI,
};
pub use ais_arm64_core::{
    KernelAisArm64RuntimeHarness, KernelAisRuntimeError, KernelAisRuntimeState,
    KERNEL_AIS_MAX_RAM_BYTES, KERNEL_CAP_AIS_EXEC,
    KERNEL_TARGET_INFERENCE_LATENCY_MAX_MS,
};
pub use ais_thermal_core::{
    KernelAisThermalGovernorHarness, KernelAisThermalState,
    KERNEL_AIS_TEMP_EMERGENCY_HALT_MC, KERNEL_AIS_TEMP_NORMAL_MC,
    KERNEL_AIS_TEMP_RECOVERY_MC, KERNEL_AIS_TEMP_THROTTLE_MC,
};
pub use intent_hw_core::{
    KernelHardwareIntentBrokerHarness, KernelHardwareIntentError,
    KernelHardwareIntentState, KernelHardwareProposalCard,
    KERNEL_CAP_INTENT_DISPATCH,
};
pub use ufs_mount_core::{
    KernelUfsMountError, KernelUfsMountHarness, KernelUfsMountState,
    KERNEL_CAP_STORAGE_MOUNT, KERNEL_INDFS_BLOCK_SIZE_BYTES,
    KERNEL_INDFS_SUPERBLOCK_MAGIC,
};








