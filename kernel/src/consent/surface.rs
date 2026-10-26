//! Isolated Display Surface & Hardware Input Mediation (Consent Broker v1)
//! Compliant with Bible §8.9 (SP-21), §10, §14 and Invariant INV-CON-001.

use crate::task::ProcessId;

/// Display plane identifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneId {
    HardwareIsolated = 0,
    UntrustedUserspace = 1,
}

/// Kernel-mediated hardware-isolated UI surface enforcing INV-CON-001
pub struct IsolatedSurface {
    is_active: bool,
    active_plane: PlaneId,
    input_exclusive: bool,
    owner_pid: ProcessId,
}

impl IsolatedSurface {
    pub const fn new() -> Self {
        Self {
            is_active: false,
            active_plane: PlaneId::UntrustedUserspace,
            input_exclusive: false,
            owner_pid: 0,
        }
    }

    /// Activates the hardware-isolated consent display plane and locks input exclusively to the kernel
    pub fn activate(&mut self) -> Result<(), &'static str> {
        self.is_active = true;
        self.active_plane = PlaneId::HardwareIsolated;
        self.input_exclusive = true;
        self.owner_pid = 0; // Kernel / Trusted Consent Broker PID
        Ok(())
    }

    /// Deactivates the isolated plane and releases input routing back to userspace
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.active_plane = PlaneId::UntrustedUserspace;
        self.input_exclusive = false;
    }

    /// Returns whether the isolated secure display surface is currently active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Validates whether a process is permitted to draw or overlay on the current display plane (INV-CON-001)
    pub fn verify_surface_access(&self, caller_pid: ProcessId) -> Result<(), &'static str> {
        if self.is_active && caller_pid != self.owner_pid {
            return Err("EPERM: Access denied to isolated consent display surface (INV-CON-001)");
        }
        Ok(())
    }

    /// Validates whether a process is permitted to inject or deliver hardware input (INV-CON-001)
    pub fn verify_input_injection(&self, caller_pid: ProcessId) -> Result<(), &'static str> {
        if self.is_active && self.input_exclusive && caller_pid != self.owner_pid {
            return Err("EPERM: Synthetic input injection rejected during active consent prompt (INV-CON-001)");
        }
        Ok(())
    }
}
