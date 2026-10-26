//! Circular Ring Buffer for Kernel Telemetry Events
//! Implements bounded lock-protected circular buffer with monotonic sequence numbers per INDOS-SPEC-TELEM-01.

use alloc::vec::Vec;
use crate::telemetry::event::{TelemetryEvent, TelemetryEventKind};

pub const RING_CAPACITY: usize = 64;

/// Circular ring buffer storing high-frequency kernel telemetry events
pub struct TelemetryRingBuffer {
    events: [Option<TelemetryEvent>; RING_CAPACITY],
    head_seq: u64,
    total_emitted: u64,
}

impl TelemetryRingBuffer {
    pub const fn new() -> Self {
        Self {
            events: [None; RING_CAPACITY],
            head_seq: 1, // 1-indexed sequence numbering
            total_emitted: 0,
        }
    }

    /// Emits a new telemetry event into the circular ring buffer
    pub fn emit(&mut self, timestamp_ticks: u64, kind: TelemetryEventKind) -> u64 {
        let seq = self.head_seq;
        let event = TelemetryEvent::new(seq, timestamp_ticks, kind);
        let idx = (seq as usize) % RING_CAPACITY;

        self.events[idx] = Some(event);
        self.head_seq += 1;
        self.total_emitted += 1;

        seq
    }

    /// Reads a single event by exact sequence number
    /// Returns None if the event has been overwritten or not yet produced
    pub fn read_event(&self, seq: u64) -> Option<TelemetryEvent> {
        if seq == 0 || seq >= self.head_seq {
            return None;
        }

        // Check if event was already overwritten by ring wrap-around
        if self.head_seq > RING_CAPACITY as u64 && seq <= self.head_seq - (RING_CAPACITY as u64) {
            return None;
        }

        let idx = (seq as usize) % RING_CAPACITY;
        if let Some(event) = self.events[idx] {
            if event.seq == seq {
                return Some(event);
            }
        }
        None
    }

    /// Reads a batch of events starting from start_seq up to max_count
    /// Returns (events, next_seq)
    pub fn read_batch(&self, start_seq: u64, max_count: usize) -> (Vec<TelemetryEvent>, u64) {
        let mut results = Vec::new();
        let oldest = self.oldest_available_seq();
        let mut curr_seq = if start_seq < oldest { oldest } else { start_seq };

        while curr_seq < self.head_seq && results.len() < max_count {
            if let Some(event) = self.read_event(curr_seq) {
                results.push(event);
                curr_seq += 1;
            } else {
                curr_seq += 1;
            }
        }

        (results, curr_seq)
    }

    /// Returns the oldest sequence number still retained in the ring buffer
    pub fn oldest_available_seq(&self) -> u64 {
        if self.head_seq <= (RING_CAPACITY as u64) {
            1
        } else {
            self.head_seq - (RING_CAPACITY as u64)
        }
    }

    pub fn head_seq(&self) -> u64 {
        self.head_seq
    }

    pub fn total_emitted(&self) -> u64 {
        self.total_emitted
    }

    pub fn capacity(&self) -> usize {
        RING_CAPACITY
    }

    pub fn clear(&mut self) {
        self.events = [None; RING_CAPACITY];
        self.head_seq = 1;
        self.total_emitted = 0;
    }
}
