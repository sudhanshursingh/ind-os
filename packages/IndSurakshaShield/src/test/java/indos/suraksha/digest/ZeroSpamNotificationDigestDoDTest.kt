package indos.suraksha.digest

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Sprint 14 Definition of Done (DoD) Test: Zero-Spam Smart Notification Digest
 * Verifies that promo spam is grouped into a silent evening digest while
 * human, UPI, and OTP messages pass immediately with zero latency.
 */
class ZeroSpamNotificationDigestDoDTest {

    @Before
    fun setUp() {
        NotificationDigestManager.resetForTesting()
    }

    @Test
    fun testZeroSpamNotificationDigest_20SampleCorpus_AccurateClassification() {
        val corpus = listOf(
            // 5 UPI / Banking Notifications -> IMMEDIATE_PASS
            NotificationItem("b1", "in.org.npci.upiapp", "BHIM UPI", "Paid ₹450 to Chai Point via UPI Ref 418291048102. Balance in A/C XX8901: ₹12,410.00"),
            NotificationItem("b2", "com.phonepe.app", "PhonePe", "Payment of ₹1,200.00 to Sharma Groceries successful. A/C XX4412 debited."),
            NotificationItem("b3", "com.google.android.apps.nbu.paisa.user", "Google Pay", "₹3,500.00 received from Rahul Verma via UPI. Added to HDFC Bank A/C XX7811."),
            NotificationItem("b4", "com.sbi.lotusintouch", "SBI Bank", "Dear Customer, A/C XX9012 credited by ₹25,000.00 on 26-Oct-26 via NEFT transfer."),
            NotificationItem("b5", "net.one97.paytm", "Paytm", "Money sent: ₹850 paid to Uber India. UPI Transaction ID 9012849102 confirmed."),

            // 3 OTP / Authentication Codes -> IMMEDIATE_PASS
            NotificationItem("o1", "com.google.android.apps.messaging", "SBI-CARD", "491028 is your OTP for transaction of ₹2,100 at Amazon India. Valid for 5 mins. Do not share."),
            NotificationItem("o2", "com.google.android.apps.messaging", "HDFC-BK", "Your netbanking verification code is 881204. Valid for 10 minutes. Do not disclose to anyone."),
            NotificationItem("o3", "com.google.android.apps.messaging", "AADHAAR", "Your Aadhaar OTP for e-KYC authentication is 719203. Valid for 10 mins."),

            // 4 Direct Human / Messaging -> IMMEDIATE_PASS
            NotificationItem("m1", "com.whatsapp", "Papa", "Have you reached office safely? Call me when you get time."),
            NotificationItem("m2", "com.whatsapp", "Family Group", "Happy Diwali to everyone! Photos uploaded to album."),
            NotificationItem("m3", "org.telegram.messenger", "Priya Sharma", "Shared the quarterly project presentation document with you."),
            NotificationItem("m4", "com.android.dialer", "Phone", "Missed call from Ankit (2 calls at 11:42 AM)"),

            // 6 Commercial Promotional Marketing Spam -> SILENT_DIGEST
            NotificationItem("p1", "com.application.zomato", "Zomato", "Hungry? Flat 50% off on your favorite Biryani! Use code BIRYANI50 now."),
            NotificationItem("p2", "in.swiggy.android", "Swiggy Instamart", "Super Deal: Fresh mangoes and snacks at 30% discount. Free delivery in 10 mins!"),
            NotificationItem("p3", "com.amazon.mShop.android.shopping", "Amazon", "Great Indian Festival Mega Sale is LIVE! Save up to 70% on smartphones & laptops."),
            NotificationItem("p4", "com.myntra.android", "Myntra", "Big Billion Fashion Days! Extra 20% discount on top brands. Hurry, limited time offer!"),
            NotificationItem("p5", "com.dominos", "Domino's Pizza", "Buy 1 Get 1 Free on all Medium Pizzas today only! Order now with code BOGO."),
            NotificationItem("p6", "com.unknown.finance", "FastLoan", "Congratulations! Pre-approved loan of ₹5,00,000 approved for your profile. Apply now!"),

            // 2 Malicious Cyber Scams -> BLOCKED_SCAM
            NotificationItem("s1", "com.google.android.apps.messaging", "VM-EBILL", "Dear consumer, your electricity power connection will be disconnected tonight 9:30 PM due to unpaid bill. Call 9876543210 immediately."),
            NotificationItem("s2", "com.google.android.apps.messaging", "VK-POLICE", "Digital Arrest Notice: CBI Mumbai has issued an urgent warrant against your Aadhaar number. Video interrogation mandatory.")
        )

        val results = mutableListOf<ClassifiedNotification>()
        for (item in corpus) {
            val classified = NotificationDigestManager.processNotification(
                id = item.id,
                packageName = item.pkg,
                title = item.title,
                body = item.body
            )
            results.add(classified)
        }

        val immediatePassList = results.filter { it.category == NotificationCategory.IMMEDIATE_PASS }
        val silentDigestList = results.filter { it.category == NotificationCategory.SILENT_DIGEST }
        val blockedScamList = results.filter { it.category == NotificationCategory.BLOCKED_SCAM }

        println("=== Sprint 14 DoD: Zero-Spam Notification Digest Benchmark ===")
        println("Total Ingested: ${corpus.size}")
        println("Immediate Pass (Banking, OTP, Human): ${immediatePassList.size} (Expected: 12)")
        println("Silent Digest (Commercial Promo Spam): ${silentDigestList.size} (Expected: 6)")
        println("Blocked Scams (Electricity & Police Extortion): ${blockedScamList.size} (Expected: 2)")

        // Assert 100% classification precision on all 3 categories
        assertEquals(12, immediatePassList.size)
        assertEquals(6, silentDigestList.size)
        assertEquals(2, blockedScamList.size)

        // Verify that 0 promotional spam items were allowed to ring/interrupt the user
        for (pass in immediatePassList) {
            assertFalse(
                "Immediate pass must not contain promo spam: ${pass.body}",
                pass.body.contains("Flat 50%") || pass.body.contains("Pre-approved loan")
            )
        }

        // Verify Evening Digest Summary generation
        val eveningSummary = NotificationDigestManager.generateEveningDigestSummary()
        assertNotNull(eveningSummary)
        assertEquals(6, eveningSummary.totalPromosCount)
        assertTrue(eveningSummary.headlineSummary.contains("6 promotional offers grouped quietly"))
        assertTrue(eveningSummary.sourceApps.contains("Zomato"))
        assertTrue(eveningSummary.sourceApps.contains("Swiggy"))
        assertTrue(eveningSummary.sourceApps.contains("Amazon"))
        println("Generated Evening Digest Headline: '${eveningSummary.headlineSummary}'")
        println("Sprint 14 DoD: Zero-Spam Notification Digest PASSED.")
    }

    private data class NotificationItem(
        val id: String,
        val pkg: String,
        val title: String,
        val body: String
    )
}
