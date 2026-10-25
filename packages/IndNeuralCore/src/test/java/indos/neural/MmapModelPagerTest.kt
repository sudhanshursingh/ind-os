package indos.neural

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/**
 * Unit test verifying Sprint 4:
 * - Zero-copy mmap model loader
 * - Instant madvise(MADV_DONTNEED) page eviction (<15 ms, >900 MB freed)
 * - Streaming JNI inference callback token dispatch (>= 15 tok/s)
 * - Mathematical Invariant 1 enforcement
 */
class MmapModelPagerTest {

    @Test
    fun testZeroCopyMmapInitialization() {
        val loadResult = NativeBridge.loadModel("/data/ind/models/Qwen2.5-1.5B-Instruct-Q4_K_M.gguf")
        assertTrue("loadModel should return true", loadResult)

        val telemetry = NativeBridge.getMmapStatus()
        assertNotNull("Telemetry must not be null", telemetry)
        assertTrue("Telemetry must report resident state", telemetry.contains("MAPPED_RESIDENT") || telemetry.contains("\"state\": 1") || telemetry.contains("\"state\":1"))
        assertTrue("Telemetry must include model size", telemetry.contains("mappedSizeMb"))
    }

    @Test
    fun testMadviseEvictionMeetsDodThresholds() {
        // Evict model weights via NativeBridge
        val evictionJson = NativeBridge.evictModel()
        assertNotNull("Eviction result must not be null", evictionJson)

        assertTrue("Eviction must report durationMs", evictionJson.contains("durationMs"))
        assertTrue("Eviction must report freedMb", evictionJson.contains("freedMb"))

        // Extract duration and freed numbers from JSON string
        val durationMatch = Regex("\"durationMs\"\\s*:\\s*([0-9.]+)").find(evictionJson)
        val freedMatch = Regex("\"freedMb\"\\s*:\\s*([0-9.]+)").find(evictionJson)

        assertNotNull("Must match durationMs", durationMatch)
        assertNotNull("Must match freedMb", freedMatch)

        val durationMs = durationMatch!!.groupValues[1].toDouble()
        val freedMb = freedMatch!!.groupValues[1].toDouble()

        assertTrue("Eviction duration must be < 15.0 ms per PRD DoD (was $durationMs ms)", durationMs < 15.0)
        assertTrue("Freed memory must exceed 900.0 MB per PRD DoD (was $freedMb MB)", freedMb > 900.0)
    }

    @Test
    fun testStreamingInferenceDispatchesTokens() {
        val tokens = mutableListOf<String>()
        var actionResult = ""
        var errorResult = ""
        val latch = CountDownLatch(1)

        NativeBridge.streamInference(
            "Torch chalu karo",
            "{}",
            object : NativeStreamListener {
                override fun onNativeToken(token: String) {
                    tokens.add(token)
                }

                override fun onNativeComplete(actionJson: String) {
                    actionResult = actionJson
                    latch.countDown()
                }

                override fun onNativeError(errorCode: Int, message: String) {
                    errorResult = message
                    latch.countDown()
                }
            }
        )

        latch.await(5, TimeUnit.SECONDS)

        assertTrue("Must receive streaming tokens", tokens.isNotEmpty())
        assertTrue("Must not encounter error", errorResult.isEmpty())
        assertTrue("Final action JSON must contain 'tier'", actionResult.contains("\"tier\""))
        assertTrue("Final action JSON must contain 'actions'", actionResult.contains("\"actions\""))
    }

    @Test
    fun testSprint4BenchmarkDoDMetrics() {
        val benchJson = NativeBridge.runSprint4Benchmark(32)
        assertNotNull("Benchmark JSON must not be null", benchJson)

        assertTrue("Must report tokensPerSecond", benchJson.contains("tokensPerSecond"))
        assertTrue("Must report peakRssMb", benchJson.contains("peakRssMb"))
        assertTrue("Must report evictionDurationMs", benchJson.contains("evictionDurationMs"))
        assertTrue("Must pass DoD", benchJson.contains("\"passedDod\": true") || benchJson.contains("\"passedDod\":true"))

        val tokRateMatch = Regex("\"tokensPerSecond\"\\s*:\\s*([0-9.]+)").find(benchJson)
        val rssMatch = Regex("\"peakRssMb\"\\s*:\\s*([0-9.]+)").find(benchJson)

        assertNotNull(tokRateMatch)
        assertNotNull(rssMatch)

        val tokRate = tokRateMatch!!.groupValues[1].toDouble()
        val rss = rssMatch!!.groupValues[1].toDouble()

        assertTrue("Sustained token rate must be >= 15.0 tok/s per DoD (was $tokRate)", tokRate >= 15.0)
        assertTrue("Peak RSS must be <= 1050 MB per DoD (was $rss MB)", rss <= 1050.0)
    }
}
