package indos.launcher.model

import android.graphics.drawable.Drawable

// ==============================================================================
// Ind OS — Launcher Data Models
// Lightweight, immutable data structures designed for zero memory overhead
// ==============================================================================

/**
 * Represents an installed launchable application in Ind OS.
 */
data class AppInfo(
    val label: String,
    val packageName: String,
    val activityName: String,
    val isSystemApp: Boolean = false,
    val icon: Drawable? = null
)

/**
 * Privacy & Security toggles displayed on the Ind Smart Contextual Card.
 */
data class PrivacyTogglesState(
    val isMicMuted: Boolean = false,
    val isCameraKilled: Boolean = false,
    val isAirGapActive: Boolean = true,
    val isSurakshaShieldActive: Boolean = true
)

/**
 * Upcoming Alarm or Morning briefing context.
 */
data class UpcomingAlarm(
    val timeString: String = "07:00 AM",
    val label: String = "Morning Briefing & News",
    val isEnabled: Boolean = true,
    val daysSummary: String = "Mon, Tue, Wed, Thu, Fri"
)

/**
 * Priority notification that bypassed spam filter (e.g. OTP, Bank, Direct Call/Chat).
 */
data class PriorityNotification(
    val id: String,
    val title: String,
    val content: String,
    val sourceApp: String,
    val timestamp: String,
    val isOtpOrFinancial: Boolean = false,
    val isSurakshaProtected: Boolean = true
)

/**
 * Intent matched from the Ind Command Bar.
 */
data class IntentActionMatch(
    val actionType: String,
    val target: String,
    val displayText: String,
    val confidence: Float = 1.0f
)

/**
 * Promotional spam aggregated into the Evening Notification Digest.
 */
data class PromoDigestCardData(
    val totalPromosCount: Int = 0,
    val sourceApps: List<String> = emptyList(),
    val headlineSummary: String = "",
    val scheduledDigestTime: String = "07:30 PM",
    val promoSnippets: List<String> = emptyList()
)

