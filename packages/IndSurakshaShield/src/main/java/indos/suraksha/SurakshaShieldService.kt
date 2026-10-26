package indos.suraksha

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.util.Log

/**
 * Privileged security watchdog service managing:
 * 1. Screen-share OTP blackout
 * 2. Banking trojan APK installation blocking
 * 3. Offline scam SMS classification
 */
class SurakshaShieldService : Service() {

    companion object {
        private const val TAG = "IND_SURAKSHA"
        private const val CHANNEL_ID = "ind_suraksha_channel"
        private const val NOTIFICATION_ID = 2001

        @Volatile
        var instance: SurakshaShieldService? = null
            private set
    }

    val policyEnforcer = indos.suraksha.policy.ActionPolicyEnforcer.getInstance()
    val firewall = indos.suraksha.firewall.PromptInjectionFirewall.INSTANCE
    val isolationEngine = indos.suraksha.isolation.DualContextIsolationEngine.INSTANCE

    lateinit var screenShareDetector: indos.suraksha.screenshare.ScreenShareDetector
    lateinit var otpInterceptor: indos.suraksha.screenshare.OtpNotificationInterceptor
    lateinit var bankingWindowGuard: indos.suraksha.screenshare.BankingWindowGuard

    override fun onCreate() {
        super.onCreate()
        instance = this
        screenShareDetector = indos.suraksha.screenshare.ScreenShareDetector.getInstance(this)
        screenShareDetector.register()
        otpInterceptor = indos.suraksha.screenshare.OtpNotificationInterceptor(screenShareDetector)
        bankingWindowGuard = indos.suraksha.screenshare.BankingWindowGuard(screenShareDetector)

        createNotificationChannel()
        val notification = createNotification("Ind Suraksha Shield Active")
        startForeground(NOTIFICATION_ID, notification)
        Log.i(TAG, "IndSurakshaShield service started (Sprint 11 Screen-Share OTP Blackout & Banking Guard active).")
    }

    override fun onDestroy() {
        if (::screenShareDetector.isInitialized) {
            screenShareDetector.unregister()
        }
        if (instance === this) {
            instance = null
        }
        super.onDestroy()
        Log.i(TAG, "IndSurakshaShield service destroyed.")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Ind Suraksha Shield",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(text: String): Notification {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
                .setContentTitle("Ind Suraksha Shield")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_lock_lock)
                .build()
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
                .setContentTitle("Ind Suraksha Shield")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_lock_lock)
                .build()
        }
    }
}
