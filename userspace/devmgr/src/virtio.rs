//! Ind OS VirtIO Hotplug Lifecycle Engine & Test Harness
//!
//! Standards:
//! - REQ-USE-203: virtio plug/unplug tests
//! - INV-DEV-001: Zero ambient authority
//! - INV-DEV-002: Dynamic hotplug notification and capability revocation

use crate::bus::{HotplugEvent, HotplugEventBus};
use crate::handout::{CapabilityHandoutRegistry, DeviceCapToken};
use crate::types::{BusType, DeviceError, DeviceId, DeviceState, DeviceType, HardwareDescriptor};

pub const MAX_VIRTIO_SLOTS: usize = 8;

pub struct VirtioSlot {
    pub slot_id: u32,
    pub device_id: Option<DeviceId>,
    pub desc: Option<HardwareDescriptor>,
    pub state: DeviceState,
    pub token: Option<DeviceCapToken>,
}

pub struct VirtioDeviceManager {
    slots: [VirtioSlot; MAX_VIRTIO_SLOTS],
    event_bus: HotplugEventBus,
    registry: CapabilityHandoutRegistry,
    instance_counter: u32,
}

impl VirtioDeviceManager {
    pub fn new() -> Self {
        Self {
            slots: [
                VirtioSlot { slot_id: 0, device_id: None, desc: None, state: DeviceState::Unattached, token: None },
                VirtioSlot { slot_id: 1, device_id: None, desc: None, state: DeviceState::Unattached, token: None },
                VirtioSlot { slot_id: 2, device_id: None, desc: None, state: DeviceState::Unattached, token: None },
                VirtioSlot { slot_id: 3, device_id: None, desc: None, state: DeviceState::Unattached, token: None },
                VirtioSlot { slot_id: 4, device_id: None, desc: None, state: DeviceState::Unattached, token: None },
                VirtioSlot { slot_id: 5, device_id: None, desc: None, state: DeviceState::Unattached, token: None },
                VirtioSlot { slot_id: 6, device_id: None, desc: None, state: DeviceState::Unattached, token: None },
                VirtioSlot { slot_id: 7, device_id: None, desc: None, state: DeviceState::Unattached, token: None },
            ],
            event_bus: HotplugEventBus::new(),
            registry: CapabilityHandoutRegistry::new(),
            instance_counter: 1,
        }
    }

    pub fn event_bus_mut(&mut self) -> &mut HotplugEventBus {
        &mut self.event_bus
    }

    pub fn registry_mut(&mut self) -> &mut CapabilityHandoutRegistry {
        &mut self.registry
    }

    pub fn plug_device(
        &mut self,
        slot_id: u32,
        device_type: DeviceType,
        desc: HardwareDescriptor,
    ) -> Result<DeviceCapToken, DeviceError> {
        let slot_idx = slot_id as usize;
        if slot_idx >= MAX_VIRTIO_SLOTS {
            return Err(DeviceError::DeviceNotFound);
        }
        if self.slots[slot_idx].state == DeviceState::Attached || self.slots[slot_idx].state == DeviceState::Bound {
            return Err(DeviceError::AlreadyAttached);
        }

        let dev_id = DeviceId::new(BusType::VirtIO, slot_id, device_type, self.instance_counter);
        self.instance_counter += 1;

        // 1. Mint capability token per REQ-USE-203
        let token = self.registry.mint_token(dev_id, &desc)?;

        // 2. Update slot state
        self.slots[slot_idx].device_id = Some(dev_id);
        self.slots[slot_idx].desc = Some(desc);
        self.slots[slot_idx].state = DeviceState::Bound;
        self.slots[slot_idx].token = Some(token);

        // 3. Publish arrival event
        self.event_bus.publish(HotplugEvent::DeviceArrived(dev_id, desc))?;

        Ok(token)
    }

    pub fn unplug_device(&mut self, slot_id: u32) -> Result<DeviceId, DeviceError> {
        let slot_idx = slot_id as usize;
        if slot_idx >= MAX_VIRTIO_SLOTS {
            return Err(DeviceError::DeviceNotFound);
        }

        let dev_id = match self.slots[slot_idx].device_id {
            Some(id) => id,
            None => return Err(DeviceError::DeviceNotFound),
        };

        // 1. Revoke capability tokens per INV-DEV-002
        self.registry.revoke_for_device(dev_id);
        if let Some(ref mut tok) = self.slots[slot_idx].token {
            tok.revoke();
        }

        // 2. Mark state as Departed
        self.slots[slot_idx].state = DeviceState::Departed;
        self.slots[slot_idx].device_id = None;
        self.slots[slot_idx].desc = None;

        // 3. Publish departed event
        self.event_bus.publish(HotplugEvent::DeviceDeparted(dev_id))?;

        Ok(dev_id)
    }

    pub fn active_devices_count(&self) -> usize {
        let mut count = 0;
        for s in self.slots.iter() {
            if s.state == DeviceState::Bound {
                count += 1;
            }
        }
        count
    }
}
