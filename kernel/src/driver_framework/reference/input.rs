//! User-Mode Keyboard & Mouse Input Reference Driver
//! Compliant with Bible §8.10 (SP-27), INDOS-SPEC-REFDRV-01, Rule N-01, Rule N-04, and REQ-KRN-107.

use crate::capability::{CapType, Capability};
use crate::task::ProcessId;

pub const MAX_INPUT_EVENTS: usize = 32;

/// Input event type categorization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventType {
    None,
    KeyPress,
    KeyRelease,
    MouseMove,
    MouseButton,
}

/// Bounded input event packet delivered to client services
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub event_type: InputEventType,
    pub code: u16,
    pub value: i32,
    pub buttons: u8,
}

impl InputEvent {
    pub const fn empty() -> Self {
        Self {
            event_type: InputEventType::None,
            code: 0,
            value: 0,
            buttons: 0,
        }
    }
}

/// User-mode Keyboard & Mouse input driver instance running in isolated user process
#[derive(Clone, Copy, Debug)]
pub struct InputDriver {
    pub device_id: u32,
    pub owner_pid: ProcessId,
    pub is_ready: bool,
    pub events: [InputEvent; MAX_INPUT_EVENTS],
    pub head: usize,
    pub tail: usize,
    pub count: usize,
    pub total_events: u64,
}

impl InputDriver {
    pub const fn new(device_id: u32, owner_pid: ProcessId) -> Self {
        Self {
            device_id,
            owner_pid,
            is_ready: false,
            events: [InputEvent::empty(); MAX_INPUT_EVENTS],
            head: 0,
            tail: 0,
            count: 0,
            total_events: 0,
        }
    }

    /// Initializes input controller using granted device capability token (INV-DRV-001)
    pub fn init(&mut self, dev_cap: &Capability) -> Result<(), &'static str> {
        if !dev_cap.is_valid || dev_cap.cap_type != CapType::Device(self.device_id) {
            return Err("EPERM: Access denied: Invalid device capability for input driver (INV-DRV-001)");
        }
        self.is_ready = true;
        Ok(())
    }

    /// Ingests a raw hardware key event into the circular ring buffer
    pub fn push_key_event(&mut self, keycode: u16, pressed: bool) {
        if !self.is_ready || self.count >= MAX_INPUT_EVENTS {
            return; // Drop on overflow to prevent memory exhaustion
        }

        let event = InputEvent {
            event_type: if pressed { InputEventType::KeyPress } else { InputEventType::KeyRelease },
            code: keycode,
            value: if pressed { 1 } else { 0 },
            buttons: 0,
        };

        self.events[self.head] = event;
        self.head = (self.head + 1) % MAX_INPUT_EVENTS;
        self.count += 1;
        self.total_events += 1;
    }

    /// Ingests a raw hardware mouse event into the circular ring buffer
    pub fn push_mouse_event(&mut self, dx: i16, dy: i16, buttons: u8) {
        if !self.is_ready || self.count >= MAX_INPUT_EVENTS {
            return;
        }

        let event = InputEvent {
            event_type: InputEventType::MouseMove,
            code: dx as u16,
            value: dy as i32,
            buttons,
        };

        self.events[self.head] = event;
        self.head = (self.head + 1) % MAX_INPUT_EVENTS;
        self.count += 1;
        self.total_events += 1;
    }

    /// Pops the next pending input event from the circular buffer
    pub fn pop_event(&mut self) -> Option<InputEvent> {
        if self.count == 0 {
            return None;
        }

        let ev = self.events[self.tail];
        self.tail = (self.tail + 1) % MAX_INPUT_EVENTS;
        self.count -= 1;
        Some(ev)
    }

    /// Returns the number of currently queued input events
    pub fn pending_events(&self) -> usize {
        self.count
    }
}
