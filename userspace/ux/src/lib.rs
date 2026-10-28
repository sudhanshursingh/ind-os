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
};
pub mod app_framework_ext;
pub use app_framework_ext::{
    AppBackStack, AppChrome, AppFrameworkExtEngine, AppLifecycleError, AppLifecycleEvent,
    AppLifecycleState, AppWindowContext, MAX_ACTIVE_WINDOWS, MAX_BACK_STACK_DEPTH,
};
pub mod accessibility;
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

pub mod formal_delegation;
pub use formal_delegation::{
    FormalCapType, FormalCapability, FormalDelegationError, FormalDelegationValidator,
};

pub mod formal_revocation;
pub use formal_revocation::{
    FormalRevocationModel, RevocableCapNode, RevocationError, MAX_TREE_CAPACITIES,
};

pub mod formal_proof_ci;
pub use formal_proof_ci::{
    FormalProofReport, FormalProofSolver, FORMAL_PROOF_CHECKSUM, FORMAL_PROOF_LATTICE_STEPS,
};

pub mod external_audit;
pub use external_audit::{
    AuditFinding, AuditSeverity, ExternalAuditTracker, FindingStatus, TOTAL_AUDIT_FINDINGS,
};

pub mod redteam_campaign;
pub use redteam_campaign::{
    AttackTestResult, AttackVectorCategory, RedTeamHarness, TOTAL_ATTACK_VECTORS,
};

pub mod production_ops;
pub use production_ops::{
    ProductionOpsUpdateManager, SlotState, UpdateError, EXPECTED_SIGNING_KEY_ID,
    VALID_PAYLOAD_CHECKSUM,
};

pub mod beta_telemetry;
pub use beta_telemetry::{
    BetaTelemetryEngine, LocalTelemetryEvent, TelemetryConsentStatus, TelemetryError,
    MAX_LOCAL_EVENTS,
};

pub mod share_sheet;
pub use share_sheet::{
    BuiltinTarget, MimeType, PendingShare, ShareError, SharePayload, ShareSheetEngine, ShareTarget,
    CAP_SHARE_RECEIVE, CAP_SHARE_SEND, MAX_INLINE_PAYLOAD, MAX_MIME_LEN,
    MAX_SHARE_SESSIONS, MAX_SHARE_TARGETS,
};

pub mod intent_registry;
pub use intent_registry::{
    Intent, IntentHandler, IntentRegistry, IntentRegistryError, IntentType, PendingIntent,
    MAX_INTENT_HANDLERS, MAX_INTENT_URI_LEN, MAX_PENDING_INTENTS,
};

pub mod clipboard;
pub use clipboard::{
    ClipReadRequest, ClipSensitivity, ClipboardEntry, ClipboardError, ClipboardStore,
    CAP_CLIPBOARD_READ, CAP_CLIPBOARD_WRITE, MAX_CLIP_PAYLOAD, MAX_CLIP_READ_REQUESTS,
    MAX_CLIPBOARD_ENTRIES, SENSITIVE_TIMEOUT_TICKS,
};

pub mod global_search;
pub use global_search::{
    GlobalSearchEngine, SearchEntry, SearchError, SearchOptIn, SearchResult, SearchSourceKind,
    CAP_SEARCH_INDEX, CAP_SEARCH_QUERY, MAX_INDEX_ENTRIES, MAX_OPT_INS, MAX_QUERY_RESULTS,
    MAX_TERM_LEN,
};

pub mod lock_screen;
pub use lock_screen::{
    AisKillSwitchState, ConsentCard, ConsentVerdict, LockClock, LockScreenEngine, LockScreenError,
    LockState, MediaPlayState, MediaSession, PlaneId, MAX_CONSENT_CARDS,
};

pub mod notification_center;
pub use notification_center::{
    AppGroup, ConsentReceipt, DndState, NcNotification, NotifCenterError, NotifPriority,
    NotificationCenterEngine, ShadeState, CAP_NOTIFY_POST, CAP_NOTIFY_READ, MAX_APP_GROUPS,
    MAX_CONSENT_RECEIPTS, MAX_NC_NOTIFICATIONS, MAX_NOTIF_BODY_LEN, MAX_NOTIF_TITLE_LEN,
};

pub mod quick_settings;
pub use quick_settings::{
    QsAisMode, QsError, QsExpansionState, QsPowerMode, QsToggleId, QuickSettingsEngine,
    QuickSettingsState, CAP_QUICK_SETTINGS_READ, CAP_QUICK_SETTINGS_WRITE, SENSOR_CAM_ACTIVE,
    SENSOR_LOC_ACTIVE, SENSOR_MIC_ACTIVE,
};

pub mod recents;
pub use recents::{
    RecentsCarouselEngine, RecentsError, RecentsTask, ThumbnailDescriptor, CAP_RECENTS_MANAGE,
    MAX_RECENTS_TASKS,
};

pub mod notification_discipline;
pub use notification_discipline::{
    AisSystemState, DisciplineError, KillSwitchOrigin, NotificationDisciplineEngine,
    NotificationRule, NotificationVerdict, CAP_NOTIFY_RULE_WRITE, MAX_NOTIFICATION_RULES,
};

pub mod phone_call;
pub use phone_call::{
    AudioRoute, CallState, InCallSession, PhoneCallEngine, PhoneCallError, CAP_APP_PHONE_CALLS,
    EMERGENCY_NUMBERS, MAX_DTMF_BUFFER, MAX_PHONE_NUMBER_LEN,
};

pub mod phone_mgmt;
pub use phone_mgmt::{
    CallDirection, CallLogRecord, CallProposalCard, DualSimConfig, PhoneMgmtEngine,
    PhoneMgmtError, ScamShieldVerdict, SimSlot, SpeedDialEntry, CAP_PHONE_HISTORY_READ,
    CAP_PHONE_HISTORY_WRITE, MAX_BLOCKLIST_ENTRIES, MAX_CALL_LOG_ENTRIES, MAX_SPEED_DIAL_SLOTS,
};

pub mod phone_audio;
pub use phone_audio::{
    CellularAudioEngine, CellularPcmStats, DspFilterConfig, PcmCodecFormat, PhoneAudioError,
    StressTestReport, VoicemailConfig, VoicemailMwi, DEFAULT_VOICEMAIL_NUMBER,
    MAX_VOICEMAIL_NUM_LEN,
};

pub mod messages_core;
pub use messages_core::{
    ConversationThread, DeliveryStatus, MessageDirection, MessagesCoreError, MessagesEngine,
    SmsMessage, CAP_APP_SMS_RECEIVE, CAP_APP_SMS_SEND, MAX_ADDRESS_LEN, MAX_MESSAGES_PER_THREAD,
    MAX_RETRY_ATTEMPTS, MAX_SMS_BODY_LEN, MAX_THREADS,
};

pub mod messages_suraksha;
pub use messages_suraksha::{
    AttachmentDescriptor, DeviceCipherEnvelope, MessagesSurakshaEngine, MessagesSurakshaError,
    OtpDescriptor, SmsScamVerdict, CAP_ATTACHMENT_MEDIATE, MAX_ENVELOPE_LEN,
    SCREEN_SHARE_BLACKOUT_STR,
};

pub mod boot_animation;
pub use boot_animation::{
    BootAnimStage, BootAnimationEngine, MascotTransform, COLOR_ANIM_BG, COLOR_CYAN_CIRCUIT,
    COLOR_GOLD_MANDALA, COLOR_ORANGE_BRAND as ANIM_ORANGE_BRAND, COLOR_WHITE_TEXT as ANIM_WHITE_TEXT,
    TOTAL_ANIM_DURATION_MS,
};


