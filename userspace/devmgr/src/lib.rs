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
