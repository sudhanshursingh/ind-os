package indos.launcher

import android.content.ComponentName
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import indos.launcher.model.AppInfo
import indos.launcher.model.PriorityNotification
import indos.launcher.model.PrivacyTogglesState
import indos.launcher.model.UpcomingAlarm
import indos.launcher.ui.IndLauncherScreen
import indos.launcher.ui.theme.IndLauncherTheme

/**
 * Zero-Bloat Intent Launcher for Ind OS.
 * Minimal RAM footprint (<45 MB resident), 120 Hz frame pacing, zero background trackers.
 */
class IndLauncherActivity : ComponentActivity() {

    companion object {
        private const val TAG = "IndLauncherActivity"
        private const val ACTION_SHOW_ORB = "indos.zerotouch.action.SHOW_ORB"
        private const val ZERO_TOUCH_PKG = "in.indos.zerotouch"
        private const val ORB_SERVICE = "indos.zerotouch.ui.IndOrbOverlayService"
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val installedApps = loadInstalledApps()
        val defaultNotifications = listOf(
            PriorityNotification(
                id = "notif_1",
                title = "State Bank of India",
                content = "UPI Alert: ₹2,450.00 credited to A/C XX4901 via IMPS.",
                sourceApp = "in.org.npci.upiapp",
                timestamp = "10m ago",
                isOtpOrFinancial = true,
                isSurakshaProtected = true
            ),
            PriorityNotification(
                id = "notif_2",
                title = "Ind Suraksha Shield",
                content = "Air-Gap Active: 0 background telemetry packets blocked today.",
                sourceApp = "in.indos.suraksha",
                timestamp = "1h ago",
                isOtpOrFinancial = false,
                isSurakshaProtected = true
            )
        )

        setContent {
            IndLauncherTheme {
                IndLauncherScreen(
                    apps = installedApps,
                    onLaunchApp = { app -> launchApp(app) },
                    onExecuteIntent = { intentQuery -> executeCommandIntent(intentQuery) },
                    onTriggerVoiceOrb = { triggerVoiceOrb() },
                    initialToggles = PrivacyTogglesState(
                        isMicMuted = false,
                        isCameraKilled = false,
                        isAirGapActive = true,
                        isSurakshaShieldActive = true
                    ),
                    initialAlarm = UpcomingAlarm(
                        timeString = "07:00 AM",
                        label = "Morning Briefing & News",
                        isEnabled = true,
                        daysSummary = "Mon - Fri"
                    ),
                    initialNotifications = defaultNotifications,
                    initialPromoDigest = indos.launcher.model.PromoDigestCardData(
                        totalPromosCount = 6,
                        sourceApps = listOf("Swiggy", "Zomato", "Myntra", "Amazon"),
                        headlineSummary = "Evening Digest: 6 promotional offers grouped quietly. Zero daytime interruptions.",
                        scheduledDigestTime = "07:30 PM",
                        promoSnippets = listOf(
                            "Swiggy: 50% off on your favorite dinner",
                            "Zomato: Free delivery on orders above ₹199",
                            "Myntra: Big Fashion Festival starts tonight"
                        )
                    )
                )
            }
        }
    }

    /**
     * Resolves all launchable activities via Android PackageManager without third-party bloat.
     */
    fun loadInstalledApps(): List<AppInfo> {
        val appList = mutableListOf<AppInfo>()
        try {
            val launcherIntent = Intent(Intent.ACTION_MAIN, null).apply {
                addCategory(Intent.CATEGORY_LAUNCHER)
            }
            val resolveInfos = packageManager.queryIntentActivities(
                launcherIntent,
                PackageManager.MATCH_ALL
            )

            for (ri in resolveInfos) {
                val pkg = ri.activityInfo.packageName
                // Skip launcher itself in the app drawer
                if (pkg == packageName) continue

                val label = ri.loadLabel(packageManager).toString()
                appList.add(
                    AppInfo(
                        label = label,
                        packageName = pkg,
                        activityName = ri.activityInfo.name,
                        isSystemApp = false,
                        icon = null
                    )
                )
            }
        } catch (e: Exception) {
            Log.w(TAG, "PackageManager query failed or running in JVM test: ${e.message}")
        }

        if (appList.isEmpty()) {
            // Provide default clean core apps for Ind OS
            appList.addAll(getDefaultIndApps())
        }

        return appList.sortedBy { it.label }
    }

    private fun getDefaultIndApps(): List<AppInfo> {
        return listOf(
            AppInfo("Phone", "com.android.dialer", "com.android.dialer.DialtactsActivity", true),
            AppInfo("Messages", "com.android.messaging", "com.android.messaging.ui.conversationlist.ConversationListActivity", true),
            AppInfo("Camera", "com.android.camera2", "com.android.camera.CameraLauncher", true),
            AppInfo("Gallery", "org.fossify.gallery", "org.fossify.gallery.activities.MainActivity", false),
            AppInfo("Contacts", "org.fossify.contacts", "org.fossify.contacts.activities.MainActivity", false),
            AppInfo("Settings", "com.android.settings", "com.android.settings.Settings", true),
            AppInfo("Ind Suraksha", "in.indos.suraksha", "indos.suraksha.SurakshaBlockActivity", true),
            AppInfo("Files", "com.android.documentsui", "com.android.documentsui.files.FilesActivity", true)
        )
    }

    private fun launchApp(app: AppInfo) {
        try {
            val intent = Intent().apply {
                component = ComponentName(app.packageName, app.activityName)
                flags = Intent.FLAG_ACTIVITY_NEW_TASK
            }
            startActivity(intent)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to launch ${app.packageName}: ${e.message}")
            try {
                val launchIntent = packageManager.getLaunchIntentForPackage(app.packageName)
                if (launchIntent != null) {
                    startActivity(launchIntent)
                }
            } catch (e2: Exception) {
                Log.e(TAG, "Fallback launch failed for ${app.packageName}: ${e2.message}")
            }
        }
    }

    private fun triggerVoiceOrb() {
        Log.i(TAG, "Triggering Ambient Voice Orb overlay from Ind Command Bar...")
        try {
            val intent = Intent(ACTION_SHOW_ORB).apply {
                component = ComponentName(ZERO_TOUCH_PKG, ORB_SERVICE)
                putExtra("extra_orb_state", "LISTENING")
            }
            startService(intent)
        } catch (e: Exception) {
            Log.w(TAG, "Unable to start IndOrbOverlayService directly: ${e.message}")
        }
    }

    private fun executeCommandIntent(query: String) {
        val q = query.trim().lowercase()
        Log.i(TAG, "Executing ambient intent from Ind Command Bar: '$query'")
        when {
            q.contains("torch") || q.contains("flashlight") -> {
                sendActionBroadcast("SYS_TOGGLE", "flashlight")
            }
            q.contains("wifi") -> {
                sendActionBroadcast("SYS_TOGGLE", "wifi")
            }
            q.contains("bluetooth") -> {
                sendActionBroadcast("SYS_TOGGLE", "bluetooth")
            }
            q.contains("dial") || q.contains("call") -> {
                try {
                    startActivity(Intent(Intent.ACTION_DIAL))
                } catch (e: Exception) {
                    Log.w(TAG, "Dialer intent failed: ${e.message}")
                }
            }
            q.contains("setting") -> {
                try {
                    startActivity(Intent(android.provider.Settings.ACTION_SETTINGS))
                } catch (e: Exception) {
                    Log.w(TAG, "Settings intent failed: ${e.message}")
                }
            }
            q.contains("samjhao") || (q.contains("translate") && q.contains("page")) || (q.contains("hindi") && q.contains("page")) -> {
                Log.i(TAG, "Routing on-screen Hindi translation & summarization request...")
                sendActionBroadcast("SCREEN_TRANSLATE_SUMMARIZE", "HINDI_3_BULLETS")
            }
            else -> {
                Log.i(TAG, "Routing generic intent to Ind Neural Core: $query")
                sendActionBroadcast("INTENT_EXECUTE", query)
            }
        }
    }

    private fun sendActionBroadcast(actionType: String, target: String) {
        try {
            val intent = Intent("indos.neural.ACTION_EXECUTE_ACTION").apply {
                putExtra("actionType", actionType)
                putExtra("target", target)
            }
            sendBroadcast(intent)
        } catch (e: Exception) {
            Log.w(TAG, "Broadcast failed: ${e.message}")
        }
    }
}
