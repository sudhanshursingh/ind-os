//! Userspace DevMgr Unified Sensor Event Hub (SP-92, REQ-HW-110, REQ-SNS-104)
//!
//! Target: Unified Sensor Event Hub & Multiplexer Service
//! Architectural Invariants:
//! - INV-HUB-001: Subscription capability barrier (CAP_SENSOR_READ required)
//! - INV-HUB-002: Microsecond timestamp synchronization across all sensor streams
//! - INV-HUB-003: Multi-client FIFO event dispatch and isolation

pub const MAX_SENSOR_SUBSCRIBERS: usize = 8;
pub const SENSOR_EVENT_RING_CAPACITY: usize = 32;
pub const CAP_SENSOR_READ: u32 = 0x0800_0000;

pub const CHANNEL_MASK_ACCEL: u32 = 1 << 0;
pub const CHANNEL_MASK_GYRO: u32  = 1 << 1;
pub const CHANNEL_MASK_ALS: u32   = 1 << 2;
pub const CHANNEL_MASK_PROX: u32  = 1 << 3;
pub const CHANNEL_MASK_MAG: u32   = 1 << 4;
pub const CHANNEL_MASK_ALL: u32   = 0x1F;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorChannel {
    ImuAccel,
    ImuGyro,
    AlsLux,
    Proximity,
    Magnetometer,
}

impl SensorChannel {
    pub const fn to_mask(self) -> u32 {
        match self {
            Self::ImuAccel => CHANNEL_MASK_ACCEL,
            Self::ImuGyro => CHANNEL_MASK_GYRO,
            Self::AlsLux => CHANNEL_MASK_ALS,
            Self::Proximity => CHANNEL_MASK_PROX,
            Self::Magnetometer => CHANNEL_MASK_MAG,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorEvent {
    pub channel: SensorChannel,
    pub timestamp_us: u64,
    pub data: [i32; 3],
}

impl SensorEvent {
    pub const fn zero() -> Self {
        Self {
            channel: SensorChannel::ImuAccel,
            timestamp_us: 0,
            data: [0; 3],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorHubError {
    PermissionDenied,
    SubscriberLimitExceeded,
    InvalidChannel,
    BufferFull,
    BufferEmpty,
    SubscriberNotFound,
}

#[derive(Clone, Copy)]
pub struct SensorSubscriber {
    pub id: u32,
    pub channel_mask: u32,
    pub queue: [SensorEvent; SENSOR_EVENT_RING_CAPACITY],
    pub head: usize,
    pub tail: usize,
    pub count: usize,
}

impl SensorSubscriber {
    pub const fn new(id: u32, channel_mask: u32) -> Self {
        Self {
            id,
            channel_mask,
            queue: [SensorEvent::zero(); SENSOR_EVENT_RING_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, event: SensorEvent) -> Result<(), SensorHubError> {
        if self.count >= SENSOR_EVENT_RING_CAPACITY {
            return Err(SensorHubError::BufferFull);
        }
        self.queue[self.tail] = event;
        self.tail = (self.tail + 1) % SENSOR_EVENT_RING_CAPACITY;
        self.count += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<SensorEvent, SensorHubError> {
        if self.count == 0 {
            return Err(SensorHubError::BufferEmpty);
        }
        let ev = self.queue[self.head];
        self.head = (self.head + 1) % SENSOR_EVENT_RING_CAPACITY;
        self.count -= 1;
        Ok(ev)
    }
}

pub struct SensorHubHarness {
    subscribers: [Option<SensorSubscriber>; MAX_SENSOR_SUBSCRIBERS],
    next_sub_id: u32,
    last_timestamp_us: u64,
    events_dispatched: usize,
}

impl SensorHubHarness {
    pub const fn new() -> Self {
        Self {
            subscribers: [None; MAX_SENSOR_SUBSCRIBERS],
            next_sub_id: 1,
            last_timestamp_us: 0,
            events_dispatched: 0,
        }
    }

    pub fn register_subscriber(&mut self, cap: u32, channel_mask: u32) -> Result<u32, SensorHubError> {
        if (cap & CAP_SENSOR_READ) == 0 {
            return Err(SensorHubError::PermissionDenied);
        }
        for slot in self.subscribers.iter_mut() {
            if slot.is_none() {
                let id = self.next_sub_id;
                self.next_sub_id += 1;
                *slot = Some(SensorSubscriber::new(id, channel_mask));
                return Ok(id);
            }
        }
        Err(SensorHubError::SubscriberLimitExceeded)
    }

    pub fn sync_timestamp(&mut self, timestamp_us: u64) {
        self.last_timestamp_us = timestamp_us;
    }

    pub fn post_event(&mut self, channel: SensorChannel, timestamp_us: u64, data: [i32; 3]) -> Result<usize, SensorHubError> {
        let mask = channel.to_mask();
        let event = SensorEvent {
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

    pub fn pop_event(&mut self, sub_id: u32) -> Result<SensorEvent, SensorHubError> {
        for slot in self.subscribers.iter_mut() {
            if let Some(sub) = slot {
                if sub.id == sub_id {
                    return sub.pop();
                }
            }
        }
        Err(SensorHubError::SubscriberNotFound)
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
