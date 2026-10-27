//! Ind OS Hardware Abstraction Layer (HAL) Core
//!
//! Constitutional References:
//! - REQ-HAL-101: Generic device-model trait defined (enumeration, IRQ, DMA, power, errors)
//! - REQ-HAL-102: At least two backends: QEMU virt (first) and hanoip (second)
//! - REQ-HAL-103: Backend-specific code isolated behind compile-time traits
//! - REQ-HAL-104: Conformance test suite runnable against any backend
//! - REQ-HAL-105: HAL interface stability protocol
//! - REQ-ARCH-101: Standalone indk-core workspace decoupling
//! - Rule N-04: Zero unsafe code in new kernel modules

pub mod platform;
pub mod traits;

pub use platform::qemu_virt::{
    QemuVirtClockReset, QemuVirtDmaAllocator, QemuVirtHalDevice, QemuVirtIrqController,
    QemuVirtMmio, QemuVirtPower, QEMU_VIRT_DMA_BASE, QEMU_VIRT_DMA_SIZE,
    QEMU_VIRT_GIC_DIST_BASE, QEMU_VIRT_UART_BASE, QEMU_VIRT_VIRTIO_BASE,
};
pub use platform::lab_prep_core::{
    KernelHardwareLabHarness, KernelLabPrepError, KERNEL_EXPECTED_UART_BAUD,
    KERNEL_SAFE_VOLTAGE_MAX_MV, KERNEL_SAFE_VOLTAGE_MIN_MV,
    KERNEL_THERMAL_CUTOFF_MILLICELSIUS,
};
pub use platform::device_evidence_core::{
    KernelDeviceEvidenceError, KernelDeviceEvidenceHarness, KernelPartitionDescriptor,
    KERNEL_MAX_PARTITION_RECORDS, KERNEL_MOTO_G60_DEVICE_CODENAME, KERNEL_MOTO_G60_SOC_ID,
};
pub use platform::boot_chain_core::{
    KernelBootChainError, KernelBootChainHarness, KERNEL_BOOT_MAGIC,
    KERNEL_QUALCOMM_PAGE_SIZE,
};
pub use platform::hardware_core::{
    KernelHardwareCoreError, KernelHardwareCoreHarness, KERNEL_HANOIP_DRAM_BASE,
    KERNEL_HANOIP_DRAM_SIZE_BYTES, KERNEL_HANOIP_GENI_UART_BASE, KERNEL_HANOIP_PDC_GIC_BASE,
};
pub use platform::ufs_core::{
    KernelUfsControllerHarness, KernelUfsDriverError, KERNEL_CAP_STORAGE_ADMIN,
    KERNEL_HANOIP_UFS_BASE,
};
pub use platform::ufs_resilience_core::{
    KernelUfsJournalEntry, KernelUfsJournalError, KernelUfsJournalHarness,
    KERNEL_CAP_STORAGE_RECOVERY, KERNEL_SCSI_CMD_SYNCHRONIZE_CACHE,
};
pub use platform::display_panel_core::{
    KernelDisplayError, KernelDisplayPanelHarness, KERNEL_DCS_CMD_DISPLAY_ON,
    KERNEL_DCS_CMD_PIXEL_FORMAT, KERNEL_DCS_CMD_SLEEP_OUT, KERNEL_HANOIP_MDSS_DSI0_BASE,
    KERNEL_PANEL_HEIGHT, KERNEL_PANEL_REFRESH_HZ, KERNEL_PANEL_WIDTH,
};
pub use platform::cpu_compositor_core::{
    KernelCompositorError, KernelCpuCompositorHarness, KernelDirtyRect,
    KERNEL_CAP_SURFACE_COMPOSITOR, KERNEL_MAX_PANEL_HEIGHT, KERNEL_MAX_PANEL_WIDTH,
};
pub use platform::touch_controller_core::{
    KernelTouchAction, KernelTouchControllerHarness, KernelTouchError, KernelTouchPoint,
    KERNEL_HANOIP_GENI_I2C1_BASE, KERNEL_TOUCH_I2C_SLAVE_ADDR, KERNEL_TOUCH_QUEUE_CAPACITY,
};
pub use platform::input_pipeline_core::{
    KernelGesture, KernelInputPipelineError, KernelInputPipelineHarness,
    KERNEL_CAP_INPUT_SHELL,
};
pub use platform::pmic_spmi_core::{
    KernelPmicError, KernelPmicSpmiHarness, KERNEL_CAP_POWER_MGMT,
    KERNEL_HANOIP_SPMI_BASE, KERNEL_PMIC_PM7150L_SID, KERNEL_PMIC_PM7150_SID,
};
pub use platform::battery_gauge_core::{
    KernelBatteryError, KernelBatteryGaugeHarness, KernelChargingState,
    KERNEL_BATT_CRITICAL_SOC_PCT, KERNEL_BATT_MAX_VOLTAGE_MV, KERNEL_BATT_MIN_VOLTAGE_MV,
};
pub use platform::thermal_mgmt_core::{
    KernelThermalError, KernelThermalMgmtHarness, KernelThermalZone,
    KERNEL_HANOIP_TSENS0_BASE, KERNEL_THERMAL_SHUTDOWN_MILLICELSIUS,
    KERNEL_THERMAL_THROTTLE_MILLICELSIUS, KERNEL_THERMAL_WARN_MILLICELSIUS,
};
pub use platform::power_domains_core::{
    KernelPowerDomainError, KernelPowerDomainState, KernelPowerDomainsHarness,
    KERNEL_CAP_POWER_DOMAINS, KERNEL_HANOIP_RPMH_RSC_BASE,
    KERNEL_PSCI_FN64_CPU_SUSPEND, KERNEL_PSCI_FN_CPU_OFF, KERNEL_PSCI_FN_SYSTEM_RESET,
};
pub use platform::usb_dwc3_core::{
    KernelUsbDwc3Harness, KernelUsbEndpointState, KernelUsbError, KernelUsbSpeed,
    KERNEL_CAP_USB_ADMIN, KERNEL_HANOIP_QUSB2_PHY_BASE, KERNEL_HANOIP_USB_DWC3_BASE,
    KERNEL_USB_DESC_TYPE_DEVICE, KERNEL_USB_REQ_GET_DESCRIPTOR, KERNEL_USB_REQ_SET_ADDRESS,
};
pub use platform::usb_role_core::{
    KernelUsbRole, KernelUsbRoleError, KernelUsbRoleHarness,
    KERNEL_CAP_USB_ROLE_SWITCH, KERNEL_CDC_ACM_BAUD_DEFAULT, KERNEL_CDC_ACM_LINE_CODING_LEN,
    KERNEL_USB_ROLE_DEVICE, KERNEL_USB_ROLE_HOST,
};
pub use platform::firmware_loader_core::{
    KernelFirmwareError, KernelFirmwareLoaderHarness, KernelFirmwareSubsystem,
    KERNEL_CAP_FIRMWARE_LOAD, KERNEL_FW_DEST_ADSP, KERNEL_FW_DEST_BT,
    KERNEL_FW_DEST_MODEM, KERNEL_FW_DEST_WLAN,
};
pub use platform::wifi_wcn3990_core::{
    KernelWifiError, KernelWifiState, KernelWifiWcn3990Harness,
    KERNEL_CAP_NET_ADMIN, KERNEL_HANOIP_WCN3990_BASE,
};
pub use platform::bluetooth_hci_core::{
    KernelBluetoothHciHarness, KernelBtError, KernelBtState,
    KERNEL_CAP_BT_ADMIN, KERNEL_HANOIP_BT_UART_BASE,
    KERNEL_HCI_CMD_LE_SET_SCAN_ENABLE, KERNEL_HCI_CMD_RESET,
};
pub use platform::sensor_imu_core::{
    KernelImuError, KernelImuSample, KernelSensorImuHarness,
    KERNEL_BMI260_CHIP_ID, KERNEL_BMI260_I2C_ADDR, KERNEL_CAP_SENSOR_READ,
    KERNEL_HANOIP_IMU_I2C_BASE,
};
pub use platform::sensor_als_prox_core::{
    KernelAlsGain, KernelAlsProxError, KernelProximityState, KernelSensorAlsProxHarness,
    KERNEL_ALS_PROX_I2C_ADDR, KERNEL_HANOIP_ALS_I2C_BASE, KERNEL_PROX_THRESHOLD_HIGH,
    KERNEL_PROX_THRESHOLD_LOW,
};
pub use platform::sensor_mag_core::{
    kernel_compute_heading_deg, KernelMagError, KernelMagReading, KernelSensorMagHarness,
    KERNEL_AK09918_I2C_ADDR, KERNEL_AK09918_WIA1_COMPANY, KERNEL_AK09918_WIA2_DEVICE,
    KERNEL_HANOIP_MAG_I2C_BASE,
};
pub use platform::sensor_hub_core::{
    KernelSensorChannel, KernelSensorEvent, KernelSensorHubError, KernelSensorHubHarness,
    KernelSensorSubscriber, KERNEL_CHANNEL_MASK_ACCEL, KERNEL_CHANNEL_MASK_ALL,
    KERNEL_CHANNEL_MASK_ALS, KERNEL_CHANNEL_MASK_GYRO, KERNEL_CHANNEL_MASK_MAG,
    KERNEL_CHANNEL_MASK_PROX, KERNEL_MAX_SENSOR_SUBSCRIBERS,
    KERNEL_SENSOR_EVENT_RING_CAPACITY,
};
pub use platform::audio_codec_core::{
    KernelAudioCodecError, KernelAudioCodecHarness, KernelAudioCodecState,
    KERNEL_CAP_DEVICE_ADMIN, KERNEL_HANOIP_AUDIO_SLIMBUS_BASE,
    KERNEL_HANOIP_AUDIO_SLIMBUS_SIZE, KERNEL_WCD9385_MANUFACTURER_ID,
    KERNEL_WCD9385_PRODUCT_ID,
};
pub use platform::audio_pcm_core::{
    KernelAudioPcmError, KernelAudioPcmHarness, KernelAudioPcmState,
    KERNEL_CAP_AUDIO_PLAYBACK, KERNEL_PCM_CHANNELS_STEREO,
    KERNEL_PCM_PERIODS, KERNEL_PCM_PERIOD_FRAMES, KERNEL_PCM_SAMPLE_RATE_48K,
};
pub use platform::concurrency_suite_core::{
    KernelConcurrencyError, KernelConcurrencySuiteHarness,
    KERNEL_SUBSYSTEM_ALL, KERNEL_SUBSYSTEM_AUDIO, KERNEL_SUBSYSTEM_BLUETOOTH,
    KERNEL_SUBSYSTEM_SENSORS, KERNEL_SUBSYSTEM_WIFI,
};
pub use platform::ais_arm64_core::{
    KernelAisArm64RuntimeHarness, KernelAisRuntimeError, KernelAisRuntimeState,
    KERNEL_AIS_MAX_RAM_BYTES, KERNEL_CAP_AIS_EXEC,
    KERNEL_TARGET_INFERENCE_LATENCY_MAX_MS,
};
pub use platform::ais_thermal_core::{
    KernelAisThermalGovernorHarness, KernelAisThermalState,
    KERNEL_AIS_TEMP_EMERGENCY_HALT_MC, KERNEL_AIS_TEMP_NORMAL_MC,
    KERNEL_AIS_TEMP_RECOVERY_MC, KERNEL_AIS_TEMP_THROTTLE_MC,
};
pub use traits::{
    ClockResetController, DmaAllocator, HalDevice, HalError, IrqController,
    PowerController, PowerState, RegisterAccess,
};

use crate::sync::Spinlock;

pub const HAL_MAX_DEVICES: usize = 8;

pub const DEV_ID_CONSOLE: u32 = 1;
pub const DEV_ID_STORAGE: u32 = 2;
pub const DEV_ID_INPUT: u32 = 3;
pub const DEV_ID_DISPLAY: u32 = 4;

/// Result report produced by the HAL Conformance Test Harness (`REQ-HAL-104`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HalConformanceReport {
    pub mmio_passed: bool,
    pub irq_passed: bool,
    pub dma_passed: bool,
    pub clock_passed: bool,
    pub power_passed: bool,
    pub device_enum_passed: bool,
    pub drivers_bound_passed: bool,
    pub all_tests_passed: bool,
}

/// Central HAL Subsystem Registry and Resource Manager.
pub struct HalManager {
    devices: [Option<QemuVirtHalDevice>; HAL_MAX_DEVICES],
    device_count: usize,
    pub irq_controller: QemuVirtIrqController,
    pub dma_allocator: QemuVirtDmaAllocator,
    pub clock_reset: QemuVirtClockReset,
    pub power: QemuVirtPower,
    initialized: bool,
}

impl HalManager {
    pub const fn new() -> Self {
        Self {
            devices: [None, None, None, None, None, None, None, None],
            device_count: 0,
            irq_controller: QemuVirtIrqController::new(),
            dma_allocator: QemuVirtDmaAllocator::new(QEMU_VIRT_DMA_BASE, QEMU_VIRT_DMA_SIZE),
            clock_reset: QemuVirtClockReset::new(),
            power: QemuVirtPower::new(),
            initialized: false,
        }
    }

    /// Initialize HAL platform backend and register baseline QEMU virt hardware devices.
    pub fn init(&mut self) {
        if self.initialized {
            return;
        }

        // 1. Register Console UART (PL011)
        self.register_device(QemuVirtHalDevice::new(
            DEV_ID_CONSOLE,
            "qemu-virt-uart-pl011",
            0x0001, // TTY / Char
            QEMU_VIRT_UART_BASE,
            0x1000,
            33,
        ));

        // 2. Register Storage (VirtIO Block)
        self.register_device(QemuVirtHalDevice::new(
            DEV_ID_STORAGE,
            "qemu-virt-virtio-blk",
            0x0002, // Block / Storage
            QEMU_VIRT_VIRTIO_BASE,
            0x0200,
            48,
        ));

        // 3. Register Input (VirtIO Tablet / Keyboard)
        self.register_device(QemuVirtHalDevice::new(
            DEV_ID_INPUT,
            "qemu-virt-virtio-input",
            0x0004, // Input / Event
            QEMU_VIRT_VIRTIO_BASE + 0x0200,
            0x0200,
            49,
        ));

        // 4. Register Display (VirtIO GPU / Framebuffer)
        self.register_device(QemuVirtHalDevice::new(
            DEV_ID_DISPLAY,
            "qemu-virt-framebuffer",
            0x0008, // Display / Video
            QEMU_VIRT_VIRTIO_BASE + 0x0400,
            0x0200,
            50,
        ));

        self.initialized = true;
    }

    pub fn register_device(&mut self, dev: QemuVirtHalDevice) -> bool {
        if self.device_count >= HAL_MAX_DEVICES {
            return false;
        }
        self.devices[self.device_count] = Some(dev);
        self.device_count += 1;
        true
    }

    pub fn get_device(&self, device_id: u32) -> Option<&QemuVirtHalDevice> {
        for slot in self.devices.iter().take(self.device_count) {
            if let Some(dev) = slot {
                if dev.device_id() == device_id {
                    return Some(dev);
                }
            }
        }
        None
    }

    pub fn get_device_mut(&mut self, device_id: u32) -> Option<&mut QemuVirtHalDevice> {
        for slot in self.devices.iter_mut().take(self.device_count) {
            if let Some(dev) = slot {
                if dev.device_id() == device_id {
                    return Some(dev);
                }
            }
        }
        None
    }

    pub fn device_count(&self) -> usize {
        self.device_count
    }

    /// Verifies that 100% of reference drivers (Console, Storage, Input, Display) are bound to HAL (`REQ-HAL-103`).
    pub fn all_reference_drivers_bound(&self) -> bool {
        let has_console = self.get_device(DEV_ID_CONSOLE).is_some();
        let has_storage = self.get_device(DEV_ID_STORAGE).is_some();
        let has_input = self.get_device(DEV_ID_INPUT).is_some();
        let has_display = self.get_device(DEV_ID_DISPLAY).is_some();
        has_console && has_storage && has_input && has_display
    }

    /// Executes the HAL Conformance Test Harness (`REQ-HAL-104`).
    pub fn run_conformance_suite(&mut self) -> Result<HalConformanceReport, HalError> {
        // Test 1: MMIO R/W & Barrier
        let mut mmio = QemuVirtMmio::new(0x0a00_0000, 0x1000);
        mmio.write_u32(0x00, 0x74726976)?; // Magic 'virt'
        mmio.write_u8(0x04, 0x02)?;        // Version
        mmio.barrier();
        let magic = mmio.read_u32(0x00)?;
        let ver = mmio.read_u8(0x04)?;
        let mmio_passed = magic == 0x74726976 && ver == 0x02;

        // Test 2: IRQ Controller Routing
        self.irq_controller.register_handler(48, 101)?;
        self.irq_controller.enable_irq(48)?;
        self.irq_controller.trigger_mock_irq(48);
        let acked_irq = self.irq_controller.ack_irq()?;
        self.irq_controller.eoi_irq(acked_irq)?;
        let irq_passed = acked_irq == 48;

        // Test 3: Coherent DMA Allocation & Alignment
        let (paddr, size) = self.dma_allocator.allocate_coherent(4096, 4096)?;
        self.dma_allocator.flush_cache(paddr, size)?;
        self.dma_allocator.invalidate_cache(paddr, size)?;
        let dma_passed = paddr >= QEMU_VIRT_DMA_BASE && (paddr % 4096 == 0) && size == 4096;

        // Test 4: Clock & Reset
        self.clock_reset.enable_clock(1)?;
        let freq = self.clock_reset.get_frequency_hz(1)?;
        self.clock_reset.assert_reset(2)?;
        self.clock_reset.deassert_reset(2)?;
        let clock_passed = freq == 24_000_000;

        // Test 5: Power State Transitions
        self.power.set_domain_power(0, PowerState::Standby)?;
        let standby = self.power.get_domain_power(0)?;
        self.power.set_domain_power(0, PowerState::Active)?;
        let active = self.power.get_domain_power(0)?;
        let power_passed = standby == PowerState::Standby && active == PowerState::Active;

        // Test 6: Device Enumeration
        let device_enum_passed = self.device_count() >= 4 && self.get_device(DEV_ID_CONSOLE).is_some();

        // Test 7: 100% of Reference Drivers Bound to HAL
        let drivers_bound_passed = self.all_reference_drivers_bound();

        let all_tests_passed = mmio_passed
            && irq_passed
            && dma_passed
            && clock_passed
            && power_passed
            && device_enum_passed
            && drivers_bound_passed;

        Ok(HalConformanceReport {
            mmio_passed,
            irq_passed,
            dma_passed,
            clock_passed,
            power_passed,
            device_enum_passed,
            drivers_bound_passed,
            all_tests_passed,
        })
    }
}

/// Global Hardware Abstraction Layer Manager Singleton
pub static HAL_MANAGER: Spinlock<HalManager> = Spinlock::new(HalManager::new());
