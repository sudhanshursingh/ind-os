//! App Framework, Plain-Language Manifests & Sandboxing (SP-63, REQ-USE-207)
//!
//! Architectural Invariants:
//! - INV-APP-001: Plain-language manifest validation & capability bounds
//! - INV-APP-002: Sandboxed spawn binds exact declared capability mask
//! - INV-APP-003: User-mediated inter-app capability sharing

pub const MAX_SANDBOXED_APPS: usize = 8;
pub const DEFAULT_APP_MEMORY_BUDGET_KB: usize = 16384; // 16MB default budget

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppFrameworkError {
    InvalidManifest,
    UndeclaredCapability,
    SandboxTableFull,
    AppNotFound,
    MediationRequired,
    UnconfirmedGrant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppManifest {
    pub app_id: &'static str,
    pub display_name: &'static str,
    pub declared_caps: u64,
    pub justification: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxedApp {
    pub pid: u32,
    pub app_id: &'static str,
    pub active_caps: u64,
    pub memory_budget_kb: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserMediationGrant {
    pub grant_id: u32,
    pub source_pid: u32,
    pub target_pid: u32,
    pub granted_cap: u64,
    pub user_confirmed: bool,
}

pub struct AppFrameworkEngine {
    sandboxes: [Option<SandboxedApp>; MAX_SANDBOXED_APPS],
    app_count: usize,
    next_pid: u32,
    next_grant_id: u32,
}

impl AppFrameworkEngine {
    pub const fn new() -> Self {
        Self {
            sandboxes: [None; MAX_SANDBOXED_APPS],
            app_count: 0,
            next_pid: 100,
            next_grant_id: 1,
        }
    }

    pub fn validate_manifest(&self, manifest: &AppManifest) -> Result<(), AppFrameworkError> {
        if manifest.app_id.is_empty() || manifest.display_name.is_empty() {
            return Err(AppFrameworkError::InvalidManifest);
        }
        // Manifest must have a human-readable justification for non-zero capabilities (REQ-USE-207)
        if manifest.declared_caps != 0 && manifest.justification.is_empty() {
            return Err(AppFrameworkError::InvalidManifest);
        }
        Ok(())
    }

    pub fn spawn_app(
        &mut self,
        manifest: &AppManifest,
        requested_caps: u64,
    ) -> Result<u32, AppFrameworkError> {
        self.validate_manifest(manifest)?;

        // Ensure app does not request capabilities outside declared manifest
        if (requested_caps & !manifest.declared_caps) != 0 {
            return Err(AppFrameworkError::UndeclaredCapability);
        }

        for slot in self.sandboxes.iter_mut() {
            if slot.is_none() {
                let pid = self.next_pid;
                self.next_pid += 1;
                *slot = Some(SandboxedApp {
                    pid,
                    app_id: manifest.app_id,
                    active_caps: requested_caps,
                    memory_budget_kb: DEFAULT_APP_MEMORY_BUDGET_KB,
                });
                self.app_count += 1;
                return Ok(pid);
            }
        }
        Err(AppFrameworkError::SandboxTableFull)
    }

    pub fn create_user_grant(
        &mut self,
        source_pid: u32,
        target_pid: u32,
        cap_to_share: u64,
    ) -> Result<UserMediationGrant, AppFrameworkError> {
        // Source app must exist and own the capability
        let source_app = self.find_app(source_pid).ok_or(AppFrameworkError::AppNotFound)?;
        if (source_app.active_caps & cap_to_share) != cap_to_share {
            return Err(AppFrameworkError::UndeclaredCapability);
        }
        // Target app must exist
        self.find_app(target_pid).ok_or(AppFrameworkError::AppNotFound)?;

        let grant_id = self.next_grant_id;
        self.next_grant_id += 1;

        Ok(UserMediationGrant {
            grant_id,
            source_pid,
            target_pid,
            granted_cap: cap_to_share,
            user_confirmed: false, // Must be explicitly confirmed by user
        })
    }

    pub fn apply_user_grant(
        &mut self,
        grant: &mut UserMediationGrant,
        user_confirmed: bool,
    ) -> Result<(), AppFrameworkError> {
        if !user_confirmed {
            grant.user_confirmed = false;
            return Err(AppFrameworkError::UnconfirmedGrant);
        }

        grant.user_confirmed = true;

        // Transfer capability to target app
        for slot in self.sandboxes.iter_mut() {
            if let Some(ref mut app) = slot {
                if app.pid == grant.target_pid {
                    app.active_caps |= grant.granted_cap;
                    return Ok(());
                }
            }
        }

        Err(AppFrameworkError::AppNotFound)
    }

    pub fn find_app(&self, pid: u32) -> Option<&SandboxedApp> {
        for slot in self.sandboxes.iter() {
            if let Some(ref app) = slot {
                if app.pid == pid {
                    return Some(app);
                }
            }
        }
        None
    }

    pub fn app_count(&self) -> usize {
        self.app_count
    }
}
