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
pub mod touch_controller;
pub mod input_pipeline;
pub mod pmic_spmi;
pub mod battery_gauge;
pub mod thermal_mgmt;
pub mod power_domains;
pub mod usb_dwc3;
pub mod usb_role;

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
pub use touch_controller::{
    TouchAction, TouchControllerHarness, TouchError, TouchPoint,
    HANOIP_GENI_I2C1_BASE, TOUCH_I2C_SLAVE_ADDR, TOUCH_QUEUE_CAPACITY,
};
pub use input_pipeline::{
    Gesture, InputPipelineError, InputPipelineHarness,
    CAP_INPUT_SHELL,
};
pub use pmic_spmi::{
    PmicError, PmicSpmiHarness, CAP_POWER_MGMT,
    HANOIP_SPMI_BASE, PMIC_PM7150L_SID, PMIC_PM7150_SID,
};
pub use battery_gauge::{
    BatteryError, BatteryGaugeHarness, ChargingState,
    BATT_CRITICAL_SOC_PCT, BATT_MAX_VOLTAGE_MV, BATT_MIN_VOLTAGE_MV,
};
pub use thermal_mgmt::{
    ThermalError, ThermalMgmtHarness, ThermalZone,
    HANOIP_TSENS0_BASE, THERMAL_SHUTDOWN_MILLICELSIUS,
    THERMAL_THROTTLE_MILLICELSIUS, THERMAL_WARN_MILLICELSIUS,
};
pub use power_domains::{
    PowerDomainError, PowerDomainState, PowerDomainsHarness,
    CAP_POWER_DOMAINS, HANOIP_RPMH_RSC_BASE, PSCI_FN64_CPU_SUSPEND,
    PSCI_FN_CPU_OFF, PSCI_FN_SYSTEM_RESET,
};
pub use usb_dwc3::{
    UsbDwc3Harness, UsbEndpointState, UsbError, UsbSpeed,
    CAP_USB_ADMIN, HANOIP_QUSB2_PHY_BASE, HANOIP_USB_DWC3_BASE,
    USB_DESC_TYPE_DEVICE, USB_REQ_GET_DESCRIPTOR, USB_REQ_SET_ADDRESS,
};
pub use usb_role::{
    UsbRole, UsbRoleError, UsbRoleHarness,
    CAP_USB_ROLE_SWITCH, CDC_ACM_BAUD_DEFAULT, CDC_ACM_LINE_CODING_LEN,
    USB_ROLE_DEVICE, USB_ROLE_HOST,
};









