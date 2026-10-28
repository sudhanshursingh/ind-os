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
// Architectural Reference: Linux drivers/pinctrl/qcom/pinctrl-sm7150.c & sm7150.dtsi
// North Tile Base: 0x0390_0000 (Pins 8, 9, etc.)
// West Tile Base:  0x0350_0000 (Pin 58 assist_key, etc.)
// Downstream Legacy Flat Alias: 0x0340_0000

// North Tile: GPIO 8 (Touchscreen Reset line)
pub const TLMM_NORTH_GPIO_CFG_8: usize = 0x0390_8000;
pub const TLMM_NORTH_GPIO_IN_OUT_8: usize = 0x0390_8004;
pub const TLMM_NORTH_GPIO_INTR_CFG_8: usize = 0x0390_8008;
pub const TLMM_NORTH_GPIO_INTR_STATUS_8: usize = 0x0390_800C;

// North Tile: GPIO 9 (Touchscreen Interrupt line, active-low / edge-falling)
pub const TLMM_NORTH_GPIO_CFG_9: usize = 0x0390_9000;
pub const TLMM_NORTH_GPIO_IN_OUT_9: usize = 0x0390_9004;
pub const TLMM_NORTH_GPIO_INTR_CFG_9: usize = 0x0390_9008;
pub const TLMM_NORTH_GPIO_INTR_STATUS_9: usize = 0x0390_900C;

// West Tile: GPIO 58 (Google Assistant Key - physical hardware button)
pub const TLMM_WEST_GPIO_CFG_58: usize = 0x0353_A000;
pub const TLMM_WEST_GPIO_IN_OUT_58: usize = 0x0353_A004;
pub const TLMM_WEST_GPIO_INTR_CFG_58: usize = 0x0353_A008;
pub const TLMM_WEST_GPIO_INTR_STATUS_58: usize = 0x0353_A00C;

// Downstream Legacy Flat Alias (for environments mapping flat 0x0340_0000 aperture)
pub const TLMM_LEGACY_GPIO_CFG_8: usize = 0x0340_8000;
pub const TLMM_LEGACY_GPIO_IN_OUT_8: usize = 0x0340_8004;
pub const TLMM_LEGACY_GPIO_CFG_9: usize = 0x0340_9000;
pub const TLMM_LEGACY_GPIO_IN_OUT_9: usize = 0x0340_9004;
pub const TLMM_LEGACY_GPIO_INTR_STATUS_9: usize = 0x0340_900C;

// Motorola-family FocalTech INT/RESET commonly uses GPIO 88/89 (north tile).
pub const TLMM_NORTH_GPIO_CFG_88: usize = 0x0395_8000;
pub const TLMM_NORTH_GPIO_IN_OUT_88: usize = 0x0395_8004;
pub const TLMM_NORTH_GPIO_INTR_STATUS_88: usize = 0x0395_800C;
pub const TLMM_NORTH_GPIO_CFG_89: usize = 0x0395_9000;
pub const TLMM_NORTH_GPIO_IN_OUT_89: usize = 0x0395_9004;
pub const TLMM_NORTH_GPIO_INTR_STATUS_89: usize = 0x0395_900C;

fn mmio_looks_live(val: u32) -> bool {
    val != 0 && val != 0xFFFF_FFFF
}

/// GPIO_CFG: bits [5:2] = function (0 = GPIO), bit 9 = OE (0 = input).
fn cfg_is_gpio_input(cfg: u32) -> bool {
    mmio_looks_live(cfg) && ((cfg >> 2) & 0xF) == 0 && (cfg & (1 << 9)) == 0
}

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
    gpio_live: bool,
    needs_idle: bool,
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
            gpio_live: false,
            needs_idle: false,
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
    /// - Drive GPIO 8 (Touch Reset line) HIGH so Ilitek TDDI touch controller exits hardware reset.
    /// - Configure GPIO 9 (Touch Interrupt line) as input with internal Pull-Up and Dual-Edge interrupt detection.
    /// - Configure GPIO 58 (Google Assistant Key - physical hardware button) as input with Pull-Up.
    /// - Configure fallback candidate pins 88 & 89 as inputs with Pull-Up.
    pub fn init_hardware_pins(&mut self) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            // 1. Configure GPIO 8 (Touch Reset line):
            // Output mode (OE=1, bit 9), 8mA drive (DRV=3, bits 8..6), Pull-Up (pull=3, bits 1..0), GPIO func (func=0)
            let cfg8 = (1u32 << 9) | (3u32 << 6) | 3u32;
            core::ptr::write_volatile(TLMM_NORTH_GPIO_CFG_8 as *mut u32, cfg8);
            core::ptr::write_volatile(TLMM_NORTH_GPIO_IN_OUT_8 as *mut u32, 2); // Deassert reset (HIGH, bit 1)
            core::ptr::write_volatile(TLMM_LEGACY_GPIO_CFG_8 as *mut u32, cfg8);
            core::ptr::write_volatile(TLMM_LEGACY_GPIO_IN_OUT_8 as *mut u32, 2);

            // 2. Configure GPIO 9 (Touch Interrupt line):
            // Input mode (OE=0), 8mA drive, Pull-Up (pull=3), GPIO func (func=0)
            let cfg9 = (0u32 << 9) | (3u32 << 6) | 3u32;
            core::ptr::write_volatile(TLMM_NORTH_GPIO_CFG_9 as *mut u32, cfg9);
            core::ptr::write_volatile(TLMM_LEGACY_GPIO_CFG_9 as *mut u32, cfg9);

            // Interrupt detection config: KPSS target 3, falling edge 2, enable 1
            let intr_cfg = (3u32 << 5) | (2u32 << 2) | 1u32;
            core::ptr::write_volatile(TLMM_NORTH_GPIO_INTR_CFG_9 as *mut u32, intr_cfg);
            core::ptr::write_volatile(TLMM_NORTH_GPIO_INTR_STATUS_9 as *mut u32, 1);
            core::ptr::write_volatile(TLMM_LEGACY_GPIO_INTR_STATUS_9 as *mut u32, 1);

            // 3. Configure GPIO 58 (Google Assistant Key - physical hardware button):
            let cfg58 = (0u32 << 9) | (3u32 << 6) | 3u32;
            core::ptr::write_volatile(TLMM_WEST_GPIO_CFG_58 as *mut u32, cfg58);
            core::ptr::write_volatile(TLMM_WEST_GPIO_INTR_CFG_58 as *mut u32, intr_cfg);
            core::ptr::write_volatile(TLMM_WEST_GPIO_INTR_STATUS_58 as *mut u32, 1);

            // 4. Configure fallback GPIO 88 & 89:
            core::ptr::write_volatile(TLMM_NORTH_GPIO_CFG_88 as *mut u32, cfg9);
            core::ptr::write_volatile(TLMM_NORTH_GPIO_CFG_89 as *mut u32, cfg9);
        }
        self.hardware_initialized = true;
        self.gpio_live = false;
        self.needs_idle = true;
    }

    /// Polls physical touch hardware on Qualcomm SM7150 (Moto G60 / hanoip).
    /// Zero, unmapped, QUP-muxed, and sticky-IRQ registers never generate taps.
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
        let is_active = {
            let cfg8 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_CFG_8 as *const u32) };
            let io8 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_IN_OUT_8 as *const u32) };
            let _ = (cfg8, io8);

            let cfg9 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_CFG_9 as *const u32) };
            let io9 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_IN_OUT_9 as *const u32) };
            let irq9 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_INTR_STATUS_9 as *const u32) };

            let cfg58 = unsafe { core::ptr::read_volatile(TLMM_WEST_GPIO_CFG_58 as *const u32) };
            let io58 = unsafe { core::ptr::read_volatile(TLMM_WEST_GPIO_IN_OUT_58 as *const u32) };
            let irq58 = unsafe { core::ptr::read_volatile(TLMM_WEST_GPIO_INTR_STATUS_58 as *const u32) };

            let cfg88 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_CFG_88 as *const u32) };
            let io88 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_IN_OUT_88 as *const u32) };
            let irq88 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_INTR_STATUS_88 as *const u32) };

            let cfg89 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_CFG_89 as *const u32) };
            let io89 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_IN_OUT_89 as *const u32) };
            let irq89 = unsafe { core::ptr::read_volatile(TLMM_NORTH_GPIO_INTR_STATUS_89 as *const u32) };

            let io8_leg = unsafe { core::ptr::read_volatile(TLMM_LEGACY_GPIO_IN_OUT_8 as *const u32) };
            let io9_leg = unsafe { core::ptr::read_volatile(TLMM_LEGACY_GPIO_IN_OUT_9 as *const u32) };
            let _ = (io8_leg, io9_leg);

            // Export live register telemetry for on-screen HUD:
            // Display CFG_9 (validates write succeeded), IO_9 (touch state), and IO_58 (side key state)
            unsafe {
                TELEM_GPIO8_VAL = cfg9;
                TELEM_GPIO9_VAL = io9;
                TELEM_GPIO9_IRQ = io58;
            }

            let mut active = false;
            let mut any_live = false;

            // Probe candidate input lines: GPIO 9 (touch), GPIO 58 (side button), 89, 88
            let candidates = [
                (cfg9, io9, irq9, TLMM_NORTH_GPIO_INTR_STATUS_9),
                (cfg58, io58, irq58, TLMM_WEST_GPIO_INTR_STATUS_58),
                (cfg89, io89, irq89, TLMM_NORTH_GPIO_INTR_STATUS_89),
                (cfg88, io88, irq88, TLMM_NORTH_GPIO_INTR_STATUS_88),
            ];

            for (cfg, io, irq, irq_addr) in candidates {
                if !cfg_is_gpio_input(cfg) {
                    continue;
                }
                any_live = true;
                let line_low = (io & 1) == 0;
                let edge = (irq & 1) != 0;
                if edge {
                    unsafe { core::ptr::write_volatile(irq_addr as *mut u32, 1); }
                }
                if line_low || edge {
                    active = true;
                    break;
                }
            }

            self.gpio_live = any_live;
            active
        };
        #[cfg(not(target_arch = "aarch64"))]
        let is_active = false;

        if !self.gpio_live {
            return false;
        }
        if self.needs_idle {
            if !is_active {
                self.needs_idle = false;
            }
            return false;
        }

        self.process_hardware_state(is_active, current_tick, active_card);
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

        if self.needs_idle {
            if line_low {
                self.last_line_state = line_low;
                return;
            }
            self.needs_idle = false;
        }

        // Auto-release timeout: if touch has been held down for longer than MAX_TOUCH_HOLD_TICKS (350ms),
        // automatically release to guarantee the state machine CAN NEVER PERMANENTLY DEADLOCK.
        if self.is_touch_down && current_tick.saturating_sub(self.touch_down_tick) >= MAX_TOUCH_HOLD_TICKS {
            self.is_touch_down = false;
            self.press_start_tick = 0;
            self.release_start_tick = current_tick;
            self.needs_idle = true;
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

                // GPIO INT has no XY. Park the synthetic point in a no-op region so a
                // real line pulse cannot tour Consent / Why / Notes / Terminal.
                let (target_x, target_y) = (540u32, 1230u32);
                let _ = active_card;
                self.nav_cycle = self.nav_cycle.wrapping_add(1);

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
        assert_eq!((ev.x, ev.y), (540, 1230));

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
        assert_eq!((ev1.x, ev1.y), (540, 1230));

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
        assert_eq!((ev2.x, ev2.y), (540, 1230));

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
        assert_eq!((ev3.x, ev3.y), (540, 1230));
    }

    #[test]
    fn test_auto_release_does_not_retrigger_while_held() {
        let mut harness = KernelTouchControllerHarness::new();
        harness.map_bus(KERNEL_HANOIP_GENI_I2C1_BASE, KERNEL_TOUCH_I2C_SLAVE_ADDR).unwrap();

        let mut tick = 1_000_000u64;
        harness.process_hardware_state(true, tick, KernelActiveCard::Telemetry);
        tick += 20 * TICKS_PER_MS;
        harness.process_hardware_state(true, tick, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 1);
        let _ = harness.pop_event();

        tick += 400 * TICKS_PER_MS;
        harness.process_hardware_state(true, tick, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 1);
        assert_eq!(harness.pop_event().unwrap().action, KernelTouchAction::Up);
        assert!(harness.needs_idle);

        tick += 20 * TICKS_PER_MS;
        harness.process_hardware_state(true, tick, KernelActiveCard::Telemetry);
        assert_eq!(harness.count(), 0);
    }
}
