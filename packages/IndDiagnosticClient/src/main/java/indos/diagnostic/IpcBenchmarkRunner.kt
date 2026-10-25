package indos.diagnostic

import android.util.Log
import indos.neural.ipc.IIndAIManager
import java.util.Arrays

/**
 * Executes high-precision Binder IPC round-trip latency benchmarks
 * between client processes and IndNeuralCore.
 *
 * Implements Sprint 2 DoD:
 * "Round-trip Binder latency IndLauncher -> IndNeuralCore < 2.5 ms median over 1,000 calls
 * (measure with System.nanoTime() loop in client; log to adb logcat -s IND_IPC_BENCH)."
 */
object IpcBenchmarkRunner {

    private const val TAG = "IND_IPC_BENCH"
    const val TARGET_MEDIAN_LATENCY_MS = 2.5

    data class BenchmarkResult(
        val totalCalls: Int,
        val minMs: Double,
        val maxMs: Double,
        val avgMs: Double,
        val medianMs: Double,
        val p95Ms: Double,
        val p99Ms: Double,
        val targetMet: Boolean,
        val engineStatus: String
    )

    fun runBenchmark(
        manager: IIndAIManager,
        clientName: String = "IndDiagnosticClient",
        iterations: Int = 1000
    ): BenchmarkResult {
        Log.i(TAG, "=================================================================")
        Log.i(TAG, "STARTING BINDER IPC BENCHMARK: $clientName -> IndNeuralCore")
        Log.i(TAG, "Planned iterations: $iterations calls (Target median < $TARGET_MEDIAN_LATENCY_MS ms)")
        Log.i(TAG, "=================================================================")

        // Warmup calls (50 iterations) to prime JIT compiler and Binder threadpools
        for (i in 1..50) {
            manager.classifyFast("Warmup payload $i", "NOTIFICATION_PRIORITY")
        }

        val latenciesNs = LongArray(iterations)
        val payload = "Important SBI transaction: OTP is 482910 for Rs. 500.00"
        val domain = "NOTIFICATION_PRIORITY"

        val benchStartNs = System.nanoTime()
        for (i in 0 until iterations) {
            val callStartNs = System.nanoTime()
            manager.classifyFast(payload, domain)
            latenciesNs[i] = System.nanoTime() - callStartNs
        }
        val totalBenchDurationMs = (System.nanoTime() - benchStartNs) / 1_000_000.0

        // Calculate statistics
        val sortedLatenciesNs = latenciesNs.clone()
        Arrays.sort(sortedLatenciesNs)

        val minMs = sortedLatenciesNs[0] / 1_000_000.0
        val maxMs = sortedLatenciesNs[iterations - 1] / 1_000_000.0
        val avgMs = (sortedLatenciesNs.sum().toDouble() / iterations) / 1_000_000.0
        val medianMs = sortedLatenciesNs[iterations / 2] / 1_000_000.0
        val p95Ms = sortedLatenciesNs[(iterations * 0.95).toInt()] / 1_000_000.0
        val p99Ms = sortedLatenciesNs[(iterations * 0.99).toInt()] / 1_000_000.0
        val targetMet = medianMs < TARGET_MEDIAN_LATENCY_MS

        val engineStatus = manager.engineStatusJson

        // Print machine-readable and DoD compliant logcat output
        Log.i(TAG, "[$TAG] Total Benchmark Duration: %.2f ms for %d calls".format(totalBenchDurationMs, iterations))
        Log.i(TAG, "[$TAG] Latency Statistics:")
        Log.i(TAG, "[$TAG]   Min:    %.3f ms".format(minMs))
        Log.i(TAG, "[$TAG]   Avg:    %.3f ms".format(avgMs))
        Log.i(TAG, "[$TAG]   Median: %.3f ms (Target < %.1f ms)".format(medianMs, TARGET_MEDIAN_LATENCY_MS))
        Log.i(TAG, "[$TAG]   P95:    %.3f ms".format(p95Ms))
        Log.i(TAG, "[$TAG]   P99:    %.3f ms".format(p99Ms))
        Log.i(TAG, "[$TAG]   Max:    %.3f ms".format(maxMs))
        Log.i(
            TAG,
            "[$TAG] Sprint 2 DoD Verification Result: %s (Median %.3f ms %s %.1f ms threshold)".format(
                if (targetMet) "PASS" else "FAIL",
                medianMs,
                if (targetMet) "<" else ">=",
                TARGET_MEDIAN_LATENCY_MS
            )
        )
        Log.i(TAG, "[$TAG] Remote Engine Telemetry: $engineStatus")
        Log.i(TAG, "=================================================================")

        return BenchmarkResult(
            totalCalls = iterations,
            minMs = minMs,
            maxMs = maxMs,
            avgMs = avgMs,
            medianMs = medianMs,
            p95Ms = p95Ms,
            p99Ms = p99Ms,
            targetMet = targetMet,
            engineStatus = engineStatus
        )
    }
}
