//! Kernel Device Manager & Hotplug Verification Harness (SP-34)
//!
//! Architectural Invariants:
//! - REQ-USE-203: Device manager + hotplug events: capability handout
//! - INV-DEV-001: Zero ambient authority (scoped MMIO, IRQ, DMA capabilities)
//! - INV-DEV-002: Dynamic hotplug notification and atomic capability revocation

pub const KERNEL_MAX_DEVICES: usize = 8;
pub const KERNEL_MAX_SUBSCRIBERS: usize = 8;
pub const KERNEL_MAX_HOTPLUG_EVENTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelBusType {
    Platform,
    VirtIO,
    PciExpress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelDeviceType {
    Block,
    Network,
    Console,
    Input,
    Gpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelDeviceId {
    pub bus_type: KernelBusType,
    pub slot: u32,
    pub dev_type: KernelDeviceType,
    pub instance_id: u32,
}

impl KernelDeviceId {
    pub const fn new(bus_type: KernelBusType, slot: u32, dev_type: KernelDeviceType, instance_id: u32) -> Self {
        Self {
            bus_type,
            slot,
            dev_type,
            instance_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelHardwareDescriptor {
    pub mmio_base: usize,
    pub mmio_size: usize,
    pub irq_line: u32,
    pub dma_size: usize,
}

impl KernelHardwareDescriptor {
    pub const fn new(mmio_base: usize, mmio_size: usize, irq_line: u32, dma_size: usize) -> Self {
        Self {
            mmio_base,
            mmio_size,
            irq_line,
            dma_size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelDevError {
    DeviceNotFound,
    AlreadyAttached,
    AccessDenied,
    DeviceDeparted,
    BufferFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelMmioCapToken {
    pub base: usize,
    pub size: usize,
    pub is_valid: bool,
}

impl KernelMmioCapToken {
    pub const fn new(base: usize, size: usize) -> Self {
        Self { base, size, is_valid: true }
    }

    pub fn check_access(&self, offset: usize, len: usize) -> Result<(), KernelDevError> {
        if !self.is_valid || (offset + len > self.size) {
            return Err(KernelDevError::AccessDenied);
        }
        Ok(())
    }

    pub fn revoke(&mut self) {
        self.is_valid = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelIrqCapToken {
    pub irq_line: u32,
    pub is_valid: bool,
}

impl KernelIrqCapToken {
    pub const fn new(irq_line: u32) -> Self {
        Self { irq_line, is_valid: true }
    }

    pub fn check_handle(&self, line: u32) -> Result<(), KernelDevError> {
        if !self.is_valid || self.irq_line != line {
            return Err(KernelDevError::AccessDenied);
        }
        Ok(())
    }

    pub fn revoke(&mut self) {
        self.is_valid = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelDmaCapToken {
    pub dma_size: usize,
    pub is_valid: bool,
}

impl KernelDmaCapToken {
    pub const fn new(dma_size: usize) -> Self {
        Self { dma_size, is_valid: true }
    }

    pub fn check_dma(&self, len: usize) -> Result<(), KernelDevError> {
        if !self.is_valid || len > self.dma_size {
            return Err(KernelDevError::AccessDenied);
        }
        Ok(())
    }

    pub fn revoke(&mut self) {
        self.is_valid = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelDeviceCapToken {
    pub token_id: u32,
    pub device_id: KernelDeviceId,
    pub mmio: KernelMmioCapToken,
    pub irq: KernelIrqCapToken,
    pub dma: KernelDmaCapToken,
    pub is_valid: bool,
}

impl KernelDeviceCapToken {
    pub const fn new(
        token_id: u32,
        device_id: KernelDeviceId,
        mmio: KernelMmioCapToken,
        irq: KernelIrqCapToken,
        dma: KernelDmaCapToken,
    ) -> Self {
        Self {
            token_id,
            device_id,
            mmio,
            irq,
            dma,
            is_valid: true,
        }
    }

    pub fn revoke(&mut self) {
        self.is_valid = false;
        self.mmio.revoke();
        self.irq.revoke();
        self.dma.revoke();
    }

    pub fn is_active(&self) -> bool {
        self.is_valid && self.mmio.is_valid && self.irq.is_valid
    }

    pub fn verify_mmio(&self, offset: usize, len: usize) -> Result<(), KernelDevError> {
        if !self.is_valid {
            return Err(KernelDevError::DeviceDeparted);
        }
        self.mmio.check_access(offset, len)
    }

    pub fn verify_irq(&self, line: u32) -> Result<(), KernelDevError> {
        if !self.is_valid {
            return Err(KernelDevError::DeviceDeparted);
        }
        self.irq.check_handle(line)
    }

    pub fn verify_dma(&self, len: usize) -> Result<(), KernelDevError> {
        if !self.is_valid {
            return Err(KernelDevError::DeviceDeparted);
        }
        self.dma.check_dma(len)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelHotplugEvent {
    DeviceArrived(KernelDeviceId, KernelHardwareDescriptor),
    DeviceDeparted(KernelDeviceId),
    DeviceReset(KernelDeviceId),
}

pub struct KernelHotplugBus {
    subscribers: [Option<u32>; KERNEL_MAX_SUBSCRIBERS],
    events: [Option<KernelHotplugEvent>; KERNEL_MAX_HOTPLUG_EVENTS],
    head: usize,
    tail: usize,
    count: usize,
}

impl KernelHotplugBus {
    pub const fn new() -> Self {
        Self {
            subscribers: [None; KERNEL_MAX_SUBSCRIBERS],
            events: [None; KERNEL_MAX_HOTPLUG_EVENTS],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn register(&mut self, sub_id: u32) -> Result<(), KernelDevError> {
        for slot in self.subscribers.iter_mut() {
            if slot.is_none() {
                *slot = Some(sub_id);
                return Ok(());
            }
        }
        Err(KernelDevError::BufferFull)
    }

    pub fn publish(&mut self, event: KernelHotplugEvent) -> Result<(), KernelDevError> {
        if self.count >= KERNEL_MAX_HOTPLUG_EVENTS {
            return Err(KernelDevError::BufferFull);
        }
        self.events[self.tail] = Some(event);
        self.tail = (self.tail + 1) % KERNEL_MAX_HOTPLUG_EVENTS;
        self.count += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<KernelHotplugEvent> {
        if self.count == 0 {
            return None;
        }
        let ev = self.events[self.head].take();
        self.head = (self.head + 1) % KERNEL_MAX_HOTPLUG_EVENTS;
        self.count -= 1;
        ev
    }

    pub fn subscriber_count(&self) -> usize {
        let mut c = 0;
        for s in self.subscribers.iter() {
            if s.is_some() {
                c += 1;
            }
        }
        c
    }

    pub fn event_count(&self) -> usize {
        self.count
    }
}

#[derive(Clone, Copy)]
pub struct KernelDeviceEntry {
    pub device_id: KernelDeviceId,
    pub desc: KernelHardwareDescriptor,
    pub token: KernelDeviceCapToken,
    pub active: bool,
}

pub struct KernelDeviceManager {
    entries: [Option<KernelDeviceEntry>; KERNEL_MAX_DEVICES],
    event_bus: KernelHotplugBus,
    next_token_id: u32,
    next_instance_id: u32,
}

impl KernelDeviceManager {
    pub const fn new() -> Self {
        Self {
            entries: [None; KERNEL_MAX_DEVICES],
            event_bus: KernelHotplugBus::new(),
            next_token_id: 1,
            next_instance_id: 1,
        }
    }

    pub fn event_bus_mut(&mut self) -> &mut KernelHotplugBus {
        &mut self.event_bus
    }

    pub fn plug_virtio_device(
        &mut self,
        slot: u32,
        dev_type: KernelDeviceType,
        desc: KernelHardwareDescriptor,
    ) -> Result<KernelDeviceCapToken, KernelDevError> {
        // Check if slot already occupied
        for entry in self.entries.iter() {
            if let Some(ref e) = entry {
                if e.active && e.device_id.slot == slot && e.device_id.bus_type == KernelBusType::VirtIO {
                    return Err(KernelDevError::AlreadyAttached);
                }
            }
        }

        let dev_id = KernelDeviceId::new(KernelBusType::VirtIO, slot, dev_type, self.next_instance_id);
        self.next_instance_id += 1;

        let token_id = self.next_token_id;
        self.next_token_id += 1;

        let mmio = KernelMmioCapToken::new(desc.mmio_base, desc.mmio_size);
        let irq = KernelIrqCapToken::new(desc.irq_line);
        let dma = KernelDmaCapToken::new(desc.dma_size);
        let token = KernelDeviceCapToken::new(token_id, dev_id, mmio, irq, dma);

        let entry = KernelDeviceEntry {
            device_id: dev_id,
            desc,
            token,
            active: true,
        };

        for slot_opt in self.entries.iter_mut() {
            if slot_opt.is_none() {
                *slot_opt = Some(entry);
                self.event_bus.publish(KernelHotplugEvent::DeviceArrived(dev_id, desc))?;
                return Ok(token);
            }
        }
        Err(KernelDevError::BufferFull)
    }

    pub fn unplug_virtio_device(&mut self, slot: u32) -> Result<KernelDeviceId, KernelDevError> {
        for slot_opt in self.entries.iter_mut() {
            if let Some(ref mut e) = slot_opt {
                if e.active && e.device_id.slot == slot && e.device_id.bus_type == KernelBusType::VirtIO {
                    e.active = false;
                    e.token.revoke();
                    let dev_id = e.device_id;
                    self.event_bus.publish(KernelHotplugEvent::DeviceDeparted(dev_id))?;
                    return Ok(dev_id);
                }
            }
        }
        Err(KernelDevError::DeviceNotFound)
    }

    pub fn active_devices_count(&self) -> usize {
        let mut count = 0;
        for slot_opt in self.entries.iter() {
            if let Some(ref e) = slot_opt {
                if e.active {
                    count += 1;
                }
            }
        }
        count
    }
}
