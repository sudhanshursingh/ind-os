//! Centralized, Pre-allocated Kernel Object Table
//! Enforces Anti-ABA generation checking (INV-OBJ-001), reference counting (INV-OBJ-002),
//! strong type isolation (INV-OBJ-003), and bounded allocation (INV-OBJ-004).

use crate::object::id::KObjectId;
use crate::object::types::{
    EndpointObject, IrqObject, KObject, MemoryRegionObject, ProcessObject,
};
use crate::syscall::{EBADF, EINVAL, ENOMEM};
use crate::task::scheduler::Priority;
use crate::task::ProcessId;

/// Maximum number of active kernel objects across the system
pub const MAX_KOBJECTS: usize = 128;

/// Entry slot in the Kernel Object Table
#[derive(Clone, Copy, Debug)]
pub struct KObjectSlot {
    pub generation: u16,
    pub ref_count: u32,
    pub object: Option<KObject>,
}

impl KObjectSlot {
    pub const fn empty() -> Self {
        Self {
            generation: 1,
            ref_count: 0,
            object: None,
        }
    }
}

/// Centralized Kernel Object Table
pub struct KObjectTable {
    pub slots: [KObjectSlot; MAX_KOBJECTS],
    pub active_count: usize,
}

impl KObjectTable {
    /// Creates a new, pre-allocated kernel object table
    pub const fn new() -> Self {
        Self {
            slots: [KObjectSlot::empty(); MAX_KOBJECTS],
            active_count: 0,
        }
    }

    /// Internal slot allocator
    fn allocate_slot(&mut self, kobj: KObject) -> Result<KObjectId, i64> {
        for i in 0..MAX_KOBJECTS {
            if self.slots[i].object.is_none() {
                let slot = &mut self.slots[i];
                let gen = slot.generation;
                slot.object = Some(kobj);
                slot.ref_count = 1;
                self.active_count += 1;
                return Ok(KObjectId::new(i as u16, gen, kobj.obj_type()));
            }
        }
        Err(ENOMEM)
    }

    /// Creates and installs a new Process object
    pub fn create_process(
        &mut self,
        pid: ProcessId,
        name: &'static str,
        priority: Priority,
        ttbr0: usize,
    ) -> Result<KObjectId, i64> {
        let obj = ProcessObject::new(pid, name, priority, ttbr0);
        self.allocate_slot(KObject::Process(obj))
    }

    /// Creates and installs a new IPC Endpoint object
    pub fn create_endpoint(&mut self, ep_id: u32) -> Result<KObjectId, i64> {
        let obj = EndpointObject::new(ep_id);
        self.allocate_slot(KObject::Endpoint(obj))
    }

    /// Creates and installs a new Memory Region object
    pub fn create_memory_region(
        &mut self,
        base: usize,
        size: usize,
        is_device: bool,
        is_writable: bool,
        is_executable: bool,
    ) -> Result<KObjectId, i64> {
        let obj = MemoryRegionObject::new(base, size, is_device, is_writable, is_executable);
        self.allocate_slot(KObject::MemoryRegion(obj))
    }

    /// Creates and installs a new Hardware Interrupt (IRQ) object
    pub fn create_irq(&mut self, irq_num: u32, is_edge: bool) -> Result<KObjectId, i64> {
        let obj = IrqObject::new(irq_num, is_edge);
        self.allocate_slot(KObject::Irq(obj))
    }

    /// Increments reference count of an object (INV-OBJ-002)
    pub fn retain(&mut self, id: KObjectId) -> Result<(), i64> {
        let idx = id.index as usize;
        if idx >= MAX_KOBJECTS {
            return Err(EBADF);
        }

        let slot = &mut self.slots[idx];
        if slot.object.is_none() || slot.generation != id.generation {
            return Err(EBADF);
        }

        let obj = slot.object.as_ref().unwrap();
        if obj.obj_type() != id.obj_type {
            return Err(EINVAL);
        }

        if slot.ref_count == 0 {
            return Err(EBADF);
        }

        slot.ref_count += 1;
        Ok(())
    }

    /// Decrements reference count of an object.
    /// If ref_count reaches 0, the object is destroyed and slot generation incremented (INV-OBJ-001, INV-OBJ-002).
    /// Returns Ok(true) if destroyed, Ok(false) if still referenced.
    pub fn release(&mut self, id: KObjectId) -> Result<bool, i64> {
        let idx = id.index as usize;
        if idx >= MAX_KOBJECTS {
            return Err(EBADF);
        }

        let slot = &mut self.slots[idx];
        if slot.object.is_none() || slot.generation != id.generation {
            return Err(EBADF);
        }

        let obj = slot.object.as_ref().unwrap();
        if obj.obj_type() != id.obj_type {
            return Err(EINVAL);
        }

        if slot.ref_count == 0 {
            return Err(EBADF);
        }

        slot.ref_count -= 1;
        if slot.ref_count == 0 {
            slot.object = None;
            slot.generation = slot.generation.wrapping_add(1);
            if slot.generation == 0 {
                slot.generation = 1;
            }
            self.active_count -= 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Resolves an immutable reference to an object, verifying generation and type (INV-OBJ-001, INV-OBJ-003)
    pub fn get(&self, id: KObjectId) -> Result<&KObject, i64> {
        let idx = id.index as usize;
        if idx >= MAX_KOBJECTS {
            return Err(EBADF);
        }

        let slot = &self.slots[idx];
        if slot.generation != id.generation {
            return Err(EBADF);
        }

        match &slot.object {
            Some(obj) => {
                if obj.obj_type() != id.obj_type {
                    Err(EINVAL)
                } else {
                    Ok(obj)
                }
            }
            None => Err(EBADF),
        }
    }

    /// Resolves a mutable reference to an object, verifying generation and type (INV-OBJ-001, INV-OBJ-003)
    pub fn get_mut(&mut self, id: KObjectId) -> Result<&mut KObject, i64> {
        let idx = id.index as usize;
        if idx >= MAX_KOBJECTS {
            return Err(EBADF);
        }

        let slot = &mut self.slots[idx];
        if slot.generation != id.generation {
            return Err(EBADF);
        }

        match &mut slot.object {
            Some(obj) => {
                if obj.obj_type() != id.obj_type {
                    Err(EINVAL)
                } else {
                    Ok(obj)
                }
            }
            None => Err(EBADF),
        }
    }

    /// Resolves a typed reference to a ProcessObject
    pub fn get_process(&self, id: KObjectId) -> Result<&ProcessObject, i64> {
        match self.get(id)? {
            KObject::Process(ref proc) => Ok(proc),
            _ => Err(EINVAL),
        }
    }

    /// Resolves a typed mutable reference to a ProcessObject
    pub fn get_process_mut(&mut self, id: KObjectId) -> Result<&mut ProcessObject, i64> {
        match self.get_mut(id)? {
            KObject::Process(ref mut proc) => Ok(proc),
            _ => Err(EINVAL),
        }
    }

    /// Resolves a typed reference to an EndpointObject
    pub fn get_endpoint(&self, id: KObjectId) -> Result<&EndpointObject, i64> {
        match self.get(id)? {
            KObject::Endpoint(ref ep) => Ok(ep),
            _ => Err(EINVAL),
        }
    }

    /// Resolves a typed mutable reference to an EndpointObject
    pub fn get_endpoint_mut(&mut self, id: KObjectId) -> Result<&mut EndpointObject, i64> {
        match self.get_mut(id)? {
            KObject::Endpoint(ref mut ep) => Ok(ep),
            _ => Err(EINVAL),
        }
    }

    /// Resolves a typed reference to a MemoryRegionObject
    pub fn get_memory_region(&self, id: KObjectId) -> Result<&MemoryRegionObject, i64> {
        match self.get(id)? {
            KObject::MemoryRegion(ref mr) => Ok(mr),
            _ => Err(EINVAL),
        }
    }

    /// Resolves a typed mutable reference to a MemoryRegionObject
    pub fn get_memory_region_mut(&mut self, id: KObjectId) -> Result<&mut MemoryRegionObject, i64> {
        match self.get_mut(id)? {
            KObject::MemoryRegion(ref mut mr) => Ok(mr),
            _ => Err(EINVAL),
        }
    }

    /// Resolves a typed reference to an IrqObject
    pub fn get_irq(&self, id: KObjectId) -> Result<&IrqObject, i64> {
        match self.get(id)? {
            KObject::Irq(ref irq) => Ok(irq),
            _ => Err(EINVAL),
        }
    }

    /// Resolves a typed mutable reference to an IrqObject
    pub fn get_irq_mut(&mut self, id: KObjectId) -> Result<&mut IrqObject, i64> {
        match self.get_mut(id)? {
            KObject::Irq(ref mut irq) => Ok(irq),
            _ => Err(EINVAL),
        }
    }

    /// Returns the reference count of an object (INV-OBJ-002)
    pub fn get_ref_count(&self, id: KObjectId) -> Result<u32, i64> {
        let idx = id.index as usize;
        if idx >= MAX_KOBJECTS {
            return Err(EBADF);
        }

        let slot = &self.slots[idx];
        if slot.generation != id.generation || slot.object.is_none() {
            return Err(EBADF);
        }

        let obj = slot.object.as_ref().unwrap();
        if obj.obj_type() != id.obj_type {
            return Err(EINVAL);
        }

        Ok(slot.ref_count)
    }

    /// Returns the number of currently active KObjects in the table
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.object.is_some()).count()
    }
}
