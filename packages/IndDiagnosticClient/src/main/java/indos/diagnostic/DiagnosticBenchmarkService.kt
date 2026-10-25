package indos.diagnostic

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
import kotlin.concurrent.thread

/**
 * Headless test harness service that connects to IndAIService over Binder IPC,
 * executes the 1,000-call benchmark loop, and logs results to logcat tag IND_IPC_BENCH.
 */
class DiagnosticBenchmarkService : Service() {

    companion object {
        private const val TAG = "IND_IPC_BENCH"
        private const val CHANNEL_ID = "ind_diagnostic_channel"
        private const val NOTIFICATION_ID = 3001
        const val ACTION_RUN_BENCHMARK = "indos.diagnostic.ACTION_RUN_BENCHMARK"
    }

    private var aiManager: IIndAIManager? = null
    private var isBound = false

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            Log.i(TAG, "Connected to IndAIService Binder interface.")
            aiManager = IIndAIManager.Stub.asInterface(service)

            thread(name = "IpcBenchmarkThread") {
                try {
                    val manager = aiManager
                    if (manager != null) {
                        IpcBenchmarkRunner.runBenchmark(manager, "DiagnosticBenchmarkService", 1000)
                    } else {
                        Log.e(TAG, "aiManager was null after service connection!")
                    }
                } catch (t: Throwable) {
                    Log.e(TAG, "Benchmark execution failed", t)
                } finally {
                    unbindAndStop()
                }
            }
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            Log.w(TAG, "IndAIService disconnected.")
            aiManager = null
            isBound = false
        }
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification("Ind OS Binder IPC Benchmark Active"))
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.i(TAG, "Starting DiagnosticBenchmarkService: initiating Binder connection...")
        bindToAiService()
        return START_NOT_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun bindToAiService() {
        val intent = Intent().apply {
            component = ComponentName("in.indos.neural", "indos.neural.IndAIService")
        }
        val bound = bindService(intent, serviceConnection, Context.BIND_AUTO_CREATE)
        isBound = bound
        if (!bound) {
            Log.e(TAG, "Failed to bind to IndAIService (is in.indos.neural installed?).")
            stopSelf()
        }
    }

    private fun unbindAndStop() {
        if (isBound) {
            try {
                unbindService(serviceConnection)
            } catch (e: Exception) {
                Log.w(TAG, "Error unbinding", e)
            }
            isBound = false
        }
        stopSelf()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Ind Diagnostic Benchmark",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(text: String): Notification {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
                .setContentTitle("Ind IPC Diagnostic")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .build()
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
                .setContentTitle("Ind IPC Diagnostic")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .build()
        }
    }
}
