//! Kernel Touch Digitizer Controller Driver (SP-78, REQ-HW-107, REQ-UX-102)
//!
//! Architectural Invariants:
//! - INV-TOUCH-001: GENI I2C bus address mapping (0x0880_0000, addr 0x38)
//! - INV-TOUCH-002: Multi-touch coordinate bounding [0, 1080) x [0, 2460)
//! - INV-TOUCH-003: Bounded interrupt-safe FIFO ring buffer

use super::interactive_shell_core::KernelActiveCard;

pub const KERNEL_HANOIP_GENI_I2C1_BASE: u64 = 0x0880_0000;
pub const KERNEL_TOUCH_I2C_SLAVE_ADDR: u8 = 0x38;
pub const KERNEL_TOUCH_QUEUE_CAPACITY: usize = 32;

// Physical hardware TLMM GPIO registers for Motorola Moto G60 (Qualcomm SM7150 / hanoip)
// TLMM Base: 0x0340_0000.
// GPIO 8: Touchscreen Reset Line (ts_reset)
pub const TLMM_GPIO_CFG_8: usize = 0x0340_8000;
pub const TLMM_GPIO_IN_OUT_8: usize = 0x0340_8004;
pub const TLMM_GPIO_INTR_CFG_8: usize = 0x0340_8008;
pub const TLMM_GPIO_INTR_STATUS_8: usize = 0x0340_800C;

// GPIO 9: Touchscreen Interrupt Line (ts_int, active-low / edge-falling)
pub const TLMM_GPIO_CFG_9: usize = 0x0340_9000;
pub const TLMM_GPIO_IN_OUT_9: usize = 0x0340_9004;
pub const TLMM_GPIO_INTR_CFG_9: usize = 0x0340_9008;
pub const TLMM_GPIO_INTR_STATUS_9: usize = 0x0340_900C;

// Live hardware register telemetry for on-screen HUD diagnostics
pub static mut TELEM_GPIO8_VAL: u32 = 0;
pub static mut TELEM_GPIO9_VAL: u32 = 0;
pub static mut TELEM_GPIO9_IRQ: u32 = 0;
pub static mut TELEM_TOUCH_COUNT: u32 = 0;

pub unsafe fn get_telemetry() -> (u32, u32, u32, u32) {
    (TELEM_GPIO8_VAL, TELEM_GPIO9_VAL, TELEM_GPIO9_IRQ, TELEM_TOUCH_COUNT)
}

// 19.2 MHz ARM generic physical counter ticks per millisecond on Qualcomm SM7150
pub const TICKS_PER_MS: u64 = 19_200;
// Debounce threshold: 15ms sustained state
pub const DEBOUNCE_TICKS: u64 = 15 * TICKS_PER_MS;
// Cooldown between tap activations: 200ms
pub const TAP_COOLDOWN_TICKS: u64 = 200 * TICKS_PER_MS;
// Maximum touch hold timeout: 350ms (guarantees the state machine can never deadlock)
pub const MAX_TOUCH_HOLD_TICKS: u64 = 350 * TICKS_PER_MS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelTouchAction {
    Down,
    Move,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelTouchError {
    InvalidMmioBase,
    InvalidSlaveAddress,
    OutOfBounds,
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelTouchPoint {
    pub id: u8,
    pub x: u32,
    pub y: u32,
    pub pressure: u8,
    pub action: KernelTouchAction,
}

impl KernelTouchPoint {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            x: 0,
            y: 0,
            pressure: 0,
            action: KernelTouchAction::Up,
        }
    }
}

pub struct KernelTouchControllerHarness {
    mmio_mapped: bool,
    queue: [KernelTouchPoint; KERNEL_TOUCH_QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
    overflow_count: u32,
    is_touch_down: bool,
    press_start_tick: u64,
    release_start_tick: u64,
    last_tap_tick: u64,
    touch_down_tick: u64,
    last_line_state: bool,
    initial_sample_captured: bool,
    nav_cycle: u32,
    hardware_initialized: bool,
}

impl KernelTouchControllerHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            queue: [KernelTouchPoint::empty(); KERNEL_TOUCH_QUEUE_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
            overflow_count: 0,
            is_touch_down: false,
            press_start_tick: 0,
            release_start_tick: 0,
            last_tap_tick: 0,
            touch_down_tick: 0,
            last_line_state: false,
            initial_sample_captured: false,
            nav_cycle: 0,
            hardware_initialized: false,
        }
    }

    pub fn map_bus(&mut self, base: u64, slave_addr: u8) -> Result<(), KernelTouchError> {
        if base != KERNEL_HANOIP_GENI_I2C1_BASE {
            return Err(KernelTouchError::InvalidMmioBase);
        }
        if slave_addr != KERNEL_TOUCH_I2C_SLAVE_ADDR {
            return Err(KernelTouchError::InvalidSlaveAddress);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn post_event(&mut self, point: KernelTouchPoint) -> Result<(), KernelTouchError> {
        if point.x >= 1080 || point.y >= 2460 {
            return Err(KernelTouchError::OutOfBounds);
        }
        if self.count >= KERNEL_TOUCH_QUEUE_CAPACITY {
            self.overflow_count += 1;
            // Drop oldest event to guarantee non-blocking behavior
            self.tail = (self.tail + 1) % KERNEL_TOUCH_QUEUE_CAPACITY;
            self.count -= 1;
        }
        self.queue[self.head] = point;
        self.head = (self.head + 1) % KERNEL_TOUCH_QUEUE_CAPACITY;
        self.count += 1;
        Ok(())
    }

    pub fn pop_event(&mut self) -> Option<KernelTouchPoint> {
        if self.count == 0 {
            return None;
        }
        let pt = self.queue[self.tail];
        self.tail = (self.tail + 1) % KERNEL_TOUCH_QUEUE_CAPACITY;
        self.count -= 1;
        Some(pt)
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_bus_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn is_touch_down(&self) -> bool {
        self.is_touch_down
    }

    pub fn nav_cycle(&self) -> u32 {
        self.nav_cycle
    }

    /// Hardware pin configuration for Motorola Moto G60:
    /// - Drive GPIO 8 (TS_RESET) HIGH so the Tianma ILI7807S TDDI touch controller stays active.
    ///   (Asserting low reset without a complete SPI reload puts TDDI into dead bootloader state).
    /// - Configure GPIO 9 (TS_INT) as input with internal Pull-Up and Dual-Edge interrupt detection.
    pub fn init_hardware_pins(&mut self) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            // 1. Configure GPIO 8 (Touch Reset line):
            // Output mode (OE=1), 8mA drive strength (DRV=3), Pull-Up (pull=2), GPIO function (func=0)
            let cfg8 = (1u32 << 9) | (3u32 << 6) | 2u32;
            core::ptr::write_volatile(TLMM_GPIO_CFG_8 as *mut u32, cfg8);
            core::ptr::write_volatile(TLMM_GPIO_IN_OUT_8 as *mut u32, 2); // Deassert reset (HIGH, bit 1)

            // 2. Configure GPIO 9 (Touch Interrupt line):
            // Input mode (OE=0), 8mA drive strength (DRV=3), Pull-Up (pull=2), GPIO function (func=0)
            let cfg9 = (0u32 << 9) | (3u32 << 6) | 2u32;
            core::ptr::write_volatile(TLMM_GPIO_CFG_9 as *mut u32, cfg9);

            // 3. Configure GPIO 9 Interrupt for edge detection:
            // INTR_CFG: Bit 0 = enable (1), Bits 4..5 = detect (4 for dual-edge, 2 for falling-edge)
            let intr_cfg = (4u32 << 4) | 1u32;
            core::ptr::write_volatile(TLMM_GPIO_INTR_CFG_9 as *mut u32, intr_cfg);

            // Clear any stale interrupt latch
            core::ptr::write_volatile(TLMM_GPIO_INTR_STATUS_9 as *mut u32, 0);
        }
        self.hardware_initialized = true;
    }

    /// Polls physical touch hardware on Qualcomm SM7150 (Moto G60 / hanoip)
    /// Reads TLMM GPIO 8 and GPIO 9 with edge detection, debouncing, and dead-lock guard.
    /// Returns true if the screen is currently being touched.
    pub fn poll_hardware(
        &mut self,
        is_hanoip: bool,
        current_tick: u64,
        active_card: KernelActiveCard,
    ) -> bool {
        if !is_hanoip || !self.mmio_mapped {
            return false;
        }

        if !self.hardware_initialized {
            self.init_hardware_pins();
        }

        #[cfg(target_arch = "aarch64")]
        let (line_low, _edge_latched) = {
            let io8 = unsafe { core::ptr::read_volatile(TLMM_GPIO_IN_OUT_8 as *const u32) };
            let io9 = unsafe { core::ptr::read_volatile(TLMM_GPIO_IN_OUT_9 as *const u32) };
            let irq9 = unsafe { core::ptr::read_volatile(TLMM_GPIO_INTR_STATUS_9 as *const u32) };

            unsafe {
                TELEM_GPIO8_VAL = io8;
                TELEM_GPIO9_VAL = io9;
                TELEM_GPIO9_IRQ = irq9;
            }

            let edge = (irq9 & 1) != 0;
            if edge {
                // Clear the latched interrupt by writing 0
                unsafe {
                    core::ptr::write_volatile(TLMM_GPIO_INTR_STATUS_9 as *mut u32, 0);
                }
            }

            // Bit 0 is 0 when line is low (touch active)
            let low = (io9 & 1) == 0;
            (low, edge)
        };
        #[cfg(not(target_arch = "aarch64"))]
        let (line_low, _edge_latched) = (false, false);

        self.process_hardware_state(line_low, current_tick, active_card);
        self.is_touch_down
    }

    /// Internal deterministic state machine with 15ms debouncing, 200ms cooldown, and 350ms deadlock guard.
    pub fn process_hardware_state(
        &mut self,
        line_low: bool,
        current_tick: u64,
        active_card: KernelActiveCard,
    ) {
        if !self.initial_sample_captured {
            self.initial_sample_captured = true;
            self.last_line_state = line_low;
        }

        // Auto-release timeout: if touch has been held down for longer than MAX_TOUCH_HOLD_TICKS (350ms),
        // automatically release to guarantee the state machine CAN NEVER PERMANENTLY DEADLOCK.
        if self.is_touch_down && current_tick.saturating_sub(self.touch_down_tick) >= MAX_TOUCH_HOLD_TICKS {
            self.is_touch_down = false;
            self.press_start_tick = 0;
            self.release_start_tick = current_tick;
            let pt = KernelTouchPoint {
                id: 0,
                x: 540,
                y: 1200,
                pressure: 0,
                action: KernelTouchAction::Up,
            };
            let _ = self.post_event(pt);
            self.last_line_state = line_low;
            return;
        }

        if line_low {
            self.release_start_tick = 0;
            if self.press_start_tick == 0 {
                self.press_start_tick = current_tick;
            }

            // Must be continuously held low for at least DEBOUNCE_TICKS (15ms)
            let press_duration = current_tick.saturating_sub(self.press_start_tick);
            let time_since_last_tap = current_tick.saturating_sub(self.last_tap_tick);

            if press_duration >= DEBOUNCE_TICKS
                && !self.is_touch_down
                && (self.last_tap_tick == 0 || time_since_last_tap >= TAP_COOLDOWN_TICKS)
            {
                self.is_touch_down = true;
                self.touch_down_tick = current_tick;
                self.last_tap_tick = current_tick;

                unsafe {
                    TELEM_TOUCH_COUNT = TELEM_TOUCH_COUNT.wrapping_add(1);
                }

                // Determine coordinates based on active screen surface:
                let (target_x, target_y) = match active_card {
                    // On Bento Grid Home (Telemetry):
                    // Advance through primary feature surfaces:
                    KernelActiveCard::Telemetry => {
                        let step = self.nav_cycle % 4;
                        self.nav_cycle = (self.nav_cycle + 1) % 4;
                        match step {
                            0 => (540, 550),  // Hero / AIS Consent Card (y: 400..750)
                            1 => (290, 950),  // Why Explorer Card (y: 800..1150, x <= 520)
                            2 => (790, 950),  // Indic Notes App (y: 800..1150, x >= 560)
                            _ => (540, 1350), // App Drawer / Terminal (y: 1200..1550)
                        }
                    }
                    // Inside an active app view, modal, or drawer:
                    // Any tap reliably hits Back / Return to Home (x: 150, y: 110)
                    _ => (150, 110),
                };

                let pt = KernelTouchPoint {
                    id: 0,
                    x: target_x,
                    y: target_y,
                    pressure: 128,
                    action: KernelTouchAction::Down,
                };
                let _ = self.post_event(pt);
            }
        } else {
            self.press_start_tick = 0;
            if self.release_start_tick == 0 {
                self.release_start_tick = current_tick;
            }

            // Must be continuously held high for at least DEBOUNCE_TICKS (15ms)
            let release_duration = current_tick.saturating_sub(self.release_start_tick);
            if release_duration >= DEBOUNCE_TICKS && self.is_touch_down {
                self.is_touch_down = false;
                let pt = KernelTouchPoint {
                    id: 0,
                    x: 540,
                    y: 1200,
                    pressure: 0,
                    action: KernelTouchAction::Up,
                };
                let _ = self.post_event(pt);
            }
        }

        self.last_line_state = line_low;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_initialization_and_bus_mapping() {
        let mut harness = KernelTouchControllerHarness::new();
        assert_eq!(harness.count(), 0);
        assert!(!harness.is_bus_mapped());

        let err_base = harness.map_bus(0x1234, KERNEL_TOUCH_I2C_SLAVE_ADDR);
        assert_eq!(err_base, Err(KernelTouchError::InvalidMmioBase));

        let err_addr = harness.map_bus(KERNEL_HANOIP_GENI_I2C1_BASE, 0x50);
        assert_eq!(err_addr, Err(KernelTouchError::InvalidSlaveAddress));

        assert!(harness.map_bus(KERNEL_HANOIP_GENI_I2C1_BASE, KERNEL_TOUCH_I2C_SLAVE_ADDR).is_ok());
        assert!(harness.is_bus_mapped());
    }

    #[test]
    fn test_touch_coordinate_bounding() {
        let mut harness = KernelTouchControllerHarness::new();
        let valid = KernelTouchPoint {
            id: 0,
            x: 540,
            y: 1230,
            pressure: 64,
            action: KernelTouchAction::Down,
        };
        assert!(harness.post_event(valid).is_ok());

        let invalid_x = KernelTouchPoint {
            id: 0,
            x: 1080,
            y: 1230,
            pressure: 64,
            action: KernelTouchAction::Down,
        };
        assert_eq!(harness.post_event(invalid_x), Err(KernelTouchError::OutOfBounds));

        let invalid_y = KernelTouchPoint {
            id: 0,
            x: 540,
            y: 2460,
            pressure: 64,
            action: KernelTouchAction::Down,
        };
        assert_eq!(harness.post_event(invalid_y), Err(KernelTouchError::OutOfBounds));
    }

    #[test]
    fn test_time_based_debouncing_and_rejection_of_spikes() {
        let mut harness = KernelTouchControllerHarness::new();
        harness.map_bus(KERNEL_HANOIP_GENI_I2C1_BASE, KERNEL_TOUCH_I2C_SLAVE_ADDR).unwrap();

        let base_tick = 1_000_000u64;

        // 1. Noise spike of 5ms (less than 15ms debounce threshold)
        harness.process_hardware_state(true, base_tick, KernelActiveCard::Telemetry);
        harness.process_hardware_state(true, base_tick + 5 * TICKS_PER_MS, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 0);
        assert!(!harness.is_touch_down());

        // 2. Glitch ends (line goes high again)
        harness.process_hardware_state(false, base_tick + 6 * TICKS_PER_MS, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 0);
        assert!(!harness.is_touch_down());

        // 3. Genuine physical touch held for 20ms (> 15ms threshold)
        let touch_start = base_tick + 100 * TICKS_PER_MS;
        harness.process_hardware_state(true, touch_start, KernelActiveCard::Telemetry);
        harness.process_hardware_state(true, touch_start + 10 * TICKS_PER_MS, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 0); // At 10ms, still debouncing

        harness.process_hardware_state(true, touch_start + 20 * TICKS_PER_MS, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 1); // At 20ms, verified contact!
        assert!(harness.is_touch_down());

        let ev = harness.pop_event().unwrap();
        assert_eq!(ev.action, KernelTouchAction::Down);
        assert_eq!((ev.x, ev.y), (540, 550)); // Target 1: Consent Modal

        // 4. Continued hold for 200ms does NOT produce extra duplicate events
        harness.process_hardware_state(true, touch_start + 150 * TICKS_PER_MS, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 0);

        // 5. Release for 20ms (> 15ms threshold) triggers Up event
        let release_start = touch_start + 250 * TICKS_PER_MS;
        harness.process_hardware_state(false, release_start, KernelActiveCard::Telemetry);
        harness.process_hardware_state(false, release_start + 20 * TICKS_PER_MS, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 1);
        assert!(!harness.is_touch_down());

        let up_ev = harness.pop_event().unwrap();
        assert_eq!(up_ev.action, KernelTouchAction::Up);
    }

    #[test]
    fn test_navigation_sequence_and_cooldown() {
        let mut harness = KernelTouchControllerHarness::new();
        harness.map_bus(KERNEL_HANOIP_GENI_I2C1_BASE, KERNEL_TOUCH_I2C_SLAVE_ADDR).unwrap();

        let mut tick = 10_000_000u64;

        // Step 1: On Home -> Tap opens Consent Modal
        harness.process_hardware_state(true, tick, KernelActiveCard::Telemetry);
        tick += 20 * TICKS_PER_MS;
        harness.process_hardware_state(true, tick, KernelActiveCard::Telemetry);
        let ev1 = harness.pop_event().unwrap();
        assert_eq!((ev1.x, ev1.y), (540, 550));

        // Release
        tick += 100 * TICKS_PER_MS;
        harness.process_hardware_state(false, tick, KernelActiveCard::Telemetry);
        tick += 20 * TICKS_PER_MS;
        harness.process_hardware_state(false, tick, KernelActiveCard::Telemetry);
        let _ = harness.pop_event();

        // Step 2: Next tap while on Consent Modal -> Back button (150, 110)
        tick += 250 * TICKS_PER_MS; // After 200ms cooldown
        harness.process_hardware_state(true, tick, KernelActiveCard::ConsentModal);
        tick += 20 * TICKS_PER_MS;
        harness.process_hardware_state(true, tick, KernelActiveCard::ConsentModal);
        let ev2 = harness.pop_event().unwrap();
        assert_eq!((ev2.x, ev2.y), (150, 110));

        // Release
        tick += 100 * TICKS_PER_MS;
        harness.process_hardware_state(false, tick, KernelActiveCard::ConsentModal);
        tick += 20 * TICKS_PER_MS;
        harness.process_hardware_state(false, tick, KernelActiveCard::ConsentModal);
        let _ = harness.pop_event();

        // Step 3: Next tap while on Home -> Why Explorer (290, 950)
        tick += 250 * TICKS_PER_MS;
        harness.process_hardware_state(true, tick, KernelActiveCard::Telemetry);
        tick += 20 * TICKS_PER_MS;
        harness.process_hardware_state(true, tick, KernelActiveCard::Telemetry);
        let ev3 = harness.pop_event().unwrap();
        assert_eq!((ev3.x, ev3.y), (290, 950));
    }
}
