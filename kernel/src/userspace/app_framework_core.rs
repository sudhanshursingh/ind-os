//! Kernel App Framework Verification Harness (SP-63, REQ-USE-207)
//!
//! Architectural Invariants:
//! - INV-APP-001: Plain-language manifest validation & capability bounds
//! - INV-APP-002: Sandboxed spawn binds exact declared capability mask
//! - INV-APP-003: User-mediated inter-app capability sharing

pub const KERNEL_MAX_SANDBOXED_APPS: usize = 8;
pub const KERNEL_DEFAULT_APP_MEMORY_BUDGET_KB: usize = 16384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAppFrameworkError {
    InvalidManifest,
    UndeclaredCapability,
    SandboxTableFull,
    AppNotFound,
    MediationRequired,
    UnconfirmedGrant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelAppManifest {
    pub app_id: &'static str,
    pub display_name: &'static str,
    pub declared_caps: u64,
    pub justification: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSandboxedApp {
    pub pid: u32,
    pub app_id: &'static str,
    pub active_caps: u64,
    pub memory_budget_kb: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelUserMediationGrant {
    pub grant_id: u32,
    pub source_pid: u32,
    pub target_pid: u32,
    pub granted_cap: u64,
    pub user_confirmed: bool,
}

pub struct KernelAppFrameworkEngine {
    sandboxes: [Option<KernelSandboxedApp>; KERNEL_MAX_SANDBOXED_APPS],
    app_count: usize,
    next_pid: u32,
    next_grant_id: u32,
}

impl KernelAppFrameworkEngine {
    pub const fn new() -> Self {
        Self {
            sandboxes: [None; KERNEL_MAX_SANDBOXED_APPS],
            app_count: 0,
            next_pid: 100,
            next_grant_id: 1,
        }
    }

    pub fn validate_manifest(&self, manifest: &KernelAppManifest) -> Result<(), KernelAppFrameworkError> {
        if manifest.app_id.is_empty() || manifest.display_name.is_empty() {
            return Err(KernelAppFrameworkError::InvalidManifest);
        }
        if manifest.declared_caps != 0 && manifest.justification.is_empty() {
            return Err(KernelAppFrameworkError::InvalidManifest);
        }
        Ok(())
    }

    pub fn spawn_app(
        &mut self,
        manifest: &KernelAppManifest,
        requested_caps: u64,
    ) -> Result<u32, KernelAppFrameworkError> {
        self.validate_manifest(manifest)?;

        if (requested_caps & !manifest.declared_caps) != 0 {
            return Err(KernelAppFrameworkError::UndeclaredCapability);
        }

        for slot in self.sandboxes.iter_mut() {
            if slot.is_none() {
                let pid = self.next_pid;
                self.next_pid += 1;
                *slot = Some(KernelSandboxedApp {
                    pid,
                    app_id: manifest.app_id,
                    active_caps: requested_caps,
                    memory_budget_kb: KERNEL_DEFAULT_APP_MEMORY_BUDGET_KB,
                });
                self.app_count += 1;
                return Ok(pid);
            }
        }
        Err(KernelAppFrameworkError::SandboxTableFull)
    }

    pub fn create_user_grant(
        &mut self,
        source_pid: u32,
        target_pid: u32,
        cap_to_share: u64,
    ) -> Result<KernelUserMediationGrant, KernelAppFrameworkError> {
        let source_app = self.find_app(source_pid).ok_or(KernelAppFrameworkError::AppNotFound)?;
        if (source_app.active_caps & cap_to_share) != cap_to_share {
            return Err(KernelAppFrameworkError::UndeclaredCapability);
        }
        self.find_app(target_pid).ok_or(KernelAppFrameworkError::AppNotFound)?;

        let grant_id = self.next_grant_id;
        self.next_grant_id += 1;

        Ok(KernelUserMediationGrant {
            grant_id,
            source_pid,
            target_pid,
            granted_cap: cap_to_share,
            user_confirmed: false,
        })
    }

    pub fn apply_user_grant(
        &mut self,
        grant: &mut KernelUserMediationGrant,
        user_confirmed: bool,
    ) -> Result<(), KernelAppFrameworkError> {
        if !user_confirmed {
            grant.user_confirmed = false;
            return Err(KernelAppFrameworkError::UnconfirmedGrant);
        }

        grant.user_confirmed = true;

        for slot in self.sandboxes.iter_mut() {
            if let Some(ref mut app) = slot {
                if app.pid == grant.target_pid {
                    app.active_caps |= grant.granted_cap;
                    return Ok(());
                }
            }
        }

        Err(KernelAppFrameworkError::AppNotFound)
    }

    pub fn find_app(&self, pid: u32) -> Option<&KernelSandboxedApp> {
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
