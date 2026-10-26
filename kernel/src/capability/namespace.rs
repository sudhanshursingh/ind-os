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
        if let Some(ref mut objs) = obj_table {
            self.table.mint_object_cap(src_handle, target_rights, badge, objs)
        } else {
            self.table.mint(src_handle, target_rights, badge)
        }
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
        if let Some(ref mut objs) = obj_table {
            self.table.delegate_object_to(src_handle, target_rights, &mut target_ns.table, objs)
        } else {
            self.table.delegate_to(src_handle, target_rights, &mut target_ns.table)
        }
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

    /// Destroys a process namespace, revoking all capabilities and releasing KObjects (INV-CAP-002, INV-CAP-003)
    pub fn destroy_namespace(
        &mut self,
        pid: ProcessId,
        mut obj_table: Option<&mut KObjectTable>,
    ) -> Result<usize, i64> {
        for ns in self.namespaces.iter_mut() {
            if ns.is_allocated && ns.process_id == pid {
                let mut total_revoked = 0;
                for h in 0..MAX_CAPABILITIES {
                    if let Ok(_) = ns.lookup(h as CapHandle) {
                        if let Ok(rev) = ns.revoke(h as CapHandle, obj_table.as_deref_mut()) {
                            total_revoked += rev;
                        }
                    }
                }
                *ns = CapNamespace::empty();
                return Ok(total_revoked);
            }
        }
        Err(EBADF)
    }

    /// Delegates a capability across process namespaces with monotonic attenuation and object tracking (INV-CAP-001, INV-CAP-003, INV-CAP-004)
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

        let delegated = Capability::new(src_cap, target_rights, src_badge);
        let target_ns = self.get_mut(target_pid)?;
        target_ns.install(delegated)
    }

    /// Returns the number of currently allocated namespaces
    pub fn allocated_count(&self) -> usize {
        self.namespaces.iter().filter(|ns| ns.is_allocated).count()
    }
}
