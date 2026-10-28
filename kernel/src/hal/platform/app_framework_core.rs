//! Kernel App Framework Platform Bridge (SP-127, REQ-APP-COMMON-006)
//!
//! Standards: Bible §8.18, §8.20, §9.9.1; INV-APP-WIN-001
//! Rule N-01: Zero external crates
//! Rule N-04: Zero unsafe code in new modules

use crate::hal::platform::shell_hw_core::KERNEL_CAP_DISPLAY_RENDER;

pub const KERNEL_SYSTEM_RESERVED_Z_BASE: u8 = 200;
pub const MAX_KERNEL_MANAGED_WINDOWS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAppFrameworkError {
    PermissionDenied,
    InvalidZIndex,
    TableFull,
    WindowNotFound,
    InvalidPid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelWindowSurfaceDescriptor {
    pub window_id: u32,
    pub pid: u32,
    pub app_id: &'static str,
    pub active_caps: u64,
    pub z_index: u8,
    pub is_visible: bool,
}

pub struct KernelAppFrameworkBridge {
    windows: [Option<KernelWindowSurfaceDescriptor>; MAX_KERNEL_MANAGED_WINDOWS],
    active_count: usize,
    next_window_id: u32,
}

impl KernelAppFrameworkBridge {
    pub const fn new() -> Self {
        Self {
            windows: [None; MAX_KERNEL_MANAGED_WINDOWS],
            active_count: 0,
            next_window_id: 1,
        }
    }

    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Registers a window surface in kernel compositor state, enforcing capability & Z-plane bounds (INV-APP-WIN-001)
    pub fn register_window_surface(
        &mut self,
        pid: u32,
        app_id: &'static str,
        caps: u64,
        z_index: u8,
    ) -> Result<u32, KernelAppFrameworkError> {
        if pid == 0 {
            return Err(KernelAppFrameworkError::InvalidPid);
        }
        // Strict capability gate: CAP_DISPLAY_RENDER is mandatory
        if (caps & (KERNEL_CAP_DISPLAY_RENDER as u64)) == 0 {
            return Err(KernelAppFrameworkError::PermissionDenied);
        }
        // Applications cannot spoof system chrome or hardware consent planes
        if z_index >= KERNEL_SYSTEM_RESERVED_Z_BASE {
            return Err(KernelAppFrameworkError::InvalidZIndex);
        }

        for slot in self.windows.iter_mut() {
            if slot.is_none() {
                let window_id = self.next_window_id;
                self.next_window_id += 1;

                *slot = Some(KernelWindowSurfaceDescriptor {
                    window_id,
                    pid,
                    app_id,
                    active_caps: caps,
                    z_index,
                    is_visible: true,
                });
                self.active_count += 1;
                return Ok(window_id);
            }
        }

        Err(KernelAppFrameworkError::TableFull)
    }

    pub fn set_visibility(&mut self, window_id: u32, visible: bool) -> Result<(), KernelAppFrameworkError> {
        for slot in self.windows.iter_mut() {
            if let Some(ref mut win) = slot {
                if win.window_id == window_id {
                    win.is_visible = visible;
                    return Ok(());
                }
            }
        }
        Err(KernelAppFrameworkError::WindowNotFound)
    }

    pub fn unregister_window_surface(&mut self, window_id: u32) -> Result<(), KernelAppFrameworkError> {
        for slot in self.windows.iter_mut() {
            if let Some(ref win) = slot {
                if win.window_id == window_id {
                    *slot = None;
                    self.active_count -= 1;
                    return Ok(());
                }
            }
        }
        Err(KernelAppFrameworkError::WindowNotFound)
    }
}
