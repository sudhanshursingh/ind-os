package indos.suraksha.digest

import indos.suraksha.scam.ScamClassifier
import java.util.regex.Pattern

// ==============================================================================
// Ind OS — Zero-Spam Smart Notification Classifier
// Ensures human, banking, and OTP messages pass instantly without latency,
// while marketing promotions and e-commerce spam are silently batched.
// ==============================================================================

object NotificationDigestClassifier {

    private val BANKING_OR_UPI_PATTERNS = listOf(
        Pattern.compile("(?i)(credited|debited|spent|transferred|withdrawn)"),
        Pattern.compile("(?i)(a/c|acct|acc)\\s*(no\\.?|xx[0-9]+|ending)"),
        Pattern.compile("(?i)(₹|rs\\.?|inr)\\s*[0-9]+([.,][0-9]{2})?"),
        Pattern.compile("(?i)(upi\\s*ref|rrn|imps|neft|rtgs)"),
        Pattern.compile("(?i)(payment\\s*(successful|received|confirmed))")
    )

    private val OTP_PATTERNS = listOf(
        Pattern.compile("(?i)\\b(otp|one\\s*time\\s*password|verification\\s*code)\\b"),
        Pattern.compile("(?i)\\b[0-9]{4,8}\\s*(is\\s*your\\s*code|is\\s*your\\s*otp)\\b"),
        Pattern.compile("(?i)(valid\\s*for|do\\s*not\\s*share)")
    )

    private val PROMO_SPAM_PATTERNS = listOf(
        Pattern.compile("(?i)(flat\\s*[0-9]+%\\s*off|[0-9]+%\\s*discount|save\\s*up\\s*to)"),
        Pattern.compile("(?i)(mega\\s*sale|flash\\s*sale|great\\s*indian\\s*festival|big\\s*billion|super\\s*deal)"),
        Pattern.compile("(?i)(use\\s*code|apply\\s*coupon|promo\\s*code|free\\s*delivery)"),
        Pattern.compile("(?i)(order\\s*now|buy\\s*1\\s*get\\s*1|hurry\\s*up|limited\\s*time\\s*offer)"),
        Pattern.compile("(?i)(pre-approved\\s*loan|credit\\s*card\\s*approved|apply\\s*for\\s*loan|instant\\s*cash)"),
        Pattern.compile("(?i)(win\\s*cash|jackpot|rummy|claim\\s*bonus|spin\\s*the\\s*wheel)")
    )

    private val KNOWN_COMMUNICATION_PACKAGES = setOf(
        "com.whatsapp",
        "org.telegram.messenger",
        "com.google.android.talk",
        "com.google.android.apps.messaging",
        "com.android.mms",
        "com.android.messaging",
        "com.android.dialer",
        "com.google.android.dialer"
    )

    private val KNOWN_FINANCIAL_PACKAGES = setOf(
        "in.org.npci.upiapp",
        "com.phonepe.app",
        "com.google.android.apps.nbu.paisa.user",
        "net.one97.paytm",
        "com.sbi.lotusintouch",
        "com.hdfcbank.android",
        "com.icicibank.mobile",
        "com.axis.mobile",
        "com.pnb.android"
    )

    /**
     * Classifies an incoming notification into IMMEDIATE_PASS, SILENT_DIGEST, or BLOCKED_SCAM.
     */
    fun classifyNotification(
        id: String,
        packageName: String,
        title: String,
        body: String,
        timestampMs: Long = System.currentTimeMillis()
    ): ClassifiedNotification {
        val combinedText = "$title $body"

        // Step 1: Scan for Phishing & Indian Cyber Scams via Suraksha ScamClassifier
        val scamResult = ScamClassifier.getInstance().classify(title, body)
        if (scamResult.isScam) {
            return ClassifiedNotification(
                id = id,
                packageName = packageName,
                senderOrTitle = title,
                body = body,
                timestampMs = timestampMs,
                category = NotificationCategory.BLOCKED_SCAM,
                classificationReason = "Malicious Scam Intercepted: ${scamResult.category}",
                isOtpOrFinancial = false
            )
        }

        // Step 2: Check for OTP and Authentication Codes -> IMMEDIATE_PASS
        if (isOtpMessage(combinedText)) {
            return ClassifiedNotification(
                id = id,
                packageName = packageName,
                senderOrTitle = title,
                body = body,
                timestampMs = timestampMs,
                category = NotificationCategory.IMMEDIATE_PASS,
                classificationReason = "Critical Authentication OTP",
                isOtpOrFinancial = true
            )
        }

        // Step 3: Check for UPI / Banking Transaction Confirmations -> IMMEDIATE_PASS
        if (isBankingOrUpiMessage(packageName, combinedText)) {
            return ClassifiedNotification(
                id = id,
                packageName = packageName,
                senderOrTitle = title,
                body = body,
                timestampMs = timestampMs,
                category = NotificationCategory.IMMEDIATE_PASS,
                classificationReason = "Critical Financial/UPI Transaction Alert",
                isOtpOrFinancial = true
            )
        }

        // Step 4: Check for Commercial Marketing / Promo Spam -> SILENT_DIGEST
        if (isPromotionalSpam(combinedText)) {
            return ClassifiedNotification(
                id = id,
                packageName = packageName,
                senderOrTitle = title,
                body = body,
                timestampMs = timestampMs,
                category = NotificationCategory.SILENT_DIGEST,
                classificationReason = "Promotional Marketing Campaign (Queued for Silent Evening Digest)",
                isOtpOrFinancial = false
            )
        }

        // Step 5: Direct Human-to-Human Messages -> IMMEDIATE_PASS
        if (isHumanDirectMessage(packageName, combinedText)) {
            return ClassifiedNotification(
                id = id,
                packageName = packageName,
                senderOrTitle = title,
                body = body,
                timestampMs = timestampMs,
                category = NotificationCategory.IMMEDIATE_PASS,
                classificationReason = "Direct Human Communication",
                isOtpOrFinancial = false
            )
        }

        // Fallback: Default to immediate pass for system and unclassified alerts
        return ClassifiedNotification(
            id = id,
            packageName = packageName,
            senderOrTitle = title,
            body = body,
            timestampMs = timestampMs,
            category = NotificationCategory.IMMEDIATE_PASS,
            classificationReason = "Standard System Notification",
            isOtpOrFinancial = false
        )
    }

    private fun isOtpMessage(text: String): Boolean {
        for (pattern in OTP_PATTERNS) {
            if (pattern.matcher(text).find()) {
                return true
            }
        }
        return false
    }

    private fun isBankingOrUpiMessage(pkg: String, text: String): Boolean {
        if (KNOWN_FINANCIAL_PACKAGES.contains(pkg)) {
            // If from known bank/UPI app, check if it contains transaction indicators
            for (pattern in BANKING_OR_UPI_PATTERNS) {
                if (pattern.matcher(text).find()) {
                    return true
                }
            }
        }

        // Matches financial SMS / notifications from banks even if received via SMS app
        var matchCount = 0
        for (pattern in BANKING_OR_UPI_PATTERNS) {
            if (pattern.matcher(text).find()) {
                matchCount++
            }
        }
        return matchCount >= 2
    }

    private fun isPromotionalSpam(text: String): Boolean {
        for (pattern in PROMO_SPAM_PATTERNS) {
            if (pattern.matcher(text).find()) {
                return true
            }
        }
        return false
    }

    private fun isHumanDirectMessage(pkg: String, text: String): Boolean {
        if (KNOWN_COMMUNICATION_PACKAGES.contains(pkg)) {
            // Verify it is not an automated promotional broadcast
            return !isPromotionalSpam(text)
        }
        return false
    }
}
