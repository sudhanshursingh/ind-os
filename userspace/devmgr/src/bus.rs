//! Ind OS Hotplug Event Bus
//!
//! Standards:
//! - REQ-USE-203: Device manager + hotplug events
//! - INV-DEV-002: Dynamic hotplug notification and event dispatch

use crate::types::{DeviceError, DeviceId, DeviceType, HardwareDescriptor};

pub const MAX_SUBSCRIBERS: usize = 16;
pub const MAX_PENDING_EVENTS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotplugEvent {
    DeviceArrived(DeviceId, HardwareDescriptor),
    DeviceDeparted(DeviceId),
    DeviceReset(DeviceId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscriber {
    pub subscriber_id: u32,
    pub target_type: Option<DeviceType>, // None = all device types
}

pub struct HotplugEventBus {
    subscribers: [Option<Subscriber>; MAX_SUBSCRIBERS],
    events: [Option<HotplugEvent>; MAX_PENDING_EVENTS],
    event_head: usize,
    event_tail: usize,
    event_count: usize,
}

impl HotplugEventBus {
    pub const fn new() -> Self {
        Self {
            subscribers: [None; MAX_SUBSCRIBERS],
            events: [None; MAX_PENDING_EVENTS],
            event_head: 0,
            event_tail: 0,
            event_count: 0,
        }
    }

    pub fn subscribe(&mut self, subscriber_id: u32, target_type: Option<DeviceType>) -> Result<(), DeviceError> {
        for slot in self.subscribers.iter_mut() {
            if slot.is_none() {
                *slot = Some(Subscriber {
                    subscriber_id,
                    target_type,
                });
                return Ok(());
            }
        }
        Err(DeviceError::BufferFull)
    }

    pub fn publish(&mut self, event: HotplugEvent) -> Result<(), DeviceError> {
        if self.event_count >= MAX_PENDING_EVENTS {
            return Err(DeviceError::BufferFull);
        }
        self.events[self.event_tail] = Some(event);
        self.event_tail = (self.event_tail + 1) % MAX_PENDING_EVENTS;
        self.event_count += 1;
        Ok(())
    }

    pub fn pop_event(&mut self) -> Option<HotplugEvent> {
        if self.event_count == 0 {
            return None;
        }
        let ev = self.events[self.event_head].take();
        self.event_head = (self.event_head + 1) % MAX_PENDING_EVENTS;
        self.event_count -= 1;
        ev
    }

    pub fn pending_count(&self) -> usize {
        self.event_count
    }

    pub fn subscriber_count(&self) -> usize {
        let mut count = 0;
        for s in self.subscribers.iter() {
            if s.is_some() {
                count += 1;
            }
        }
        count
    }
}
