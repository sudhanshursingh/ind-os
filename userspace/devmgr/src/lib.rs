//! Ind OS Device Manager Daemon (`devmgr`)
//!
//! Standards:
//! - REQ-USE-203: Device manager + hotplug events: capability handout
//! - INV-DEV-001: Zero ambient authority
//! - INV-DEV-002: Dynamic hotplug notification and capability revocation

#![no_std]

pub mod bus;
pub mod handout;
pub mod types;
pub mod virtio;
pub mod lab_prep;
pub mod device_evidence;
pub mod boot_chain;
pub mod hardware_core;
pub mod ufs;
pub mod ufs_resilience;
pub mod display_panel;
pub mod cpu_compositor;

pub use bus::{HotplugEvent, HotplugEventBus, Subscriber};
pub use handout::{CapabilityHandoutRegistry, DeviceCapToken, DmaCapToken, IrqCapToken, MmioCapToken};
pub use types::{BusType, DeviceError, DeviceId, DeviceState, DeviceType, HardwareDescriptor};
pub use virtio::VirtioDeviceManager;
pub use lab_prep::{
    HardwareLabHarness, LabPrepError, EXPECTED_UART_BAUD, SAFE_VOLTAGE_MAX_MV,
    SAFE_VOLTAGE_MIN_MV, THERMAL_CUTOFF_MILLICELSIUS,
};
pub use device_evidence::{
    DeviceEvidenceError, DeviceEvidenceHarness, PartitionDescriptor,
    MAX_PARTITION_RECORDS, MOTO_G60_DEVICE_CODENAME, MOTO_G60_SOC_ID,
};
pub use boot_chain::{
    BootChainError, BootChainHarness, BOOT_MAGIC, QUALCOMM_PAGE_SIZE,
};
pub use hardware_core::{
    HardwareCoreError, HardwareCoreHarness, HANOIP_DRAM_BASE,
    HANOIP_DRAM_SIZE_BYTES, HANOIP_GENI_UART_BASE, HANOIP_PDC_GIC_BASE,
};
pub use ufs::{
    UfsControllerHarness, UfsDriverError, CAP_STORAGE_ADMIN,
    HANOIP_UFS_BASE,
};
pub use ufs_resilience::{
    UfsJournalEntry, UfsJournalError, UfsJournalHarness,
    CAP_STORAGE_RECOVERY, SCSI_CMD_SYNCHRONIZE_CACHE,
};
pub use display_panel::{
    DisplayError, DisplayPanelHarness, DCS_CMD_DISPLAY_ON,
    DCS_CMD_PIXEL_FORMAT, DCS_CMD_SLEEP_OUT, HANOIP_MDSS_DSI0_BASE,
    PANEL_HEIGHT, PANEL_REFRESH_HZ, PANEL_WIDTH,
};
pub use cpu_compositor::{
    CompositorError, CpuCompositorHarness, DirtyRect,
    CAP_SURFACE_COMPOSITOR, MAX_PANEL_HEIGHT, MAX_PANEL_WIDTH,
};



