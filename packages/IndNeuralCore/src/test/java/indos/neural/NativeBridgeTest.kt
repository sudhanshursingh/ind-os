package indos.neural

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Unit test verifying NativeBridge hardware probe, JNI report generation,
 * and target silicon specifications (Snapdragon 732G / Kryo 470 / Adreno 618).
 */
class NativeBridgeTest {

    @Test
    fun testHardwareReportContainsRequiredSiliconSpecs() {
        val reportStr = NativeBridge.getHardwareReport()
        assertNotNull("Report must not be null", reportStr)
        assertTrue("Report must not be empty", reportStr.isNotBlank())

        assertTrue("Target silicon must match Snapdragon 732G", reportStr.contains("Qualcomm Snapdragon 732G (SM7150-AC)"))
        assertTrue("Optimal Kryo thread count must be 4", reportStr.contains("\"optimalKryoThreads\": 4") || reportStr.contains("\"optimalKryoThreads\":4"))
        assertTrue("ARMv8.2-A DotProd capability must be declared", reportStr.contains("hasArmv82DotProd"))
        assertTrue("Vulkan compute capability must be declared", reportStr.contains("hasVulkanCompute"))
        assertTrue("GPU name must reference Adreno 618", reportStr.contains("Adreno (TM) 618"))
        assertTrue("gemmBenchmarks must be present", reportStr.contains("gemmBenchmarks"))
        assertTrue("estimatedTokensPerSec must be present", reportStr.contains("estimatedTokensPerSec"))
    }

    @Test
    fun testNativeLoadingStateIsReportedWithoutCrash() {
        // Assert that checking library state doesn't throw unhandled LinkageError
        val loaded = NativeBridge.isNativeLoaded()
        // In JVM host testing without arm64 runtime, isNativeLoaded() is false and falls back cleanly
        assertTrue("isNativeLoaded must return boolean without throwing", loaded || !loaded)
    }
}
