//! Kernel Compositor Verification Harness (SP-60)
//!
//! Architectural Invariants:
//! - INV-COMP-001: Capability-mediated display access (CAP_DISPLAY_RENDER / OVERLAY)
//! - INV-COMP-002: Bounded damage tracking with zero heap allocation in render fast path
//! - INV-COMP-003: Anti-overlay consent protection (topmost unforgeable z-order)
//! - INV-COMP-004: Focus integrity and strict input routing isolation

pub const KERNEL_MAX_WINDOWS: usize = 16;
pub const KERNEL_MAX_DIRTY_RECTS: usize = 8;
pub const KERNEL_SYSTEM_OVERLAY_Z_BASE: u32 = 100_000;

pub const KERNEL_CAP_DISPLAY_RENDER: u32 = 0x0001_0000;
pub const KERNEL_CAP_DISPLAY_OVERLAY: u32 = 0x0002_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelCompositorError {
    PermissionDenied,
    WindowTableFull,
    WindowNotFound,
    InvalidBounds,
    DamagePoolFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KernelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl KernelRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < (self.x + self.width) && py >= self.y && py < (self.y + self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelInputEventType {
    PointerDown,
    PointerUp,
    PointerMove,
    KeyPress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelInputEvent {
    pub event_type: KernelInputEventType,
    pub x: u32,
    pub y: u32,
    pub key_code: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelWindowSurface {
    pub window_id: u32,
    pub owner_pid: u32,
    pub bounds: KernelRect,
    pub z_order: u32,
    pub is_system_overlay: bool,
    pub damage: [Option<KernelRect>; KERNEL_MAX_DIRTY_RECTS],
    pub damage_count: usize,
    pub has_focus: bool,
}

pub struct KernelCompositorEngine {
    windows: [Option<KernelWindowSurface>; KERNEL_MAX_WINDOWS],
    next_window_id: u32,
    focused_window_id: Option<u32>,
    vsync_count: u64,
}

impl KernelCompositorEngine {
    pub const fn new() -> Self {
        Self {
            windows: [None; KERNEL_MAX_WINDOWS],
            next_window_id: 1,
            focused_window_id: None,
            vsync_count: 0,
        }
    }

    pub fn create_surface(
        &mut self,
        cap_tokens: u32,
        owner_pid: u32,
        bounds: KernelRect,
        is_overlay: bool,
    ) -> Result<u32, KernelCompositorError> {
        if (cap_tokens & KERNEL_CAP_DISPLAY_RENDER) == 0 {
            return Err(KernelCompositorError::PermissionDenied);
        }

        if is_overlay && (cap_tokens & KERNEL_CAP_DISPLAY_OVERLAY) == 0 {
            return Err(KernelCompositorError::PermissionDenied);
        }

        let z_order = if is_overlay {
            KERNEL_SYSTEM_OVERLAY_Z_BASE + self.next_window_id
        } else {
            self.next_window_id
        };

        for slot in self.windows.iter_mut() {
            if slot.is_none() {
                let id = self.next_window_id;
                self.next_window_id += 1;

                *slot = Some(KernelWindowSurface {
                    window_id: id,
                    owner_pid,
                    bounds,
                    z_order,
                    is_system_overlay: is_overlay,
                    damage: [None; KERNEL_MAX_DIRTY_RECTS],
                    damage_count: 0,
                    has_focus: false,
                });

                self.focused_window_id = Some(id);
                return Ok(id);
            }
        }

        Err(KernelCompositorError::WindowTableFull)
    }

    pub fn post_damage(
        &mut self,
        window_id: u32,
        dirty: KernelRect,
    ) -> Result<(), KernelCompositorError> {
        for slot in self.windows.iter_mut() {
            if let Some(ref mut win) = slot {
                if win.window_id == window_id {
                    if win.damage_count < KERNEL_MAX_DIRTY_RECTS {
                        win.damage[win.damage_count] = Some(dirty);
                        win.damage_count += 1;
                        return Ok(());
                    } else {
                        return Err(KernelCompositorError::DamagePoolFull);
                    }
                }
            }
        }
        Err(KernelCompositorError::WindowNotFound)
    }

    pub fn route_input(&mut self, event: &KernelInputEvent) -> Result<u32, KernelCompositorError> {
        let mut overlay_id = None;
        let mut top_z = 0;

        for slot in self.windows.iter() {
            if let Some(ref win) = slot {
                if win.is_system_overlay
                    && win.bounds.contains(event.x, event.y)
                    && win.z_order > top_z
                {
                    top_z = win.z_order;
                    overlay_id = Some(win.window_id);
                }
            }
        }

        if let Some(id) = overlay_id {
            self.focused_window_id = Some(id);
            return Ok(id);
        }

        let mut target_id = None;
        let mut current_max_z = 0;
        for slot in self.windows.iter() {
            if let Some(ref win) = slot {
                if win.bounds.contains(event.x, event.y) && win.z_order > current_max_z {
                    current_max_z = win.z_order;
                    target_id = Some(win.window_id);
                }
            }
        }

        if let Some(id) = target_id {
            self.focused_window_id = Some(id);
            Ok(id)
        } else {
            Err(KernelCompositorError::WindowNotFound)
        }
    }

    pub fn render_vsync(&mut self) -> usize {
        self.vsync_count += 1;
        let mut total_damaged = 0;
        for slot in self.windows.iter_mut() {
            if let Some(ref mut win) = slot {
                total_damaged += win.damage_count;
                win.damage = [None; KERNEL_MAX_DIRTY_RECTS];
                win.damage_count = 0;
            }
        }
        total_damaged
    }

    pub fn focused_window(&self) -> Option<u32> {
        self.focused_window_id
    }

    pub fn vsync_count(&self) -> u64 {
        self.vsync_count
    }
}
