#![no_std]

//! Ind OS UX Shell & App Framework Core Library (Phase 8, SP-60..SP-69)
//!
//! Architectural Invariants:
//! - INV-COMP-001..004: Capability display gating, zero-heap damage tracking, anti-overlay consent protection
//! - Rule N-01: Pure in-tree Rust, zero external crates in TCB
//! - Rule N-04: Zero unsafe code in new modules

pub mod compositor;
pub use compositor::{
    CompositorEngine, CompositorError, DamageRegion, InputEvent, InputEventType, Rect,
    WindowSurface, CAP_DISPLAY_OVERLAY, CAP_DISPLAY_RENDER, MAX_DIRTY_RECTS, MAX_WINDOWS,
    SYSTEM_OVERLAY_Z_BASE,
};
pub mod shell_v2;
pub use shell_v2::{
    NotificationAction, NotificationItem, NotificationSource, ShellError, ShellV2Engine,
    INTENT_BUF_LEN, MAX_LAUNCHER_APPS, MAX_NOTIFICATIONS,
};
pub mod intent_ui;
pub use intent_ui::{
    ExplanationReceipt, IndicImeEngine, IntentError, IntentSession, IntentState, IntentUiEngine,
    ProposalCard, MAX_INTENT_STR_LEN,
};pub mod app_framework;
pub use app_framework::{
    AppFrameworkError, AppFrameworkEngine, AppManifest, SandboxedApp, UserMediationGrant,
    DEFAULT_APP_MEMORY_BUDGET_KB, MAX_SANDBOXED_APPS,
};pub mod accessibility;
pub use accessibility::{
    A11yError, A11yNode, A11yRole, AccessibilitySubsystem, MagnifierService, ScreenReaderEngine,
    SwitchAccessEngine, MAX_A11Y_NODES, MAX_ZOOM_PERCENT, MIN_ZOOM_PERCENT,
};
pub mod proactive_ui;
pub use proactive_ui::{
    AggressivenessDial, ProactiveError, ProactiveSuggestion, ProactiveUiEngine, MAX_SUGGESTIONS,
};
pub mod core_apps;
pub use core_apps::{
    CoreAppDescriptor, CoreAppError, CoreAppsSuite, CORE_APPS_COUNT,
};
pub mod perf_budget;
pub use perf_budget::{
    PerfBudgetAuditor, PerfBudgetError, PerfBudgetRecord, MAX_APP_MEMORY_KB, MAX_COLD_START_MS,
    MAX_FRAME_TIME_US,
};
pub mod ux_validation;
pub use ux_validation::{
    PersonaTaskResult, UxValidationError, UxValidationHarness, MAX_PERSONA_TASKS,
};
pub mod polish;
pub use polish::{
    AdversarialUiHarness, PolishSecurityError, EXPECTED_CONSENT_BROKER_MAGIC,
};
pub mod shell_hw;
pub use shell_hw::{
    GestureEvent, GestureType, ShellHwEngine, ShellHwError, ShellHwState, TouchPoint,
    CAP_TOUCH_INPUT, MAIN_SURFACE_HEIGHT, NAV_BAR_HEIGHT, PANEL_HEIGHT,
    PANEL_WIDTH, REFRESH_RATE_HZ, STATUS_BAR_HEIGHT,
};
pub mod settings_power;
pub use settings_power::{
    BatteryTelemetry, ChargingStatus, PowerGovernorState, PowerProfile, SettingsPowerEngine,
    SettingsPowerError, BATTERY_EMPTY_MV, BATTERY_FULL_MV, CAP_POWER_MGMT, DISPLAY_RATE_ECO_HZ,
    DISPLAY_RATE_PERF_HZ, THERMAL_CRITICAL_MC, THERMAL_MILD_MC, THERMAL_SEVERE_MC,
};

pub mod perf_profile;
pub use perf_profile::{
    PerfHotspot, PerfMetricSample, PerfProfileError, PerfProfileHarness, CAP_PERF_PROFILE,
    MAX_PERF_HOTSPOTS,
};
pub mod touch_opt;
pub use touch_opt::{
    FastTouchPoint, TouchOptEngine, TouchOptError, CAP_TOUCH_OPT, TOUCH_OPT_MAX_BATCH,
};
pub mod storage_opt;
pub use storage_opt::{
    CachedBlock, StorageOptError, StorageOptHarness, CAP_STORAGE_OPT, STORAGE_PAGE_CACHE_SIZE,
};
pub mod compositor_opt;
pub use compositor_opt::{
    CompositorOptEngine, CompositorOptError, FrameTimingSample, CAP_COMPOSITOR_OPT,
    TARGET_FRAME_TIME_US,
};
pub mod interactive_shell;
pub use interactive_shell::{
    ActiveCard, AppAction, InteractiveShellEngine, InteractiveShellError, CAP_INTERACTIVE_SHELL,
};

