//! IndOS User-Mode DMA & IOMMU Memory Broker
//! Compliant with Bible §8.10 (SP-26), INDOS-SPEC-DRV-01, Rule N-01, Rule N-04, and Invariant INV-DRV-003.

use crate::capability::{CapRights, CapType, Capability};
use crate::task::ProcessId;

pub const MAX_DMA_WINDOWS: usize = 16;
pub const MAX_DMA_WINDOW_SIZE: usize = 64 * 1024; // 64 KiB max per window

/// DMA transfer direction constraints
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaDirection {
    HostToDevice,
    DeviceToHost,
    Bidirectional,
}

/// SMMU / IOMMU-isolated DMA memory window allocated to a user-mode driver
#[derive(Clone, Copy, Debug)]
pub struct DmaWindow {
    pub window_id: u32,
    pub owner_pid: ProcessId,
    pub buffer_phys_base: u64,
    pub buffer_iova_base: u64,
    pub size_bytes: usize,
    pub direction: DmaDirection,
    pub is_active: bool,
}

impl DmaWindow {
    pub const fn empty() -> Self {
        Self {
            window_id: 0,
            owner_pid: 0,
            buffer_phys_base: 0,
            buffer_iova_base: 0,
            size_bytes: 0,
            direction: DmaDirection::Bidirectional,
            is_active: false,
        }
    }
}

/// Central DMA broker managing IOMMU domains and granting capability-bounded DMA windows
pub struct DmaBroker {
    pub windows: [DmaWindow; MAX_DMA_WINDOWS],
    pub window_count: usize,
}

impl DmaBroker {
    pub const fn new() -> Self {
        Self {
            windows: [DmaWindow::empty(); MAX_DMA_WINDOWS],
            window_count: 0,
        }
    }

    /// Allocates an IOMMU-isolated DMA window for a user-mode driver process
    pub fn allocate_window(
        &mut self,
        owner_pid: ProcessId,
        phys_base: u64,
        iova_base: u64,
        size_bytes: usize,
        direction: DmaDirection,
    ) -> Result<u32, &'static str> {
        if self.window_count >= MAX_DMA_WINDOWS {
            return Err("ENOMEM: DMA broker window table full");
        }
        if size_bytes == 0 || size_bytes > MAX_DMA_WINDOW_SIZE {
            return Err("EINVAL: Invalid DMA window size (must be 1..64 KiB)");
        }
        // IOVA and physical base must be aligned to 64 bytes
        if (phys_base % 64 != 0) || (iova_base % 64 != 0) {
            return Err("EINVAL: DMA buffer addresses must be 64-byte aligned");
        }

        let win_id = (self.window_count + 1) as u32;
        let window = DmaWindow {
            window_id: win_id,
            owner_pid,
            buffer_phys_base: phys_base,
            buffer_iova_base: iova_base,
            size_bytes,
            direction,
            is_active: true,
        };

        self.windows[self.window_count] = window;
        self.window_count += 1;
        Ok(win_id)
    }

    /// Validates a requested DMA transfer against capability rights and bounds (INV-DRV-003)
    pub fn validate_transfer(
        &self,
        window_id: u32,
        caller_pid: ProcessId,
        offset: usize,
        len: usize,
        transfer_dir: DmaDirection,
        cap: &Capability,
    ) -> Result<(), &'static str> {
        // Enforce INV-DRV-003: Capability verification
        if !cap.is_valid {
            return Err("EPERM: Access denied: Invalid DMA capability token (INV-DRV-003)");
        }
        if cap.cap_type != CapType::DmaWindow(window_id) {
            return Err("EPERM: Access denied: Capability does not grant access to this DMA window (INV-DRV-003)");
        }

        // Direction rights checking
        match transfer_dir {
            DmaDirection::HostToDevice => {
                if !cap.rights.contains(CapRights::READ) {
                    return Err("EPERM: Access denied: HostToDevice requires READ rights (INV-DRV-003)");
                }
            }
            DmaDirection::DeviceToHost => {
                if !cap.rights.contains(CapRights::WRITE) {
                    return Err("EPERM: Access denied: DeviceToHost requires WRITE rights (INV-DRV-003)");
                }
            }
            DmaDirection::Bidirectional => {
                if !cap.rights.contains(CapRights::READ.union(CapRights::WRITE)) {
                    return Err("EPERM: Access denied: Bidirectional DMA requires READ+WRITE rights (INV-DRV-003)");
                }
            }
        }

        let slot = (window_id as usize).checked_sub(1).ok_or("EINVAL: Invalid window id")?;
        if slot >= self.window_count || self.windows[slot].window_id != window_id {
            return Err("ENOENT: DMA window not found");
        }

        let win = &self.windows[slot];
        if !win.is_active {
            return Err("EBUSY: DMA window is revoked or inactive");
        }
        if win.owner_pid != caller_pid {
            return Err("EPERM: Caller is not the registered owner of this DMA window");
        }

        // Verify transfer direction is compatible with window direction
        if win.direction != DmaDirection::Bidirectional && win.direction != transfer_dir {
            return Err("EPERM: Transfer direction violates DMA window configuration");
        }

        // Enforce Bounds Checking (INV-DRV-003: No out-of-bounds DMA tampering)
        let end_offset = offset.checked_add(len).ok_or("EINVAL: Integer overflow in DMA offset")?;
        if end_offset > win.size_bytes {
            return Err("EINVAL: DMA buffer bounds violation: transfer exceeds window size (INV-DRV-003)");
        }

        Ok(())
    }

    /// Revokes an allocated DMA window, removing access
    pub fn revoke_window(
        &mut self,
        window_id: u32,
        owner_pid: ProcessId,
        cap: &Capability,
    ) -> Result<(), &'static str> {
        if !cap.is_valid || cap.cap_type != CapType::DmaWindow(window_id) {
            return Err("EPERM: Invalid DMA capability");
        }

        let slot = (window_id as usize).checked_sub(1).ok_or("EINVAL: Invalid window id")?;
        if slot >= self.window_count || self.windows[slot].window_id != window_id {
            return Err("ENOENT: DMA window not found");
        }

        let win = &mut self.windows[slot];
        if win.owner_pid != owner_pid {
            return Err("EPERM: Caller is not the owner of this DMA window");
        }

        win.is_active = false;
        Ok(())
    }

    /// Retrieves window descriptor
    pub fn get_window(&self, window_id: u32) -> Option<&DmaWindow> {
        let slot = (window_id as usize).checked_sub(1)?;
        if slot < self.window_count && self.windows[slot].window_id == window_id {
            Some(&self.windows[slot])
        } else {
            None
        }
    }
}
