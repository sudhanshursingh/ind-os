//! Kernel Device Evidence Pack & Recovery Verification (SP-71, REQ-HW-102, Rule N-06)
//!
//! Architectural Invariants:
//! - INV-EVID-001: Device identity integrity (Moto G60 `hanoip`, SM7150-AC)
//! - INV-EVID-002: Partition map & cryptographic hash pinning
//! - INV-EVID-003: Non-destructive recovery validation

pub const KERNEL_MOTO_G60_DEVICE_CODENAME: &str = "hanoip";
pub const KERNEL_MOTO_G60_SOC_ID: &str = "SM7150-AC";
pub const KERNEL_MAX_PARTITION_RECORDS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelDeviceEvidenceError {
    DeviceMismatch,
    PartitionNotFound,
    HashVerificationFailed,
    TableFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelPartitionDescriptor {
    pub name: &'static str,
    pub size_kb: u64,
    pub is_ab_slot: bool,
    pub sha256_hash: &'static str,
}

pub struct KernelDeviceEvidenceHarness {
    device_verified: bool,
    partitions: [Option<KernelPartitionDescriptor>; KERNEL_MAX_PARTITION_RECORDS],
    partition_count: usize,
    recovery_image_verified: bool,
}

impl KernelDeviceEvidenceHarness {
    pub const fn new() -> Self {
        Self {
            device_verified: false,
            partitions: [None; KERNEL_MAX_PARTITION_RECORDS],
            partition_count: 0,
            recovery_image_verified: false,
        }
    }

    pub fn verify_device_identity(&mut self, codename: &str, soc: &str) -> Result<(), KernelDeviceEvidenceError> {
        let codename_matches = codename.len() == KERNEL_MOTO_G60_DEVICE_CODENAME.len()
            && codename.as_bytes() == KERNEL_MOTO_G60_DEVICE_CODENAME.as_bytes();
        let soc_matches = soc.len() == KERNEL_MOTO_G60_SOC_ID.len()
            && soc.as_bytes() == KERNEL_MOTO_G60_SOC_ID.as_bytes();

        if !codename_matches || !soc_matches {
            return Err(KernelDeviceEvidenceError::DeviceMismatch);
        }
        self.device_verified = true;
        Ok(())
    }

    pub fn record_partition(
        &mut self,
        name: &'static str,
        size_kb: u64,
        is_ab_slot: bool,
        sha256_hash: &'static str,
    ) -> Result<(), KernelDeviceEvidenceError> {
        for slot in self.partitions.iter_mut() {
            if slot.is_none() {
                *slot = Some(KernelPartitionDescriptor {
                    name,
                    size_kb,
                    is_ab_slot,
                    sha256_hash,
                });
                self.partition_count += 1;
                return Ok(());
            }
        }
        Err(KernelDeviceEvidenceError::TableFull)
    }

    pub fn verify_recovery_image(&mut self, image_hash: &'static str, expected_hash: &'static str) -> Result<(), KernelDeviceEvidenceError> {
        if image_hash.as_bytes() != expected_hash.as_bytes() {
            return Err(KernelDeviceEvidenceError::HashVerificationFailed);
        }
        self.recovery_image_verified = true;
        Ok(())
    }

    pub fn device_verified(&self) -> bool {
        self.device_verified
    }

    pub fn partition_count(&self) -> usize {
        self.partition_count
    }

    pub fn recovery_image_verified(&self) -> bool {
        self.recovery_image_verified
    }
}
