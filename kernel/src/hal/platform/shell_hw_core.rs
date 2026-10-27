//! Kernel Shell v2 on Physical Display and Touch Core (SP-100, REQ-HW-106, REQ-UX-102)
//!
//! Direct-to-DSI framebuffer zero-copy swapchain and low-latency touch gesture arbiter.

pub const KERNEL_CAP_DISPLAY_RENDER: u32 = 0x0000_0020;
pub const KERNEL_CAP_TOUCH_INPUT: u32 = 0x0000_0040;

pub const KERNEL_SHELL_PANEL_WIDTH: u32 = 1080;
pub const KERNEL_SHELL_PANEL_HEIGHT: u32 = 2460;
pub const KERNEL_SHELL_REFRESH_RATE_HZ: u32 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelGestureType {
    Tap,
    Drag,
    Fling,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelShellTouchPoint {
    pub x: u32,
    pub y: u32,
    pub pressure: u16,
    pub touch_id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelGestureEvent {
    pub gesture: KernelGestureType,
    pub x: u32,
    pub y: u32,
    pub velocity_px_per_s: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelShellHwError {
    PermissionDenied,
    InvalidCoordinates,
    PanelNotReady,
}

pub struct KernelShellHwHarness {
    is_active: bool,
    frame_count: u32,
    back_buffer_idx: u8,
    gestures_dispatched: u32,
    last_point: Option<KernelShellTouchPoint>,
    chrome_rendered: bool,
}

impl KernelShellHwHarness {
    pub const fn new() -> Self {
        Self {
            is_active: false,
            frame_count: 0,
            back_buffer_idx: 0,
            gestures_dispatched: 0,
            last_point: None,
            chrome_rendered: false,
        }
    }

    pub fn bind_surface(&mut self, cap_token: u32) -> Result<(), KernelShellHwError> {
        if (cap_token & KERNEL_CAP_DISPLAY_RENDER) == 0 {
            return Err(KernelShellHwError::PermissionDenied);
        }
        self.is_active = true;
        self.back_buffer_idx = 0;
        Ok(())
    }

    pub fn process_touch(
        &mut self,
        point: KernelShellTouchPoint,
        cap_token: u32,
    ) -> Result<Option<KernelGestureEvent>, KernelShellHwError> {
        if (cap_token & KERNEL_CAP_TOUCH_INPUT) == 0 {
            return Err(KernelShellHwError::PermissionDenied);
        }
        if point.x >= KERNEL_SHELL_PANEL_WIDTH || point.y >= KERNEL_SHELL_PANEL_HEIGHT {
            return Err(KernelShellHwError::InvalidCoordinates);
        }

        let gesture = if let Some(prev) = self.last_point {
            let dx = if point.x > prev.x { point.x - prev.x } else { prev.x - point.x };
            let dy = if point.y > prev.y { point.y - prev.y } else { prev.y - point.y };
            let dist = dx + dy;

            if point.pressure == 0 {
                self.last_point = None;
                Some(KernelGestureEvent {
                    gesture: KernelGestureType::Release,
                    x: point.x,
                    y: point.y,
                    velocity_px_per_s: 0,
                })
            } else if dist > 60 {
                self.last_point = Some(point);
                Some(KernelGestureEvent {
                    gesture: KernelGestureType::Fling,
                    x: point.x,
                    y: point.y,
                    velocity_px_per_s: dist * 60,
                })
            } else if dist > 10 {
                self.last_point = Some(point);
                Some(KernelGestureEvent {
                    gesture: KernelGestureType::Drag,
                    x: point.x,
                    y: point.y,
                    velocity_px_per_s: dist * 30,
                })
            } else {
                Some(KernelGestureEvent {
                    gesture: KernelGestureType::Tap,
                    x: point.x,
                    y: point.y,
                    velocity_px_per_s: 0,
                })
            }
        } else {
            self.last_point = Some(point);
            Some(KernelGestureEvent {
                gesture: KernelGestureType::Tap,
                x: point.x,
                y: point.y,
                velocity_px_per_s: 0,
            })
        };

        if gesture.is_some() {
            self.gestures_dispatched += 1;
        }

        Ok(gesture)
    }

    pub fn render_chrome(&mut self) -> Result<(), KernelShellHwError> {
        if !self.is_active {
            return Err(KernelShellHwError::PanelNotReady);
        }
        self.chrome_rendered = true;
        Ok(())
    }

    pub fn swap_buffers(&mut self, cap_token: u32) -> Result<u32, KernelShellHwError> {
        if (cap_token & KERNEL_CAP_DISPLAY_RENDER) == 0 {
            return Err(KernelShellHwError::PermissionDenied);
        }
        if !self.is_active {
            return Err(KernelShellHwError::PanelNotReady);
        }

        self.back_buffer_idx = (self.back_buffer_idx + 1) % 2;
        self.frame_count += 1;
        Ok(self.frame_count)
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn gestures_dispatched(&self) -> u32 {
        self.gestures_dispatched
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_chrome_rendered(&self) -> bool {
        self.chrome_rendered
    }
}
