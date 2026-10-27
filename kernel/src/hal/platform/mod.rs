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






