//! Touch Digitizer Controller Driver (SP-78, REQ-HW-107, REQ-UX-102)
//!
//! Architectural Invariants:
//! - INV-TOUCH-001: GENI I2C bus address mapping (0x0880_0000, addr 0x38)
//! - INV-TOUCH-002: Multi-touch coordinate bounding [0, 1080) x [0, 2460)
//! - INV-TOUCH-003: Bounded interrupt-safe FIFO ring buffer

pub const HANOIP_GENI_I2C1_BASE: u64 = 0x0880_0000;
pub const TOUCH_I2C_SLAVE_ADDR: u8 = 0x38;
pub const TOUCH_QUEUE_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchAction {
    Down,
    Move,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchError {
    InvalidMmioBase,
    InvalidSlaveAddress,
    OutOfBounds,
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchPoint {
    pub id: u8,
    pub x: u32,
    pub y: u32,
    pub pressure: u8,
    pub action: TouchAction,
}

impl TouchPoint {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            x: 0,
            y: 0,
            pressure: 0,
            action: TouchAction::Up,
        }
    }
}

pub struct TouchControllerHarness {
    mmio_mapped: bool,
    queue: [TouchPoint; TOUCH_QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
    overflow_count: u32,
}

impl TouchControllerHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            queue: [TouchPoint::empty(); TOUCH_QUEUE_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
            overflow_count: 0,
        }
    }

    pub fn map_bus(&mut self, base: u64, slave_addr: u8) -> Result<(), TouchError> {
        if base != HANOIP_GENI_I2C1_BASE {
            return Err(TouchError::InvalidMmioBase);
        }
        if slave_addr != TOUCH_I2C_SLAVE_ADDR {
            return Err(TouchError::InvalidSlaveAddress);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn post_event(&mut self, point: TouchPoint) -> Result<(), TouchError> {
        if point.x >= 1080 || point.y >= 2460 {
            return Err(TouchError::OutOfBounds);
        }
        if self.count >= TOUCH_QUEUE_CAPACITY {
            self.overflow_count += 1;
            // Drop oldest event to guarantee non-blocking behavior
            self.tail = (self.tail + 1) % TOUCH_QUEUE_CAPACITY;
            self.count -= 1;
        }
        self.queue[self.head] = point;
        self.head = (self.head + 1) % TOUCH_QUEUE_CAPACITY;
        self.count += 1;
        Ok(())
    }

    pub fn pop_event(&mut self) -> Option<TouchPoint> {
        if self.count == 0 {
            return None;
        }
        let pt = self.queue[self.tail];
        self.tail = (self.tail + 1) % TOUCH_QUEUE_CAPACITY;
        self.count -= 1;
        Some(pt)
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_bus_mapped(&self) -> bool {
        self.mmio_mapped
    }
}
