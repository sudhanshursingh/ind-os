package indos.neural

import android.util.Log

/**
 * JNI Bridge connecting Java/Kotlin userspace to libind_ai.so (C++17).
 * Exposes hardware acceleration probe (Kryo 470 DotProd SIMD vs Adreno Vulkan)
 * and kernel-level madvise page eviction.
 */
object NativeBridge {

    private const val TAG = "IND_NATIVE"
    private var isLoaded = false

    init {
        try {
            System.loadLibrary("ind_ai")
            isLoaded = true
            Log.i(TAG, "libind_ai.so successfully loaded into process ${android.os.Process.myPid()}")
        } catch (e: UnsatisfiedLinkError) {
            Log.w(TAG, "libind_ai.so not available in current runtime environment (desktop JVM or unsupported ABI): ${e.message}")
            isLoaded = false
        }
    }

    fun isNativeLoaded(): Boolean = isLoaded

    /**
     * Executes the hardware acceleration probe and returns detailed JSON telemetry.
     */
    fun getHardwareReport(): String {
        return if (isLoaded) {
            runHardwareProbe()
        } else {
            // Graceful fallback for non-NDK test environments
            """
            {
                "targetSilicon": "Qualcomm Snapdragon 732G (SM7150-AC)",
                "cpuCoreCount": 8,
                "optimalKryoThreads": 4,
                "hasArmv82DotProd": true,
                "hasFp16": true,
                "hasNeon": true,
                "gpuName": "Adreno (TM) 618",
                "hasVulkanCompute": true,
                "gemmBenchmarks": {
                    "cpu1ThreadGflops": 12.4,
                    "cpu4ThreadGflops": 46.8,
                    "cpu4ThreadSpeedup": 3.77,
                    "vulkanGpuGflops": 142.5,
                    "estimatedTokensPerSec": 17.5
                },
                "simulatedFallback": true
            }
            """.trimIndent()
        }
    }

    /**
     * Native C++ declarations implemented in ind_jni_bridge.cpp
     */
    external fun runHardwareProbe(): String
    external fun runCpuGemmBenchmark(threads: Int, dim: Int): Double
    external fun runVulkanComputeBenchmark(dim: Int): Double
    external fun evictMmapRange(addr: Long, length: Long): Boolean
    external fun getNativeStatus(): String
}
