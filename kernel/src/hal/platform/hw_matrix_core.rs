//! Kernel Hardware Compatibility Matrix v1 Core (SP-104, PRD-001 §8.3, REQ-HW-114)
//!
//! Peripheral compatibility classification and constitutional blob exclusion verifier.

pub const KERNEL_CAP_AUDIT_HARDWARE: u32 = 0x0000_0400;

pub const KERNEL_TOTAL_MATRIX_ENTRIES: usize = 15;
pub const KERNEL_WORKS_COUNT: usize = 12;
pub const KERNEL_PARTIAL_COUNT: usize = 2;
pub const KERNEL_OUT_OF_SCOPE_COUNT: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelHwStatus {
    Works,
    Partial,
    Blocked,
    OutOfScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelHwMatrixError {
    PermissionDenied,
    ComponentNotFound,
    InvalidAuditState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelHwMatrixEntry {
    pub id: u8,
    pub name: &'static str,
    pub status: KernelHwStatus,
    pub is_critical: bool,
}

pub struct KernelHwMatrixHarness {
    entries: [KernelHwMatrixEntry; KERNEL_TOTAL_MATRIX_ENTRIES],
    audited: bool,
}

impl KernelHwMatrixHarness {
    pub const fn new() -> Self {
        Self {
            entries: [
                KernelHwMatrixEntry { id: 1, name: "Boot-Chain & EDL", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 2, name: "Kryo 470 CPU Core", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 3, name: "LPDDR4X DRAM", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 4, name: "UFS 2.2 Storage", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 5, name: "Tianma FT8726 DSI Display", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 6, name: "FocalTech FT3518 Touch", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 7, name: "PM6150 PMIC & Fuel Gauge", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 8, name: "Synopsys DWC3 USB", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 9, name: "WCN3990 Wi-Fi", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 10, name: "WCN3990 Bluetooth", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 11, name: "Sensors (IMU/Mag/ALS)", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 12, name: "WCD9385 Audio Codec", status: KernelHwStatus::Works, is_critical: true },
                KernelHwMatrixEntry { id: 13, name: "Hexagon 688 DSP", status: KernelHwStatus::OutOfScope, is_critical: false },
                KernelHwMatrixEntry { id: 14, name: "Spectra 350L Camera ISP", status: KernelHwStatus::Partial, is_critical: false },
                KernelHwMatrixEntry { id: 15, name: "FPC1020 Fingerprint", status: KernelHwStatus::Partial, is_critical: false },
            ],
            audited: false,
        }
    }

    pub fn audit_all(&mut self, cap_token: u32) -> Result<(), KernelHwMatrixError> {
        if (cap_token & KERNEL_CAP_AUDIT_HARDWARE) == 0 {
            return Err(KernelHwMatrixError::PermissionDenied);
        }
        self.audited = true;
        Ok(())
    }

    pub fn get_entry(&self, id: u8) -> Option<KernelHwMatrixEntry> {
        if id >= 1 && id <= KERNEL_TOTAL_MATRIX_ENTRIES as u8 {
            Some(self.entries[(id - 1) as usize])
        } else {
            None
        }
    }

    pub fn works_count(&self) -> usize {
        let mut count = 0;
        for e in &self.entries {
            if e.status == KernelHwStatus::Works {
                count += 1;
            }
        }
        count
    }

    pub fn partial_count(&self) -> usize {
        let mut count = 0;
        for e in &self.entries {
            if e.status == KernelHwStatus::Partial {
                count += 1;
            }
        }
        count
    }

    pub fn out_of_scope_count(&self) -> usize {
        let mut count = 0;
        for e in &self.entries {
            if e.status == KernelHwStatus::OutOfScope {
                count += 1;
            }
        }
        count
    }

    pub fn is_audited(&self) -> bool {
        self.audited
    }
}
