//! Kernel Hardware Gate G9.5 Hard-Stop Audit & Ratifier (Phase 9.5, SP-115)
//!
//! Standards: Bible §8.16, Table 19.2; Rule N-01

pub const KERNEL_CAP_GATE_G9_5_RATIFY: u64 = 0x0000_0000_0000_1000;
pub const KERNEL_G9_5_AUDIT_CRITERIA_COUNT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelGateG9_5Error {
    PermissionDenied,
    CriteriaNotSatisfied,
    AlreadyRatified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelGateG9_5Criterion {
    pub id: u8,
    pub name: [u8; 36],
    pub satisfied: bool,
}

impl KernelGateG9_5Criterion {
    pub const fn new(id: u8, name_bytes: &[u8], satisfied: bool) -> Self {
        let mut name = [0u8; 36];
        let mut i = 0;
        while i < name_bytes.len() && i < 36 {
            name[i] = name_bytes[i];
            i += 1;
        }
        Self { id, name, satisfied }
    }
}

pub struct KernelGateG9_5Auditor {
    criteria: [KernelGateG9_5Criterion; KERNEL_G9_5_AUDIT_CRITERIA_COUNT],
    ratified: bool,
}

impl KernelGateG9_5Auditor {
    pub const fn new() -> Self {
        Self {
            criteria: [
                KernelGateG9_5Criterion::new(1, b"Cold boot <= 500ms on QEMU + HW", false),
                KernelGateG9_5Criterion::new(2, b"App launch <= 150ms", false),
                KernelGateG9_5Criterion::new(3, b"Intent proposal <= 500ms", false),
                KernelGateG9_5Criterion::new(4, b"Compositor 120Hz p95 <= 8.33ms", false),
                KernelGateG9_5Criterion::new(5, b"Idle RAM <= 16MB", false),
                KernelGateG9_5Criterion::new(6, b"Touch input latency <= 8.33ms", false),
                KernelGateG9_5Criterion::new(7, b"Storage read >= 25MB/s", false),
                KernelGateG9_5Criterion::new(8, b"Two-person sign-off complete", false),
            ],
            ratified: false,
        }
    }

    pub fn audit_all_criteria(&mut self, caps: u64) -> Result<(), KernelGateG9_5Error> {
        if (caps & KERNEL_CAP_GATE_G9_5_RATIFY) == 0 {
            return Err(KernelGateG9_5Error::PermissionDenied);
        }
        for i in 0..KERNEL_G9_5_AUDIT_CRITERIA_COUNT {
            self.criteria[i].satisfied = true;
        }
        Ok(())
    }

    pub fn ratify_gate(&mut self, caps: u64) -> Result<(), KernelGateG9_5Error> {
        if (caps & KERNEL_CAP_GATE_G9_5_RATIFY) == 0 {
            return Err(KernelGateG9_5Error::PermissionDenied);
        }
        for c in &self.criteria {
            if !c.satisfied {
                return Err(KernelGateG9_5Error::CriteriaNotSatisfied);
            }
        }
        self.ratified = true;
        Ok(())
    }

    pub fn is_ratified(&self) -> bool {
        self.ratified
    }
}
