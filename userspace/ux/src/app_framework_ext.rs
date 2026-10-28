//! App Framework Extensions: Lifecycle, Window Management, Back-Stack & Chrome (SP-127, REQ-APP-COMMON-006)
//!
//! Standards: Bible §8.18, §8.20, §9.9.1; Universal App Charter
//! Invariants:
//! - INV-APP-LIFE-001: Strict six-state lifecycle machine
//! - INV-APP-STACK-001: Deterministic LIFO back-stack with Bento home fallback
//! - INV-APP-CHROME-001: Universal glassmorphic chrome with back, title, action tray
//! - INV-APP-WIN-001: Capability-bound window descriptors & z-plane isolation
//! Rule N-01: Zero external crates
//! Rule N-04: Zero unsafe code in userspace

pub const MAX_BACK_STACK_DEPTH: usize = 16;
pub const MAX_ACTIVE_WINDOWS: usize = 8;
pub const CAP_DISPLAY_RENDER: u64 = 0x0000_0000_0000_0001;
pub const CAP_SYSTEM_CHROME: u64  = 0x0000_0000_0000_0002;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLifecycleState {
    Unloaded,
    Created,
    Foreground,
    Background,
    Suspended,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLifecycleEvent {
    Launch,
    Foreground,
    Background,
    Suspend,
    Resume,
    Terminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLifecycleError {
    InvalidTransition,
    StackOverflow,
    StackEmpty,
    WindowNotFound,
    PermissionDenied,
    TableFull,
    InvalidPid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppChrome {
    pub title: &'static str,
    pub back_enabled: bool,
    pub badge: Option<&'static str>,
    pub action_count: usize,
    pub actions: [Option<&'static str>; 3],
}

impl AppChrome {
    pub const fn new(title: &'static str, back_enabled: bool) -> Self {
        Self {
            title,
            back_enabled,
            badge: None,
            action_count: 0,
            actions: [None, None, None],
        }
    }

    pub fn with_badge(mut self, badge: &'static str) -> Self {
        self.badge = Some(badge);
        self
    }

    pub fn with_action(mut self, action_name: &'static str) -> Self {
        if self.action_count < 3 {
            self.actions[self.action_count] = Some(action_name);
            self.action_count += 1;
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppWindowContext {
    pub window_id: u32,
    pub pid: u32,
    pub app_id: &'static str,
    pub chrome: AppChrome,
    pub state: AppLifecycleState,
    pub active_caps: u64,
    pub z_index: u8,
}

pub struct AppBackStack {
    entries: [u32; MAX_BACK_STACK_DEPTH],
    depth: usize,
}

impl AppBackStack {
    pub const fn new() -> Self {
        Self {
            entries: [0; MAX_BACK_STACK_DEPTH],
            depth: 0,
        }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn is_empty(&self) -> bool {
        self.depth == 0
    }

    pub fn peek(&self) -> Option<u32> {
        if self.depth == 0 {
            None
        } else {
            Some(self.entries[self.depth - 1])
        }
    }

    pub fn push(&mut self, window_id: u32) -> Result<(), AppLifecycleError> {
        if self.depth >= MAX_BACK_STACK_DEPTH {
            return Err(AppLifecycleError::StackOverflow);
        }
        self.entries[self.depth] = window_id;
        self.depth += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<u32, AppLifecycleError> {
        if self.depth == 0 {
            return Err(AppLifecycleError::StackEmpty);
        }
        self.depth -= 1;
        let window_id = self.entries[self.depth];
        self.entries[self.depth] = 0;
        Ok(window_id)
    }

    pub fn clear(&mut self) {
        self.entries = [0; MAX_BACK_STACK_DEPTH];
        self.depth = 0;
    }
}

pub struct AppFrameworkExtEngine {
    windows: [Option<AppWindowContext>; MAX_ACTIVE_WINDOWS],
    window_count: usize,
    back_stack: AppBackStack,
    next_window_id: u32,
    active_window_id: Option<u32>,
}

impl AppFrameworkExtEngine {
    pub const fn new() -> Self {
        Self {
            windows: [None; MAX_ACTIVE_WINDOWS],
            window_count: 0,
            back_stack: AppBackStack::new(),
            next_window_id: 1,
            active_window_id: None,
        }
    }

    pub fn active_window_id(&self) -> Option<u32> {
        self.active_window_id
    }

    pub fn back_stack_depth(&self) -> usize {
        self.back_stack.depth()
    }

    pub fn window_count(&self) -> usize {
        self.window_count
    }

    /// Validates and applies strict six-state lifecycle state machine transitions (INV-APP-LIFE-001)
    pub fn transition_state(
        current: AppLifecycleState,
        event: AppLifecycleEvent,
    ) -> Result<AppLifecycleState, AppLifecycleError> {
        match (current, event) {
            (AppLifecycleState::Unloaded, AppLifecycleEvent::Launch) => Ok(AppLifecycleState::Created),
            (AppLifecycleState::Created, AppLifecycleEvent::Foreground) => Ok(AppLifecycleState::Foreground),
            (AppLifecycleState::Background, AppLifecycleEvent::Foreground) => Ok(AppLifecycleState::Foreground),
            (AppLifecycleState::Foreground, AppLifecycleEvent::Background) => Ok(AppLifecycleState::Background),
            (AppLifecycleState::Background, AppLifecycleEvent::Suspend) => Ok(AppLifecycleState::Suspended),
            (AppLifecycleState::Suspended, AppLifecycleEvent::Resume) => Ok(AppLifecycleState::Background),
            (AppLifecycleState::Created, AppLifecycleEvent::Terminate) => Ok(AppLifecycleState::Terminated),
            (AppLifecycleState::Background, AppLifecycleEvent::Terminate) => Ok(AppLifecycleState::Terminated),
            (AppLifecycleState::Suspended, AppLifecycleEvent::Terminate) => Ok(AppLifecycleState::Terminated),
            _ => Err(AppLifecycleError::InvalidTransition),
        }
    }

    /// Spawns or presents a window surface with back-stack push (INV-APP-STACK-001, INV-APP-WIN-001)
    pub fn launch_app_window(
        &mut self,
        pid: u32,
        app_id: &'static str,
        title: &'static str,
        active_caps: u64,
    ) -> Result<u32, AppLifecycleError> {
        if pid == 0 {
            return Err(AppLifecycleError::InvalidPid);
        }
        // Verify caller possesses display capability (INV-APP-WIN-001)
        if (active_caps & CAP_DISPLAY_RENDER) == 0 {
            return Err(AppLifecycleError::PermissionDenied);
        }

        // Background current active window if any
        if let Some(prev_id) = self.active_window_id {
            self.send_lifecycle_event(prev_id, AppLifecycleEvent::Background)?;
        }

        let window_id = self.next_window_id;
        self.next_window_id += 1;

        let has_back = self.back_stack.depth() > 0;
        let chrome = AppChrome::new(title, has_back);

        let window = AppWindowContext {
            window_id,
            pid,
            app_id,
            chrome,
            state: AppLifecycleState::Created,
            active_caps,
            z_index: 10,
        };

        for slot in self.windows.iter_mut() {
            if slot.is_none() {
                *slot = Some(window);
                self.window_count += 1;
                self.back_stack.push(window_id)?;
                self.active_window_id = Some(window_id);

                // Transition Created -> Foreground
                self.send_lifecycle_event(window_id, AppLifecycleEvent::Foreground)?;
                return Ok(window_id);
            }
        }

        Err(AppLifecycleError::TableFull)
    }

    /// Navigates back down the LIFO stack; returns None when stack becomes empty to fall back to Bento Home (INV-APP-STACK-001)
    pub fn navigate_back(&mut self) -> Result<Option<u32>, AppLifecycleError> {
        if self.back_stack.is_empty() {
            self.active_window_id = None;
            return Ok(None);
        }

        let closing_id = self.back_stack.pop()?;
        self.close_window(closing_id)?;

        if let Some(top_id) = self.back_stack.peek() {
            self.active_window_id = Some(top_id);
            self.send_lifecycle_event(top_id, AppLifecycleEvent::Foreground)?;
            Ok(Some(top_id))
        } else {
            // Stack completely empty -> Return cleanly to Bento Home canvas
            self.active_window_id = None;
            Ok(None)
        }
    }

    pub fn send_lifecycle_event(
        &mut self,
        window_id: u32,
        event: AppLifecycleEvent,
    ) -> Result<AppLifecycleState, AppLifecycleError> {
        for slot in self.windows.iter_mut() {
            if let Some(ref mut win) = slot {
                if win.window_id == window_id {
                    let next = Self::transition_state(win.state, event)?;
                    win.state = next;
                    return Ok(next);
                }
            }
        }
        Err(AppLifecycleError::WindowNotFound)
    }

    pub fn get_window(&self, window_id: u32) -> Option<&AppWindowContext> {
        for slot in self.windows.iter() {
            if let Some(ref win) = slot {
                if win.window_id == window_id {
                    return Some(win);
                }
            }
        }
        None
    }

    pub fn close_window(&mut self, window_id: u32) -> Result<(), AppLifecycleError> {
        for slot in self.windows.iter_mut() {
            if let Some(ref mut win) = slot {
                if win.window_id == window_id {
                    let _ = Self::transition_state(win.state, AppLifecycleEvent::Terminate);
                    *slot = None;
                    self.window_count -= 1;
                    return Ok(());
                }
            }
        }
        Err(AppLifecycleError::WindowNotFound)
    }
}
