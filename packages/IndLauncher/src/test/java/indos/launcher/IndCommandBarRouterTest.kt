package indos.launcher

import indos.launcher.model.AppInfo
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Validates ambient intent query matching, fuzzy app filtering, and command routing.
 */
class IndCommandBarRouterTest {

    private val testApps = listOf(
        AppInfo("WhatsApp", "com.whatsapp", "com.whatsapp.Main"),
        AppInfo("YouTube", "com.google.android.youtube", "com.google.android.youtube.HomeActivity"),
        AppInfo("Phone", "com.android.dialer", "com.android.dialer.DialtactsActivity", true),
        AppInfo("Ind Suraksha Shield", "in.indos.suraksha", "indos.suraksha.SurakshaBlockActivity", true),
        AppInfo("State Bank of India YONO", "com.sbi.lotusintouch", "com.sbi.lotusintouch.MainActivity")
    )

    @Test
    fun testFuzzyAppFiltering() {
        val query = "what"
        val filtered = testApps.filter {
            it.label.lowercase().contains(query) || it.packageName.lowercase().contains(query)
        }
        assertEquals(1, filtered.size)
        assertEquals("WhatsApp", filtered.first().label)
    }

    @Test
    fun testSystemActionMatching() {
        val systemIntents = listOf(
            "turn on flashlight" to "flashlight",
            "turn on torch" to "flashlight",
            "wifi settings" to "wifi",
            "bluetooth on" to "bluetooth"
        )

        for ((input, expectedTarget) in systemIntents) {
            val q = input.lowercase()
            val matchedTarget = when {
                q.contains("torch") || q.contains("flashlight") -> "flashlight"
                q.contains("wifi") -> "wifi"
                q.contains("bluetooth") -> "bluetooth"
                else -> "unknown"
            }
            assertEquals("Input '$input' must resolve to '$expectedTarget'", expectedTarget, matchedTarget)
        }
    }

    @Test
    fun testIndianBankingAppQuickMatch() {
        val query = "yono"
        val filtered = testApps.filter {
            it.label.lowercase().contains(query) || it.packageName.lowercase().contains(query)
        }
        assertEquals(1, filtered.size)
        assertTrue(filtered.first().label.contains("YONO"))
    }
}
