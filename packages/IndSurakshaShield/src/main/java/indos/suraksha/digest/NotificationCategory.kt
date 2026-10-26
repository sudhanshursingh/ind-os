package indos.suraksha.digest

// ==============================================================================
// Ind OS — Zero-Spam Smart Notification Classification
// ==============================================================================

/**
 * Routing classification for incoming notifications in Ind OS.
 */
enum class NotificationCategory {
    /**
     * Critical/Urgent notifications that must ring/vibrate immediately:
     * - Bank debits/credits & UPI confirmations
     * - One-Time Passwords (OTPs) and 2FA codes
     * - Direct human calls and private messages (WhatsApp, SMS, Telegram)
     */
    IMMEDIATE_PASS,

    /**
     * Commercial and promotional marketing campaigns grouped quietly into an evening digest:
     * - Food delivery discounts (Zomato, Swiggy)
     * - E-commerce sales & flash deals (Amazon, Flipkart, Myntra)
     * - Loan & credit card solicitations
     * - Game rewards & casino spam
     */
    SILENT_DIGEST,

    /**
     * Malicious threats blocked outright by Ind Suraksha Shield:
     * - Fake electricity bill disconnection threats
     * - Digital arrest extortion
     * - Phishing KYC links
     */
    BLOCKED_SCAM
}

/**
 * Clean data model representing a classified notification.
 */
data class ClassifiedNotification(
    val id: String,
    val packageName: String,
    val senderOrTitle: String,
    val body: String,
    val timestampMs: Long,
    val category: NotificationCategory,
    val classificationReason: String,
    val isOtpOrFinancial: Boolean = false
)

/**
 * Summary representation of the batched silent evening digest.
 */
data class EveningDigestSummary(
    val totalPromosCount: Int,
    val sourceApps: List<String>,
    val headlineSummary: String,
    val promoSnippets: List<String>,
    val scheduledDigestTime: String = "07:30 PM",
    val formattedDate: String = "Today"
)
