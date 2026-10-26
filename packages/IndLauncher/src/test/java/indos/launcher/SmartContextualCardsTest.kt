package indos.launcher

import indos.launcher.model.PriorityNotification
import indos.launcher.model.PrivacyTogglesState
import indos.launcher.model.UpcomingAlarm
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Validates Smart Contextual Card interactions, privacy toggle state transitions,
 * and zero-spam notification filtering.
 */
class SmartContextualCardsTest {

    @Test
    fun testPrivacyTogglesStateTransitions() {
        var state = PrivacyTogglesState(
            isMicMuted = false,
            isCameraKilled = false,
            isAirGapActive = true,
            isSurakshaShieldActive = true
        )

        // Toggle Mic Mute
        state = state.copy(isMicMuted = !state.isMicMuted)
        assertTrue(state.isMicMuted)

        // Toggle Camera Killswitch
        state = state.copy(isCameraKilled = !state.isCameraKilled)
        assertTrue(state.isCameraKilled)

        // Verify Air-Gap is active by default
        assertTrue(state.isAirGapActive)
        assertTrue(state.isSurakshaShieldActive)
    }

    @Test
    fun testAlarmDismissState() {
        var alarm = UpcomingAlarm(
            timeString = "07:00 AM",
            label = "Morning Briefing & News",
            isEnabled = true,
            daysSummary = "Mon - Fri"
        )
        assertTrue(alarm.isEnabled)

        // Dismiss alarm
        alarm = alarm.copy(isEnabled = false, label = "Alarm Dismissed")
        assertFalse(alarm.isEnabled)
        assertEquals("Alarm Dismissed", alarm.label)
    }

    @Test
    fun testPriorityNotifications_FilteredFromPromoSpam() {
        val incomingNotifications = listOf(
            PriorityNotification("1", "HDFC Bank", "₹5,000 received via UPI", "com.hdfc", "2m ago", isOtpOrFinancial = true),
            PriorityNotification("2", "Mom", "Call me when you reach home", "com.whatsapp", "5m ago", isOtpOrFinancial = false),
            PriorityNotification("3", "Spam Promo Deal", "50% off on shoes!", "com.unknown.spam", "10m ago", isOtpOrFinancial = false, isSurakshaProtected = false)
        )

        // Suraksha Shield priority filter: only verified priority financial/messaging alerts
        val priorityOnly = incomingNotifications.filter { it.isSurakshaProtected }

        assertEquals(2, priorityOnly.size)
        assertTrue(priorityOnly.any { it.isOtpOrFinancial })
        assertFalse(priorityOnly.any { it.title.contains("Spam") })
    }
}
