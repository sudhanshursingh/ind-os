//! Per-Process Capability Namespace (CSpace) Management for IndK
//! Compliant with Bible §8.9 / §14 and Machine Invariants:
//! - INV-CAP-001: Monotonic Rights Attenuation
//! - INV-CAP-002: Revocation Soundness & Cascading Lineage Tracking
//! - INV-CAP-003: Object-Table Reference Coherence
//! - INV-CAP-004: Strict Namespace Isolation (Handles are process-local indices)

use crate::capability::rights::CapRights;
use crate::capability::table::{CapHandle, CapTable, CapType, Capability, MAX_CAPABILITIES};
use crate::object::{KObjectId, KObjectTable};
use crate::syscall::{EBADF, EEXIST, ENOMEM, EPERM};
use crate::task::ProcessId;

pub const MAX_NAMESPACES: usize = 16;

/// A per-process Capability Namespace (CSpace)
#[derive(Clone, Copy)]
pub struct CapNamespace {
    pub process_id: ProcessId,
    pub table: CapTable,
    pub badge_mask: u64,
    pub is_allocated: bool,
}

impl CapNamespace {
    pub const fn empty() -> Self {
        Self {
            process_id: 0,
            table: CapTable::new(),
            badge_mask: 0,
            is_allocated: false,
        }
    }

    pub fn new(process_id: ProcessId, badge_mask: u64) -> Self {
        Self {
            process_id,
            table: CapTable::new(),
            badge_mask,
            is_allocated: true,
        }
    }

    /// Looks up a capability in this namespace by process-local handle (INV-CAP-004)
    pub fn lookup(&self, handle: CapHandle) -> Result<&Capability, i64> {
        self.table.get(handle)
    }

    /// Looks up a capability mutably
    pub fn lookup_mut(&mut self, handle: CapHandle) -> Result<&mut Capability, i64> {
        self.table.get_mut(handle)
    }

    /// Installs a capability into this namespace
    pub fn install(&mut self, cap: Capability) -> Result<CapHandle, i64> {
        self.table.insert(cap)
    }

    /// Installs an object capability, incrementing the object's reference count (INV-CAP-003)
    pub fn install_object(
        &mut self,
        kobj_id: KObjectId,
        rights: CapRights,
        badge: u64,
        obj_table: &mut KObjectTable,
    ) -> Result<CapHandle, i64> {
        obj_table.retain(kobj_id)?;
        let cap = Capability::new(CapType::Object(kobj_id), rights, badge);
        match self.table.insert(cap) {
            Ok(h) => Ok(h),
            Err(e) => {
                let _ = obj_table.release(kobj_id);
                Err(e)
            }
        }
    }

    /// Mints a derived capability within this namespace with rights attenuation (INV-CAP-001)
    pub fn mint(
        &mut self,
        src_handle: CapHandle,
        target_rights: CapRights,
        badge: u64,
        mut obj_table: Option<&mut KObjectTable>,
    ) -> Result<CapHandle, i64> {
        let h = if let Some(ref mut objs) = obj_table {
            self.table.mint_object_cap(src_handle, target_rights, badge, objs)?
        } else {
            self.table.mint(src_handle, target_rights, badge)?
        };
        if let Ok(slot) = self.table.get_mut(h) {
            slot.parent_pid = Some(self.process_id);
            slot.parent_handle = Some(src_handle);
        }
        Ok(h)
    }

    /// Revokes a capability and cascades to all child capabilities, releasing KObjects if applicable (INV-CAP-002, INV-CAP-003)
    pub fn revoke(
        &mut self,
        handle: CapHandle,
        mut obj_table: Option<&mut KObjectTable>,
    ) -> Result<usize, i64> {
        if let Some(ref mut objs) = obj_table {
            self.table.revoke_with_objects(handle, objs)
        } else {
            self.table.revoke(handle)
        }
    }

    /// Delegates a capability from this namespace to a target namespace (INV-CAP-001, INV-CAP-003, INV-CAP-004)
    pub fn delegate_to(
        &self,
        src_handle: CapHandle,
        target_rights: CapRights,
        target_ns: &mut CapNamespace,
        mut obj_table: Option<&mut KObjectTable>,
    ) -> Result<CapHandle, i64> {
        let h = if let Some(ref mut objs) = obj_table {
            self.table.delegate_object_to(src_handle, target_rights, &mut target_ns.table, objs)?
        } else {
            self.table.delegate_to(src_handle, target_rights, &mut target_ns.table)?
        };
        if let Ok(slot) = target_ns.table.get_mut(h) {
            slot.parent_pid = Some(self.process_id);
            slot.parent_handle = Some(src_handle);
        }
        Ok(h)
    }

    /// Returns the number of currently active capability slots
    pub fn active_count(&self) -> usize {
        self.table.active_count()
    }
}

/// Global registry of per-process Capability Namespaces
pub struct NamespaceRegistry {
    pub namespaces: [CapNamespace; MAX_NAMESPACES],
}

impl NamespaceRegistry {
    pub const fn new() -> Self {
        Self {
            namespaces: [CapNamespace::empty(); MAX_NAMESPACES],
        }
    }

    /// Finds index of namespace for pid
    pub fn find_namespace_idx(&self, pid: ProcessId) -> Option<usize> {
        for (i, ns) in self.namespaces.iter().enumerate() {
            if ns.is_allocated && ns.process_id == pid {
                return Some(i);
            }
        }
        None
    }

    /// Creates or assigns a capability namespace for a process
    pub fn create_namespace(&mut self, pid: ProcessId, badge_mask: u64) -> Result<(), i64> {
        // Reject if namespace already exists for pid
        for ns in self.namespaces.iter() {
            if ns.is_allocated && ns.process_id == pid {
                return Err(EEXIST);
            }
        }
        for ns in self.namespaces.iter_mut() {
            if !ns.is_allocated {
                *ns = CapNamespace::new(pid, badge_mask);
                return Ok(());
            }
        }
        Err(ENOMEM)
    }

    /// Retrieves an immutable reference to a process namespace (INV-CAP-004)
    pub fn get(&self, pid: ProcessId) -> Result<&CapNamespace, i64> {
        for ns in self.namespaces.iter() {
            if ns.is_allocated && ns.process_id == pid {
                return Ok(ns);
            }
        }
        Err(EBADF)
    }

    /// Retrieves a mutable reference to a process namespace (INV-CAP-004)
    pub fn get_mut(&mut self, pid: ProcessId) -> Result<&mut CapNamespace, i64> {
        for ns in self.namespaces.iter_mut() {
            if ns.is_allocated && ns.process_id == pid {
                return Ok(ns);
            }
        }
        Err(EBADF)
    }

    /// Delegates a capability across process namespaces with monotonic attenuation, cross-process
    /// lineage tracking, and object tracking (INV-CAP-001, INV-CAP-002, INV-CAP-003, INV-CAP-004)
    pub fn delegate(
        &mut self,
        src_pid: ProcessId,
        src_handle: CapHandle,
        target_pid: ProcessId,
        target_rights: CapRights,
        mut obj_table: Option<&mut KObjectTable>,
    ) -> Result<CapHandle, i64> {
        let (src_cap, src_badge) = {
            let src_ns = self.get(src_pid)?;
            let src = src_ns.lookup(src_handle)?;
            if !src.rights.contains(CapRights::GRANT) {
                return Err(EPERM);
            }
            if !src.rights.can_attenuate_to(target_rights) {
                return Err(EPERM);
            }
            (src.cap_type, src.badge)
        };

        if let CapType::Object(kobj_id) = src_cap {
            if let Some(ref mut objs) = obj_table {
                objs.retain(kobj_id)?;
            }
        }

        let mut delegated = Capability::new(src_cap, target_rights, src_badge);
        delegated.parent_pid = Some(src_pid);
        delegated.parent_handle = Some(src_handle);

        let target_ns = self.get_mut(target_pid)?;
        match target_ns.install(delegated) {
            Ok(h) => Ok(h),
            Err(e) => {
                if let CapType::Object(kobj_id) = src_cap {
                    if let Some(ref mut objs) = obj_table {
                        let _ = objs.release(kobj_id);
                    }
                }
                Err(e)
            }
        }
    }

    /// Globally revokes a capability and transitively invalidates all derived descendant capabilities
    /// across all process namespaces, releasing KObjects in KObjectTable (INV-CAP-002, INV-CAP-003).
    /// Bounded, non-recursive stack worklist bitmask traversal (Watchpoint 2).
    /// Returns total number of capability slots invalidated across all namespaces.
    pub fn revoke(
        &mut self,
        root_pid: ProcessId,
        root_handle: CapHandle,
        mut obj_table: Option<&mut KObjectTable>,
    ) -> Result<usize, i64> {
        let root_ns_idx = self.find_namespace_idx(root_pid).ok_or(EBADF)?;
        let r_idx = root_handle as usize;
        if r_idx >= MAX_CAPABILITIES || !self.namespaces[root_ns_idx].table.slots[r_idx].is_valid {
            return Err(EBADF);
        }

        // Bounded stack bitmask: 16 namespaces x 64 bits = 128 bytes total
        let mut to_revoke = [0u64; MAX_NAMESPACES];
        to_revoke[root_ns_idx] |= 1u64 << r_idx;

        let mut cascade = true;
        while cascade {
            cascade = false;
            for ns_idx in 0..MAX_NAMESPACES {
                if !self.namespaces[ns_idx].is_allocated {
                    continue;
                }
                for h in 0..MAX_CAPABILITIES {
                    let slot = &self.namespaces[ns_idx].table.slots[h];
                    if !slot.is_valid {
                        continue;
                    }
                    if (to_revoke[ns_idx] & (1u64 << h)) != 0 {
                        continue;
                    }

                    // Check local lineage within same namespace
                    if let Some(p_slot) = slot.parent_slot {
                        let p_idx = p_slot as usize;
                        if p_idx < MAX_CAPABILITIES && (to_revoke[ns_idx] & (1u64 << p_idx)) != 0 {
                            to_revoke[ns_idx] |= 1u64 << h;
                            cascade = true;
                            continue;
                        }
                    }

                    // Check cross-namespace lineage
                    if let (Some(parent_pid), Some(parent_h)) = (slot.parent_pid, slot.parent_handle) {
                        let p_h_idx = parent_h as usize;
                        if p_h_idx < MAX_CAPABILITIES {
                            if let Some(parent_ns_idx) = self.find_namespace_idx(parent_pid) {
                                if (to_revoke[parent_ns_idx] & (1u64 << p_h_idx)) != 0 {
                                    to_revoke[ns_idx] |= 1u64 << h;
                                    cascade = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut total_revoked = 0;
        for ns_idx in 0..MAX_NAMESPACES {
            if !self.namespaces[ns_idx].is_allocated {
                continue;
            }
            for h in 0..MAX_CAPABILITIES {
                if (to_revoke[ns_idx] & (1u64 << h)) != 0 {
                    let slot = &mut self.namespaces[ns_idx].table.slots[h];
                    if slot.is_valid {
                        let old_type = slot.cap_type;
                        slot.is_valid = false;
                        slot.cap_type = CapType::Empty;
                        slot.rights = CapRights::NONE;
                        slot.parent_slot = None;
                        slot.parent_pid = None;
                        slot.parent_handle = None;
                        total_revoked += 1;

                        if let CapType::Object(kobj_id) = old_type {
                            if let Some(ref mut objs) = obj_table {
                                let _ = objs.release(kobj_id);
                            }
                        }
                    }
                }
            }
        }

        Ok(total_revoked)
    }

    /// Destroys a process namespace, revoking all capabilities globally and releasing KObjects (INV-CAP-002, INV-CAP-003)
    pub fn destroy_namespace(
        &mut self,
        pid: ProcessId,
        mut obj_table: Option<&mut KObjectTable>,
    ) -> Result<usize, i64> {
        let ns_idx = self.find_namespace_idx(pid).ok_or(EBADF)?;
        let mut total_revoked = 0;
        for h in 0..MAX_CAPABILITIES {
            if self.namespaces[ns_idx].table.slots[h].is_valid {
                if let Ok(rev) = self.revoke(pid, h as CapHandle, obj_table.as_deref_mut()) {
                    total_revoked += rev;
                }
            }
        }
        self.namespaces[ns_idx] = CapNamespace::empty();
        Ok(total_revoked)
    }

    /// Returns total active capability slots across all process namespaces (INV-CAP-LEAK-001)
    pub fn total_active_capabilities(&self) -> usize {
        let mut count = 0;
        for ns in self.namespaces.iter() {
            if ns.is_allocated {
                count += ns.active_count();
            }
        }
        count
    }

    /// Returns the number of currently allocated namespaces
    pub fn allocated_count(&self) -> usize {
        self.namespaces.iter().filter(|ns| ns.is_allocated).count()
    }
}
