//! Kernel Display Panel & DSI Controller Driver (SP-76, PEN-004, REQ-HW-106)
//!
//! Architectural Invariants:
//! - INV-DISP-001: MDSS DSI host controller MMIO mapping (0x0ae9_4000)
//! - INV-DISP-002: MIPI DCS panel initialization command sequencing
//! - INV-DISP-003: Backlight PWM duty cycle bounding [0, 100]

pub const KERNEL_HANOIP_MDSS_DSI0_BASE: u64 = 0x0ae9_4000;
pub const KERNEL_PANEL_WIDTH: u32 = 1080;
pub const KERNEL_PANEL_HEIGHT: u32 = 2460;
pub const KERNEL_PANEL_REFRESH_HZ: u32 = 120;

pub const KERNEL_DCS_CMD_SLEEP_OUT: u8 = 0x11;
pub const KERNEL_DCS_CMD_DISPLAY_ON: u8 = 0x29;
pub const KERNEL_DCS_CMD_PIXEL_FORMAT: u8 = 0x3a;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelDisplayError {
    InvalidMmioBase,
    InvalidCommand,
    InvalidDutyCycle,
    NotInitialized,
}

pub struct KernelDisplayPanelHarness {
    mmio_mapped: bool,
    sleep_out: bool,
    display_on: bool,
    pixel_format_rgb888: bool,
    backlight_pct: u8,
}

impl KernelDisplayPanelHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            sleep_out: false,
            display_on: false,
            pixel_format_rgb888: false,
            backlight_pct: 0,
        }
    }

    pub fn map_mmio(&mut self, base: u64) -> Result<(), KernelDisplayError> {
        if base != KERNEL_HANOIP_MDSS_DSI0_BASE {
            return Err(KernelDisplayError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn send_dcs_command(&mut self, cmd: u8) -> Result<(), KernelDisplayError> {
        if !self.mmio_mapped {
            return Err(KernelDisplayError::NotInitialized);
        }
        match cmd {
            KERNEL_DCS_CMD_SLEEP_OUT => self.sleep_out = true,
            KERNEL_DCS_CMD_PIXEL_FORMAT => self.pixel_format_rgb888 = true,
            KERNEL_DCS_CMD_DISPLAY_ON => {
                if !self.sleep_out || !self.pixel_format_rgb888 {
                    return Err(KernelDisplayError::InvalidCommand);
                }
                self.display_on = true;
            }
            _ => return Err(KernelDisplayError::InvalidCommand),
        }
        Ok(())
    }

    pub fn set_backlight(&mut self, duty_pct: u8) -> Result<(), KernelDisplayError> {
        if duty_pct > 100 {
            return Err(KernelDisplayError::InvalidDutyCycle);
        }
        self.backlight_pct = duty_pct;
        Ok(())
    }

    pub fn is_display_active(&self) -> bool {
        self.mmio_mapped && self.sleep_out && self.display_on && self.pixel_format_rgb888
    }

    pub fn backlight_pct(&self) -> u8 {
        self.backlight_pct
    }
}
