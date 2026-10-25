package indos.neural

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.provider.Settings
import android.util.Log

/**
 * Service to execute privileged hardware and settings operations
 * verifying WRITE_SECURE_SETTINGS without user prompts.
 * Satisfies Sprint 1 DoD.
 */
class IndSystemPrivilegeTestService : Service() {

    companion object {
        private const val TAG = "IND_PRIVILEGE_TEST"
        private const val CHANNEL_ID = "ind_system_privilege_test"
        private const val NOTIFICATION_ID = 1001
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        val notification = createNotification("Ind OS Privileged Test Service Active")
        startForeground(NOTIFICATION_ID, notification)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val toggleTarget = intent?.getStringExtra("toggle") ?: "wifi"
        Log.i(TAG, "Executing privileged toggle test for: $toggleTarget")

        when (toggleTarget.lowercase()) {
            "wifi" -> {
                val current = Settings.Global.getInt(contentResolver, Settings.Global.WIFI_ON, 0)
                val target = if (current == 1) 0 else 1
                Settings.Global.putInt(contentResolver, Settings.Global.WIFI_ON, target)
                Log.i(TAG, "Privileged toggle: Settings.Global.WIFI_ON changed from $current to $target")
            }
            "brightness" -> {
                val targetBrightness = intent?.getIntExtra("value", 150) ?: 150
                Settings.System.putInt(contentResolver, Settings.System.SCREEN_BRIGHTNESS, targetBrightness)
                Log.i(TAG, "Privileged toggle: Settings.System.SCREEN_BRIGHTNESS set to $targetBrightness")
            }
            else -> {
                Log.w(TAG, "Unknown toggle target: $toggleTarget")
            }
        }

        stopSelf(startId)
        return START_NOT_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Ind OS Privileged Testing",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(text: String): Notification {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
                .setContentTitle("Ind OS System Service")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .build()
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
                .setContentTitle("Ind OS System Service")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .build()
        }
    }
}
