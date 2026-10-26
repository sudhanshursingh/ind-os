package indos.suraksha.digest

import android.util.Log
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicInteger

// ==============================================================================
// Ind OS — Zero-Spam Smart Notification Digest Manager
// Manages silent queuing of commercial promo spam and real-time pass-through
// of critical human, UPI, and OTP alerts.
// ==============================================================================

object NotificationDigestManager {

    private const val TAG = "NotificationDigestManager"

    private val immediateNotifications = CopyOnWriteArrayList<ClassifiedNotification>()
    private val silentDigestQueue = CopyOnWriteArrayList<ClassifiedNotification>()
    private val blockedScamCounter = AtomicInteger(0)

    /**
     * Ingests and routes an incoming notification based on zero-spam policies.
     */
    fun processNotification(
        id: String,
        packageName: String,
        title: String,
        body: String,
        timestampMs: Long = System.currentTimeMillis()
    ): ClassifiedNotification {
        val classified = NotificationDigestClassifier.classifyNotification(
            id = id,
            packageName = packageName,
            title = title,
            body = body,
            timestampMs = timestampMs
        )

        when (classified.category) {
            NotificationCategory.IMMEDIATE_PASS -> {
                Log.i(TAG, "[IMMEDIATE_PASS] Allowed to ring/vibrate: '${classified.senderOrTitle}' from $packageName (${classified.classificationReason})")
                immediateNotifications.add(0, classified)
            }
            NotificationCategory.SILENT_DIGEST -> {
                Log.i(TAG, "[SILENT_DIGEST] Suppressed and batched quietly: '${classified.senderOrTitle}' from $packageName (${classified.classificationReason})")
                silentDigestQueue.add(0, classified)
            }
            NotificationCategory.BLOCKED_SCAM -> {
                Log.w(TAG, "[BLOCKED_SCAM] Intercepted and blocked: '${classified.senderOrTitle}' (${classified.classificationReason})")
                blockedScamCounter.incrementAndGet()
            }
        }

        return classified
    }

    /**
     * Returns all immediately passed notifications.
     */
    fun getImmediateNotifications(): List<ClassifiedNotification> {
        return immediateNotifications.toList()
    }

    /**
     * Returns all promotional notifications silently held in the digest queue.
     */
    fun getSilentDigestQueue(): List<ClassifiedNotification> {
        return silentDigestQueue.toList()
    }

    /**
     * Returns total count of scam notifications blocked by Suraksha.
     */
    fun getBlockedScamCount(): Int {
        return blockedScamCounter.get()
    }

    /**
     * Generates an aggregated Evening Digest summary for the user.
     */
    fun generateEveningDigestSummary(scheduledTime: String = "07:30 PM"): EveningDigestSummary {
        val promos = silentDigestQueue.toList()
        val distinctApps = promos.map { it.packageName }.distinct()
        val appNames = distinctApps.map { pkg ->
            when {
                pkg.contains("zomato") -> "Zomato"
                pkg.contains("swiggy") -> "Swiggy"
                pkg.contains("amazon") -> "Amazon"
                pkg.contains("flipkart") -> "Flipkart"
                pkg.contains("myntra") -> "Myntra"
                pkg.contains("dominos") -> "Domino's"
                else -> pkg.substringAfterLast('.').replaceFirstChar { it.uppercase() }
            }
        }

        val headline = if (promos.isEmpty()) {
            "Zero promotional spam today. Your day remained quiet."
        } else {
            "Evening Digest: ${promos.size} promotional offers grouped quietly from ${appNames.take(3).joinToString(", ")}. No action required."
        }

        val snippets = promos.take(5).map { "${it.senderOrTitle}: ${it.body}" }

        return EveningDigestSummary(
            totalPromosCount = promos.size,
            sourceApps = appNames,
            headlineSummary = headline,
            promoSnippets = snippets,
            scheduledDigestTime = scheduledTime
        )
    }

    /**
     * Clears all queues (used for testing and daily reset).
     */
    fun resetForTesting() {
        immediateNotifications.clear()
        silentDigestQueue.clear()
        blockedScamCounter.set(0)
    }
}
