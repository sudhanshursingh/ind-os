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
