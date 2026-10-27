//! Kernel Unified Sensor Event Hub Core (SP-92, REQ-HW-110, REQ-SNS-104)
//!
//! Target: Unified Sensor Event Hub & Multiplexer Service
//! Architectural Invariants:
//! - INV-HUB-001: Subscription capability barrier (KERNEL_CAP_SENSOR_READ required)
//! - INV-HUB-002: Microsecond timestamp synchronization across all sensor streams
//! - INV-HUB-003: Multi-client FIFO event dispatch and isolation

pub const KERNEL_MAX_SENSOR_SUBSCRIBERS: usize = 8;
pub const KERNEL_SENSOR_EVENT_RING_CAPACITY: usize = 32;
pub const KERNEL_CAP_SENSOR_READ: u32 = 0x0800_0000;

pub const KERNEL_CHANNEL_MASK_ACCEL: u32 = 1 << 0;
pub const KERNEL_CHANNEL_MASK_GYRO: u32  = 1 << 1;
pub const KERNEL_CHANNEL_MASK_ALS: u32   = 1 << 2;
pub const KERNEL_CHANNEL_MASK_PROX: u32  = 1 << 3;
pub const KERNEL_CHANNEL_MASK_MAG: u32   = 1 << 4;
pub const KERNEL_CHANNEL_MASK_ALL: u32   = 0x1F;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSensorChannel {
    ImuAccel,
    ImuGyro,
    AlsLux,
    Proximity,
    Magnetometer,
}

impl KernelSensorChannel {
    pub const fn to_mask(self) -> u32 {
        match self {
            Self::ImuAccel => KERNEL_CHANNEL_MASK_ACCEL,
            Self::ImuGyro => KERNEL_CHANNEL_MASK_GYRO,
            Self::AlsLux => KERNEL_CHANNEL_MASK_ALS,
            Self::Proximity => KERNEL_CHANNEL_MASK_PROX,
            Self::Magnetometer => KERNEL_CHANNEL_MASK_MAG,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSensorEvent {
    pub channel: KernelSensorChannel,
    pub timestamp_us: u64,
    pub data: [i32; 3],
}

impl KernelSensorEvent {
    pub const fn zero() -> Self {
        Self {
            channel: KernelSensorChannel::ImuAccel,
            timestamp_us: 0,
            data: [0; 3],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSensorHubError {
    PermissionDenied,
    SubscriberLimitExceeded,
    InvalidChannel,
    BufferFull,
    BufferEmpty,
    SubscriberNotFound,
}

#[derive(Clone, Copy)]
pub struct KernelSensorSubscriber {
    pub id: u32,
    pub channel_mask: u32,
    pub queue: [KernelSensorEvent; KERNEL_SENSOR_EVENT_RING_CAPACITY],
    pub head: usize,
    pub tail: usize,
    pub count: usize,
}

impl KernelSensorSubscriber {
    pub const fn new(id: u32, channel_mask: u32) -> Self {
        Self {
            id,
            channel_mask,
            queue: [KernelSensorEvent::zero(); KERNEL_SENSOR_EVENT_RING_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, event: KernelSensorEvent) -> Result<(), KernelSensorHubError> {
        if self.count >= KERNEL_SENSOR_EVENT_RING_CAPACITY {
            return Err(KernelSensorHubError::BufferFull);
        }
        self.queue[self.tail] = event;
        self.tail = (self.tail + 1) % KERNEL_SENSOR_EVENT_RING_CAPACITY;
        self.count += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<KernelSensorEvent, KernelSensorHubError> {
        if self.count == 0 {
            return Err(KernelSensorHubError::BufferEmpty);
        }
        let ev = self.queue[self.head];
        self.head = (self.head + 1) % KERNEL_SENSOR_EVENT_RING_CAPACITY;
        self.count -= 1;
        Ok(ev)
    }
}

pub struct KernelSensorHubHarness {
    subscribers: [Option<KernelSensorSubscriber>; KERNEL_MAX_SENSOR_SUBSCRIBERS],
    next_sub_id: u32,
    last_timestamp_us: u64,
    events_dispatched: usize,
}

impl KernelSensorHubHarness {
    pub const fn new() -> Self {
        Self {
            subscribers: [None; KERNEL_MAX_SENSOR_SUBSCRIBERS],
            next_sub_id: 1,
            last_timestamp_us: 0,
            events_dispatched: 0,
        }
    }

    pub fn register_subscriber(&mut self, cap: u32, channel_mask: u32) -> Result<u32, KernelSensorHubError> {
        if (cap & KERNEL_CAP_SENSOR_READ) == 0 {
            return Err(KernelSensorHubError::PermissionDenied);
        }
        for slot in self.subscribers.iter_mut() {
            if slot.is_none() {
                let id = self.next_sub_id;
                self.next_sub_id += 1;
                *slot = Some(KernelSensorSubscriber::new(id, channel_mask));
                return Ok(id);
            }
        }
        Err(KernelSensorHubError::SubscriberLimitExceeded)
    }

    pub fn sync_timestamp(&mut self, timestamp_us: u64) {
        self.last_timestamp_us = timestamp_us;
    }

    pub fn post_event(&mut self, channel: KernelSensorChannel, timestamp_us: u64, data: [i32; 3]) -> Result<usize, KernelSensorHubError> {
        let mask = channel.to_mask();
        let event = KernelSensorEvent {
            channel,
            timestamp_us,
            data,
        };
        self.last_timestamp_us = timestamp_us;
        let mut delivered = 0;

        for slot in self.subscribers.iter_mut() {
            if let Some(sub) = slot {
                if (sub.channel_mask & mask) != 0 {
                    let _ = sub.push(event);
                    delivered += 1;
                }
            }
        }
        self.events_dispatched += delivered;
        Ok(delivered)
    }

    pub fn pop_event(&mut self, sub_id: u32) -> Result<KernelSensorEvent, KernelSensorHubError> {
        for slot in self.subscribers.iter_mut() {
            if let Some(sub) = slot {
                if sub.id == sub_id {
                    return sub.pop();
                }
            }
        }
        Err(KernelSensorHubError::SubscriberNotFound)
    }

    pub fn subscribers_count(&self) -> usize {
        let mut count = 0;
        for s in self.subscribers.iter() {
            if s.is_some() {
                count += 1;
            }
        }
        count
    }

    pub fn events_dispatched(&self) -> usize {
        self.events_dispatched
    }

    pub fn last_timestamp_us(&self) -> u64 {
        self.last_timestamp_us
    }
}
