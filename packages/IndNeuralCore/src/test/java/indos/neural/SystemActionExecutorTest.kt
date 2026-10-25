package indos.neural

import android.content.Context
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.lang.reflect.Proxy

/**
 * Sprint 6 DoD Verification Test Suite:
 * Validates Core OS System Intent Execution:
 * - SYS_TOGGLE (Wi-Fi, Bluetooth, DND, Torch, Brightness, Volume)
 * - OPEN_APP (Fuzzy spoken-name to package resolution)
 * - PLACE_CALL (Telephony)
 * - SET_ALARM (AlarmClock)
 *
 * DoD Requirement: Each action executes in < 500 ms end-to-end.
 */
class SystemActionExecutorTest {

    private lateinit var mockContext: Context
    private lateinit var executor: SystemActionExecutor
    private lateinit var appResolver: AppIndexResolver

    private open class FakeContext : android.content.ContextWrapper(null) {
        override fun getApplicationContext(): Context = this
        override fun getPackageName(): String = "in.indos.neural"
        override fun getPackageManager(): android.content.pm.PackageManager? = null
        override fun getSystemService(name: String): Any? = null
        override fun getContentResolver(): android.content.ContentResolver? = null
        override fun startActivity(intent: android.content.Intent?) {}
    }

    @Before
    fun setUp() {
        mockContext = FakeContext()
        executor = SystemActionExecutor(mockContext)
        appResolver = AppIndexResolver(mockContext)
    }

    @Test
    fun testAppIndexResolverExactAndFuzzyResolution() {
        val testCases = mapOf(
            "whatsapp" to "com.whatsapp",
            "व्हाट्सएप" to "com.whatsapp",
            "youtube" to "com.google.android.youtube",
            "यूट्यूब" to "com.google.android.youtube",
            "chrome" to "com.android.chrome",
            "browser" to "com.android.chrome",
            "camera" to "com.android.camera2",
            "settings" to "com.android.settings",
            "phone" to "com.android.dialer",
            "clock" to "com.google.android.deskclock",
            "alarm" to "com.google.android.deskclock",
            "calculator" to "com.google.android.calculator",
            // Fuzzy typos (distance <= 2)
            "watsap" to "com.whatsapp",
            "youtub" to "com.google.android.youtube",
            "camra" to "com.android.camera2"
        )

        for ((spokenName, expectedPkg) in testCases) {
            val startNs = System.nanoTime()
            val resolved = appResolver.resolvePackage(spokenName)
            val elapsedMs = (System.nanoTime() - startNs) / 1_000_000.0

            assertNotNull("Failed to resolve '$spokenName'", resolved)
            assertEquals("Spoken name '$spokenName' should resolve to $expectedPkg", expectedPkg, resolved)
            assertTrue("App resolution must be < 15 ms (was $elapsedMs ms)", elapsedMs < 15.0)
        }
    }

    @Test
    fun testSysTogglesExecuteUnder500ms() {
        val toggles = listOf(
            Triple("flashlight", mapOf("value" to true), "Torch ON"),
            Triple("flashlight", mapOf("value" to false), "Torch OFF"),
            Triple("wifi", mapOf("value" to true), "Wi-Fi ON"),
            Triple("wifi", mapOf("value" to false), "Wi-Fi OFF"),
            Triple("bluetooth", mapOf("value" to true), "Bluetooth ON"),
            Triple("bluetooth", mapOf("value" to false), "Bluetooth OFF"),
            Triple("volume", mapOf("value" to 80), "Volume 80%"),
            Triple("volume", mapOf("value" to 0), "Volume Mute"),
            Triple("brightness", mapOf("value" to 70), "Brightness 70%"),
            Triple("dnd", mapOf("value" to true), "DND ON"),
            Triple("airplane_mode", mapOf("value" to false), "Airplane mode OFF")
        )

        for ((target, params, label) in toggles) {
            val result = executor.executeAction("SYS_TOGGLE", target, params)
            assertTrue("Action $label must succeed", result.success)
            assertTrue(
                "DoD Requirement: Action $label must execute in < 500 ms (was ${result.durationMs} ms)",
                result.durationMs < 500.0
            )
        }
    }

    @Test
    fun testOpenAppExecutesUnder500ms() {
        val apps = listOf("whatsapp", "youtube", "chrome", "camera", "settings", "phone", "calculator")

        for (app in apps) {
            val result = executor.executeAction("OPEN_APP", app)
            assertTrue("Launch of $app must succeed", result.success)
            assertTrue(
                "DoD Requirement: Launching $app must execute in < 500 ms (was %.2f ms)".format(result.durationMs),
                result.durationMs < 500.0
            )
        }
    }

    @Test
    fun testPlaceCallAndSetAlarmExecuteUnder500ms() {
        // Telephony call
        val callResult = executor.executeAction("PLACE_CALL", "112")
        assertTrue("Place call must succeed", callResult.success)
        assertTrue("DoD Requirement: Call must execute in < 500 ms", callResult.durationMs < 500.0)

        // AlarmClock intent
        val alarmResult = executor.executeAction("SET_ALARM", "alarm", mapOf("hour" to 7, "minutes" to 30))
        assertTrue("Set alarm must succeed", alarmResult.success)
        assertTrue("DoD Requirement: Set alarm must execute in < 500 ms", alarmResult.durationMs < 500.0)
    }

    @Test
    fun testCompoundEndToEndActionExecutionPipeline() {
        // Flagship compound prompt from PRD §7 Sprint 5/6
        val prompt = "Torch chalu karo aur volume 80 percent kar do"
        val actionJson = PromptCompiler.compileAction(prompt)

        assertNotNull("Compiled JSON must not be null", actionJson)
        assertTrue("Must adhere to GBNF schema", PromptCompiler.validateActionJson(actionJson))

        val results = executor.executeActionJson(actionJson)
        assertEquals("Must execute exactly 2 actions (Torch + Volume)", 2, results.size)

        for (res in results) {
            assertTrue("Action ${res.actionType} on ${res.target} must succeed", res.success)
            assertTrue(
                "Action ${res.actionType} on ${res.target} must execute in < 500 ms (was %.2f ms)".format(res.durationMs),
                res.durationMs < 500.0
            )
        }
    }
}
