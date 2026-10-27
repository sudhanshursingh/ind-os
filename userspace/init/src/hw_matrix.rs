//! Ind OS Hardware Compatibility Matrix v1 Auditor (SP-104)
//!
//! Standards: Bible §6.3, §8.15, Table 19.2; PRD-001 §8.3, REQ-HW-114
//! Invariants: INV-HW-MAT-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Pure memory-safe Rust

pub const CAP_AUDIT_HARDWARE: u64 = 0x0000_0000_0000_0400;

pub const TOTAL_MATRIX_ENTRIES: usize = 15;
pub const WORKS_COUNT: usize = 12;
pub const PARTIAL_COUNT: usize = 2;
pub const OUT_OF_SCOPE_COUNT: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwStatus {
    Works,
    Partial,
    Blocked,
    OutOfScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwMatrixError {
    PermissionDenied,
    ComponentNotFound,
    InvalidAuditState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HwMatrixEntry {
    pub id: u8,
    pub name: &'static str,
    pub status: HwStatus,
    pub is_critical: bool,
}

pub struct HwMatrixAuditor {
    entries: [HwMatrixEntry; TOTAL_MATRIX_ENTRIES],
    audited: bool,
}

impl HwMatrixAuditor {
    pub const fn new() -> Self {
        Self {
            entries: [
                HwMatrixEntry { id: 1, name: "Boot-Chain & EDL", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 2, name: "Kryo 470 CPU Core", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 3, name: "LPDDR4X DRAM", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 4, name: "UFS 2.2 Storage", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 5, name: "Tianma FT8726 DSI Display", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 6, name: "FocalTech FT3518 Touch", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 7, name: "PM6150 PMIC & Fuel Gauge", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 8, name: "Synopsys DWC3 USB", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 9, name: "WCN3990 Wi-Fi", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 10, name: "WCN3990 Bluetooth", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 11, name: "Sensors (IMU/Mag/ALS)", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 12, name: "WCD9385 Audio Codec", status: HwStatus::Works, is_critical: true },
                HwMatrixEntry { id: 13, name: "Hexagon 688 DSP", status: HwStatus::OutOfScope, is_critical: false },
                HwMatrixEntry { id: 14, name: "Spectra 350L Camera ISP", status: HwStatus::Partial, is_critical: false },
                HwMatrixEntry { id: 15, name: "FPC1020 Fingerprint", status: HwStatus::Partial, is_critical: false },
            ],
            audited: false,
        }
    }

    pub fn audit_all(&mut self, caps: u64) -> Result<(), HwMatrixError> {
        if (caps & CAP_AUDIT_HARDWARE) == 0 {
            return Err(HwMatrixError::PermissionDenied);
        }
        self.audited = true;
        Ok(())
    }

    pub fn get_entry(&self, id: u8) -> Option<HwMatrixEntry> {
        if id >= 1 && id <= TOTAL_MATRIX_ENTRIES as u8 {
            Some(self.entries[(id - 1) as usize])
        } else {
            None
        }
    }

    pub fn works_count(&self) -> usize {
        let mut count = 0;
        for e in &self.entries {
            if e.status == HwStatus::Works {
                count += 1;
            }
        }
        count
    }

    pub fn partial_count(&self) -> usize {
        let mut count = 0;
        for e in &self.entries {
            if e.status == HwStatus::Partial {
                count += 1;
            }
        }
        count
    }

    pub fn out_of_scope_count(&self) -> usize {
        let mut count = 0;
        for e in &self.entries {
            if e.status == HwStatus::OutOfScope {
                count += 1;
            }
        }
        count
    }

    pub fn is_audited(&self) -> bool {
        self.audited
    }
}
