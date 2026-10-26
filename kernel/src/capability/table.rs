//! Per-Process Capability Table and Slot Management for IndK
//! Compliant with Bible §8.8 / §10 and Machine Invariants INV-CAP-001 & INV-CAP-002.

use crate::capability::rights::CapRights;
use crate::object::{KObjectId, KObjectTable};
use crate::syscall::{EBADF, EINVAL, ENOMEM, EPERM};
use crate::task::ProcessId;

pub type CapHandle = u32;
pub const MAX_CAPABILITIES: usize = 64;

/// Typed kernel object references guarded by capabilities
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapType {
    Empty,
    Object(KObjectId), // Typed first-class kernel object handle (INV-CAP-003)
    Endpoint(u32),     // Synchronous IPC Endpoint ID
    Notification(u32), // Asynchronous notification ID
    Process(ProcessId),
    MemoryRegion { base: usize, size: usize },
    SyscallBitmap(u64),
    File(u64),         // Capability-mediated VFS node ID (REQ-KRN-106)
    Device(u32),       // Capability-governed device binding (REQ-KRN-107, INV-DRV-001)
    Interrupt(u32),    // Capability-authenticated IRQ subscription & ACK (INV-DRV-002)
    DmaWindow(u32),    // Capability-bounded DMA/IOMMU memory window (INV-DRV-003)
}

/// Unforgeable capability access control token
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capability {
    pub cap_type: CapType,
    pub rights: CapRights,
    pub badge: u64,
    pub is_valid: bool,
    pub parent_slot: Option<u32>, // Lineage tracking within table (INV-CAP-002)
    pub parent_pid: Option<ProcessId>, // Process ID of parent capability (INV-CAP-002)
    pub parent_handle: Option<CapHandle>, // Handle in parent's namespace (INV-CAP-002)
}

impl Capability {
    pub const fn empty() -> Self {
        Self {
            cap_type: CapType::Empty,
            rights: CapRights::NONE,
            badge: 0,
            is_valid: false,
            parent_slot: None,
            parent_pid: None,
            parent_handle: None,
        }
    }

    pub const fn new(cap_type: CapType, rights: CapRights, badge: u64) -> Self {
        Self {
            cap_type,
            rights,
            badge,
            is_valid: true,
            parent_slot: None,
            parent_pid: None,
            parent_handle: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CapTable {
    pub slots: [Capability; MAX_CAPABILITIES],
}

impl CapTable {
    pub const fn new() -> Self {
        Self {
            slots: [Capability::empty(); MAX_CAPABILITIES],
        }
    }

    /// Allocates the first available empty slot and stores the capability
    pub fn insert(&mut self, cap: Capability) -> Result<CapHandle, i64> {
        for i in 0..MAX_CAPABILITIES {
            if !self.slots[i].is_valid {
                self.slots[i] = cap;
                return Ok(i as CapHandle);
            }
        }
        Err(ENOMEM)
    }

    /// Inserts a capability into a designated slot index
    pub fn insert_at(&mut self, slot_idx: usize, cap: Capability) -> Result<CapHandle, i64> {
        if slot_idx >= MAX_CAPABILITIES {
            return Err(EINVAL);
        }
        self.slots[slot_idx] = cap;
        Ok(slot_idx as CapHandle)
    }

    /// Looks up a capability by handle, returning an immutable reference
    pub fn get(&self, handle: CapHandle) -> Result<&Capability, i64> {
        let idx = handle as usize;
        if idx >= MAX_CAPABILITIES {
            return Err(EBADF);
        }
        let cap = &self.slots[idx];
        if !cap.is_valid {
            return Err(EBADF);
        }
        Ok(cap)
    }

    /// Looks up a capability by handle, returning a mutable reference
    pub fn get_mut(&mut self, handle: CapHandle) -> Result<&mut Capability, i64> {
        let idx = handle as usize;
        if idx >= MAX_CAPABILITIES {
            return Err(EBADF);
        }
        let cap = &mut self.slots[idx];
        if !cap.is_valid {
            return Err(EBADF);
        }
        Ok(cap)
    }

    /// Mints a derived capability with attenuated rights (INV-CAP-001)
    /// Monotonic Attenuation: target_rights MUST be a subset of parent rights.
    pub fn mint(
        &mut self,
        src_handle: CapHandle,
        target_rights: CapRights,
        badge: u64,
    ) -> Result<CapHandle, i64> {
        let cap_type = {
            let src = self.get(src_handle)?;
            // Check that source capability has GRANT authority
            if !src.rights.contains(CapRights::GRANT) {
                return Err(EPERM);
            }
            // Enforce Monotonic Attenuation (INV-CAP-001):
            // Cannot grant rights not possessed by source capability!
            if !src.rights.can_attenuate_to(target_rights) {
                return Err(EPERM);
            }
            src.cap_type
        };

        let mut derived_cap = Capability::new(cap_type, target_rights, badge);
        derived_cap.parent_slot = Some(src_handle);

        self.insert(derived_cap)
    }

    /// Mints a derived capability with monotonic rights attenuation and object reference tracking (INV-CAP-001, INV-CAP-003)
    pub fn mint_object_cap(
        &mut self,
        src_handle: CapHandle,
        target_rights: CapRights,
        badge: u64,
        obj_table: &mut KObjectTable,
    ) -> Result<CapHandle, i64> {
        let cap_type = {
            let src = self.get(src_handle)?;
            if !src.rights.contains(CapRights::GRANT) {
                return Err(EPERM);
            }
            if !src.rights.can_attenuate_to(target_rights) {
                return Err(EPERM);
            }
            src.cap_type
        };

        if let CapType::Object(kobj_id) = cap_type {
            obj_table.retain(kobj_id)?;
        }

        let mut derived_cap = Capability::new(cap_type, target_rights, badge);
        derived_cap.parent_slot = Some(src_handle);

        match self.insert(derived_cap) {
            Ok(h) => Ok(h),
            Err(e) => {
                if let CapType::Object(kobj_id) = cap_type {
                    let _ = obj_table.release(kobj_id);
                }
                Err(e)
            }
        }
    }

    /// Revokes a capability and recursively invalidates all derived child capabilities (INV-CAP-002)
    /// Returns the total number of capability slots invalidated.
    pub fn revoke(&mut self, handle: CapHandle) -> Result<usize, i64> {
        let idx = handle as usize;
        if idx >= MAX_CAPABILITIES || !self.slots[idx].is_valid {
            return Err(EBADF);
        }

        // Invalidate root slot
        self.slots[idx].is_valid = false;
        self.slots[idx].cap_type = CapType::Empty;
        self.slots[idx].rights = CapRights::NONE;
        let mut revoked_count = 1;

        // Cascade revocation to all child slots (INV-CAP-002 Revocation Soundness)
        let mut cascade = true;
        while cascade {
            cascade = false;
            for i in 0..MAX_CAPABILITIES {
                if self.slots[i].is_valid {
                    if let Some(parent) = self.slots[i].parent_slot {
                        let parent_idx = parent as usize;
                        if parent_idx < MAX_CAPABILITIES && !self.slots[parent_idx].is_valid {
                            self.slots[i].is_valid = false;
                            self.slots[i].cap_type = CapType::Empty;
                            self.slots[i].rights = CapRights::NONE;
                            revoked_count += 1;
                            cascade = true; // continue checking downstream dependencies
                        }
                    }
                }
            }
        }

        Ok(revoked_count)
    }

    /// Revokes a capability and recursively invalidates all derived child capabilities (INV-CAP-002),
    /// releasing KObject references in KObjectTable (INV-CAP-003).
    /// Returns the total number of capability slots invalidated.
    pub fn revoke_with_objects(
        &mut self,
        handle: CapHandle,
        obj_table: &mut KObjectTable,
    ) -> Result<usize, i64> {
        let idx = handle as usize;
        if idx >= MAX_CAPABILITIES || !self.slots[idx].is_valid {
            return Err(EBADF);
        }

        let mut to_revoke = [false; MAX_CAPABILITIES];
        to_revoke[idx] = true;

        let mut cascade = true;
        while cascade {
            cascade = false;
            for i in 0..MAX_CAPABILITIES {
                if !to_revoke[i] && self.slots[i].is_valid {
                    if let Some(parent) = self.slots[i].parent_slot {
                        let parent_idx = parent as usize;
                        if parent_idx < MAX_CAPABILITIES && to_revoke[parent_idx] {
                            to_revoke[i] = true;
                            cascade = true;
                        }
                    }
                }
            }
        }

        let mut revoked_count = 0;
        for i in 0..MAX_CAPABILITIES {
            if to_revoke[i] && self.slots[i].is_valid {
                let old_type = self.slots[i].cap_type;
                self.slots[i].is_valid = false;
                self.slots[i].cap_type = CapType::Empty;
                self.slots[i].rights = CapRights::NONE;
                self.slots[i].parent_slot = None;
                self.slots[i].parent_pid = None;
                self.slots[i].parent_handle = None;
                revoked_count += 1;

                if let CapType::Object(kobj_id) = old_type {
                    let _ = obj_table.release(kobj_id);
                }
            }
        }

        Ok(revoked_count)
    }

    /// Delegates a capability to another process's capability table with monotonic rights attenuation
    pub fn delegate_to(
        &self,
        src_handle: CapHandle,
        target_rights: CapRights,
        target_table: &mut CapTable,
    ) -> Result<CapHandle, i64> {
        let src = self.get(src_handle)?;
        if !src.rights.contains(CapRights::GRANT) {
            return Err(EPERM);
        }
        if !src.rights.can_attenuate_to(target_rights) {
            return Err(EPERM);
        }

        let delegated_cap = Capability::new(src.cap_type, target_rights, src.badge);
        target_table.insert(delegated_cap)
    }

    /// Delegates an object capability to another capability table with monotonic rights attenuation
    /// and object reference count increment (INV-CAP-001, INV-CAP-003)
    pub fn delegate_object_to(
        &self,
        src_handle: CapHandle,
        target_rights: CapRights,
        target_table: &mut CapTable,
        obj_table: &mut KObjectTable,
    ) -> Result<CapHandle, i64> {
        let src = self.get(src_handle)?;
        if !src.rights.contains(CapRights::GRANT) {
            return Err(EPERM);
        }
        if !src.rights.can_attenuate_to(target_rights) {
            return Err(EPERM);
        }

        if let CapType::Object(kobj_id) = src.cap_type {
            obj_table.retain(kobj_id)?;
        }

        let delegated_cap = Capability::new(src.cap_type, target_rights, src.badge);
        match target_table.insert(delegated_cap) {
            Ok(h) => Ok(h),
            Err(e) => {
                if let CapType::Object(kobj_id) = src.cap_type {
                    let _ = obj_table.release(kobj_id);
                }
                Err(e)
            }
        }
    }

    /// Returns the number of currently active capability slots
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_valid).count()
    }
}
