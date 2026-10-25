package indos.launcher

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.Build
import android.os.IBinder
import android.util.Log
import indos.neural.ipc.IIndAIManager
import java.util.Arrays
import kotlin.concurrent.thread

/**
 * Service hosted directly inside IndLauncher to benchmark Binder roundtrip latency
 * to IndNeuralCore, verifying Sprint 2 DoD:
 * "Round-trip Binder latency IndLauncher -> IndNeuralCore < 2.5 ms median over 1,000 calls"
 */
class IndLauncherIpcBenchmarkService : Service() {

    companion object {
        private const val TAG = "IND_IPC_BENCH"
        private const val CHANNEL_ID = "ind_launcher_bench"
        private const val NOTIFICATION_ID = 4001
        const val ACTION_RUN_LAUNCHER_BENCHMARK = "indos.launcher.ACTION_RUN_BENCHMARK"
        const val TARGET_MEDIAN_LATENCY_MS = 2.5
    }

    private var aiManager: IIndAIManager? = null
    private var isBound = false

    private val connection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            Log.i(TAG, "IndLauncher connected to IndNeuralCore Binder interface.")
            aiManager = IIndAIManager.Stub.asInterface(service)

            thread(name = "LauncherBenchThread") {
                try {
                    val manager = aiManager
                    if (manager != null) {
                        runBenchmark(manager, 1000)
                    } else {
                        Log.e(TAG, "aiManager is null in IndLauncher!")
                    }
                } catch (t: Throwable) {
                    Log.e(TAG, "Error running benchmark from IndLauncher", t)
                } finally {
                    unbindAndStop()
                }
            }
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            aiManager = null
            isBound = false
        }
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification("Running IndLauncher -> IndNeuralCore IPC Benchmark"))
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.i(TAG, "IndLauncher IPC Benchmark triggered. Binding to IndNeuralCore...")
        bindToAiService()
        return START_NOT_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun bindToAiService() {
        val intent = Intent().apply {
            component = ComponentName("in.indos.neural", "indos.neural.IndAIService")
        }
        val bound = bindService(intent, connection, Context.BIND_AUTO_CREATE)
        isBound = bound
        if (!bound) {
            Log.e(TAG, "IndLauncher failed to bind to IndAIService.")
            stopSelf()
        }
    }

    private fun unbindAndStop() {
        if (isBound) {
            try {
                unbindService(connection)
            } catch (e: Exception) {
                Log.w(TAG, "Error unbinding from IndLauncher", e)
            }
            isBound = false
        }
        stopSelf()
    }

    private fun runBenchmark(manager: IIndAIManager, iterations: Int) {
        Log.i(TAG, "=================================================================")
        Log.i(TAG, "STARTING SPRINT 2 DoD BENCHMARK: IndLauncher -> IndNeuralCore")
        Log.i(TAG, "Executing $iterations round-trip Binder calls...")
        Log.i(TAG, "=================================================================")

        // Warmup calls
        for (i in 1..50) {
            manager.classifyFast("Warmup payload", "NOTIFICATION_PRIORITY")
        }

        val latenciesNs = LongArray(iterations)
        val payload = "UPI notification: Rs 200 credited from user@upi"
        val domain = "NOTIFICATION_PRIORITY"

        val benchStartNs = System.nanoTime()
        for (i in 0 until iterations) {
            val callStartNs = System.nanoTime()
            manager.classifyFast(payload, domain)
            latenciesNs[i] = System.nanoTime() - callStartNs
        }
        val totalMs = (System.nanoTime() - benchStartNs) / 1_000_000.0

        val sortedNs = latenciesNs.clone()
        Arrays.sort(sortedNs)

        val minMs = sortedNs[0] / 1_000_000.0
        val maxMs = sortedNs[iterations - 1] / 1_000_000.0
        val avgMs = (sortedNs.sum().toDouble() / iterations) / 1_000_000.0
        val medianMs = sortedNs[iterations / 2] / 1_000_000.0
        val p95Ms = sortedNs[(iterations * 0.95).toInt()] / 1_000_000.0
        val p99Ms = sortedNs[(iterations * 0.99).toInt()] / 1_000_000.0
        val targetMet = medianMs < TARGET_MEDIAN_LATENCY_MS

        val engineStatus = manager.engineStatusJson

        Log.i(TAG, "[$TAG] Total Duration: %.2f ms for %d calls".format(totalMs, iterations))
        Log.i(TAG, "[$TAG] Latency Statistics (IndLauncher -> IndNeuralCore):")
        Log.i(TAG, "[$TAG]   Min:    %.3f ms".format(minMs))
        Log.i(TAG, "[$TAG]   Avg:    %.3f ms".format(avgMs))
        Log.i(TAG, "[$TAG]   Median: %.3f ms (DoD Target < %.1f ms)".format(medianMs, TARGET_MEDIAN_LATENCY_MS))
        Log.i(TAG, "[$TAG]   P95:    %.3f ms".format(p95Ms))
        Log.i(TAG, "[$TAG]   P99:    %.3f ms".format(p99Ms))
        Log.i(TAG, "[$TAG]   Max:    %.3f ms".format(maxMs))
        Log.i(
            TAG,
            "[$TAG] Sprint 2 DoD Result: %s (Median %.3f ms %s %.1f ms)".format(
                if (targetMet) "PASS" else "FAIL",
                medianMs,
                if (targetMet) "<" else ">=",
                TARGET_MEDIAN_LATENCY_MS
            )
        )
        Log.i(TAG, "[$TAG] Engine Status: $engineStatus")
        Log.i(TAG, "=================================================================")
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Ind Launcher Benchmark",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(text: String): Notification {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
                .setContentTitle("Ind Launcher Benchmark")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .build()
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
                .setContentTitle("Ind Launcher Benchmark")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .build()
        }
    }
}
