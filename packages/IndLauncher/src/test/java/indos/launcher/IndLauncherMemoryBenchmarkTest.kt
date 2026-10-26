package indos.launcher

import indos.launcher.perf.FramePacingBenchmark
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Validates memory footprint constraint (<45 MB RAM) and zero background trackers.
 */
class IndLauncherMemoryBenchmarkTest {

    @Test
    fun testMemoryFootprint_StrictlyUnder45MB() {
        val ramMb = FramePacingBenchmark.getResidentMemoryUsageMb()
        println("IndLauncher Resident Memory: $ramMb MB (Budget: 45.0 MB)")
        assertTrue("IndLauncher resident RAM must be < 45.0 MB, measured: $ramMb MB", ramMb < 45.0)
    }

    @Test
    fun testZeroBackgroundTrackers_NoCommercialTelemetry() {
        val verified = FramePacingBenchmark.verifyZeroTrackers()
        assertTrue("IndLauncher must contain zero commercial ad/telemetry trackers", verified)
    }
}
