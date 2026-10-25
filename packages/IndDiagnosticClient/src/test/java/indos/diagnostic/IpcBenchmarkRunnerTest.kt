package indos.diagnostic

import indos.neural.ipc.IIndAIManager
import indos.neural.ipc.IIndStreamCallback
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Unit test verifying IPC benchmark calculations, statistics,
 * and DoD criteria evaluation.
 */
class IpcBenchmarkRunnerTest {

    private val mockAiManager = object : IIndAIManager.Stub() {
        override fun classifyFast(payload: String?, domain: String?): String {
            return """{"riskScore":0.0,"verdict":"PASS","reason":"Test"}"""
        }

        override fun executeZeroTouchCommand(
            userUtterance: String?,
            screenGraphJson: String?,
            callback: IIndStreamCallback?
        ) {
            callback?.onToken("token")
            callback?.onActionResolved("""{"tier":"GREEN","actions":[],"tts":"Done"}""")
        }

        override fun evictModelWeights() {}

        override fun getEngineStatusJson(): String {
            return """{"engine":"IndNeuralCore-Test","residentRamMB":18}"""
        }
    }

    @Test
    fun testBenchmarkExecutes1000CallsAndMeetsLatencyTarget() {
        val result = IpcBenchmarkRunner.runBenchmark(
            manager = mockAiManager,
            clientName = "UnitTestClient",
            iterations = 1000
        )

        assertEquals("Total calls must be 1,000", 1000, result.totalCalls)
        assertTrue("Min latency must be >= 0", result.minMs >= 0.0)
        assertTrue("Max latency must be >= min latency", result.maxMs >= result.minMs)
        assertTrue("Median latency must be <= max latency", result.medianMs <= result.maxMs)
        assertTrue("P95 must be <= P99", result.p95Ms <= result.p99Ms)
        assertTrue(
            "In-process mock median latency (%.3f ms) must easily pass DoD target < 2.5 ms".format(result.medianMs),
            result.targetMet
        )
        assertNotNull("Engine status must not be null", result.engineStatus)
    }
}
