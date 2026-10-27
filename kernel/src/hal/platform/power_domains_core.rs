//! Kernel Power Domains & PSCI / RPMh Driver (SP-83, REQ-HW-108, REQ-PWR-103)
//!
//! Architectural Invariants:
//! - INV-PWR-001: ARM standard PSCI v1.1 SMC interface
//! - INV-PWR-002: Qualcomm RPMh RSC MMIO mapping (0x179c_0000)
//! - INV-PWR-003: Capability-gated power transitions (CAP_POWER_DOMAINS)

pub const KERNEL_HANOIP_RPMH_RSC_BASE: u64 = 0x179c_0000;
pub const KERNEL_PSCI_FN64_CPU_SUSPEND: u32 = 0xC400_0001;
pub const KERNEL_PSCI_FN_CPU_OFF: u32 = 0x8400_0002;
pub const KERNEL_PSCI_FN_SYSTEM_RESET: u32 = 0x8400_0009;
pub const KERNEL_CAP_POWER_DOMAINS: u32 = 0x0020_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPowerDomainState {
    Active,
    Retention,
    Suspend,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPowerDomainError {
    InvalidMmioBase,
    PermissionDenied,
    InvalidPsciFunction,
    TransitionFailed,
}

pub struct KernelPowerDomainsHarness {
    rsc_mapped: bool,
    core_states: [KernelPowerDomainState; 8],
    active_rpmh_cmds: u32,
}

impl KernelPowerDomainsHarness {
    pub const fn new() -> Self {
        Self {
            rsc_mapped: false,
            core_states: [KernelPowerDomainState::Active; 8],
            active_rpmh_cmds: 0,
        }
    }

    pub fn map_rsc(&mut self, base: u64) -> Result<(), KernelPowerDomainError> {
        if base != KERNEL_HANOIP_RPMH_RSC_BASE {
            return Err(KernelPowerDomainError::InvalidMmioBase);
        }
        self.rsc_mapped = true;
        Ok(())
    }

    pub fn set_core_power_state(&mut self, cap_mask: u32, core_id: usize, state: KernelPowerDomainState) -> Result<(), KernelPowerDomainError> {
        if (cap_mask & KERNEL_CAP_POWER_DOMAINS) == 0 {
            return Err(KernelPowerDomainError::PermissionDenied);
        }
        if core_id >= 8 {
            return Err(KernelPowerDomainError::TransitionFailed);
        }
        self.core_states[core_id] = state;
        self.active_rpmh_cmds += 1;
        Ok(())
    }

    pub fn execute_psci_call(&self, fn_id: u32) -> Result<i32, KernelPowerDomainError> {
        match fn_id {
            KERNEL_PSCI_FN64_CPU_SUSPEND => Ok(0),
            KERNEL_PSCI_FN_CPU_OFF => Ok(0),
            KERNEL_PSCI_FN_SYSTEM_RESET => Ok(0),
            _ => Err(KernelPowerDomainError::InvalidPsciFunction),
        }
    }

    pub fn is_rsc_mapped(&self) -> bool {
        self.rsc_mapped
    }

    pub fn core_state(&self, core_id: usize) -> KernelPowerDomainState {
        self.core_states[core_id]
    }

    pub fn active_rpmh_cmds(&self) -> u32 {
        self.active_rpmh_cmds
    }
}
