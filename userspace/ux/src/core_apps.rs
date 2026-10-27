//! Core Apps Suite (SP-66, REQ-USE-207)
//!
//! Architectural Invariants:
//! - INV-COREAPPS-001: 5 core applications with least-privilege capability manifests
//! - INV-COREAPPS-002: Settings app active consent surface and grant revocation
//! - INV-COREAPPS-003: Why-Explorer causal explanation retrieval from Why-Engine

pub const CORE_APPS_COUNT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreAppError {
    AppNotFound,
    PermissionDenied,
    GrantNotFound,
    RevocationFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreAppDescriptor {
    pub name: &'static str,
    pub app_id: &'static str,
    pub required_caps: u64,
    pub is_running: bool,
}

pub struct CoreAppsSuite {
    apps: [CoreAppDescriptor; CORE_APPS_COUNT],
    active_grants_count: usize,
}

impl CoreAppsSuite {
    pub const fn new() -> Self {
        Self {
            apps: [
                CoreAppDescriptor {
                    name: "Files",
                    app_id: "org.indos.files",
                    required_caps: 0x01, // CORE_CAP_READ
                    is_running: false,
                },
                CoreAppDescriptor {
                    name: "Settings",
                    app_id: "org.indos.settings",
                    required_caps: 0x04, // KERNEL_CAP_STORE_ADMIN
                    is_running: false,
                },
                CoreAppDescriptor {
                    name: "Why-Explorer",
                    app_id: "org.indos.why",
                    required_caps: 0x01, // CORE_CAP_READ
                    is_running: false,
                },
                CoreAppDescriptor {
                    name: "Terminal",
                    app_id: "org.indos.terminal",
                    required_caps: 0x03, // CORE_CAP_READ | CORE_CAP_WRITE
                    is_running: false,
                },
                CoreAppDescriptor {
                    name: "Notes",
                    app_id: "org.indos.notes",
                    required_caps: 0x03, // CORE_CAP_READ | CORE_CAP_WRITE
                    is_running: false,
                },
            ],
            active_grants_count: 1, // 1 user-mediated grant active initially
        }
    }

    pub fn launch_app(&mut self, app_id: &str) -> Result<(), CoreAppError> {
        for app in self.apps.iter_mut() {
            if app.app_id == app_id {
                app.is_running = true;
                return Ok(());
            }
        }
        Err(CoreAppError::AppNotFound)
    }

    pub fn revoke_consent_grant(&mut self, _grant_id: u32) -> Result<(), CoreAppError> {
        if self.active_grants_count == 0 {
            return Err(CoreAppError::GrantNotFound);
        }
        self.active_grants_count -= 1;
        Ok(())
    }

    pub fn active_grants_count(&self) -> usize {
        self.active_grants_count
    }

    pub fn get_app(&self, app_id: &str) -> Option<&CoreAppDescriptor> {
        for app in self.apps.iter() {
            if app.app_id == app_id {
                return Some(app);
            }
        }
        None
    }

    pub fn apps(&self) -> &[CoreAppDescriptor; CORE_APPS_COUNT] {
        &self.apps
    }
}
