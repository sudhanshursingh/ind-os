/*
 * Copyright (C) 2026 Ind OS Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package indos.suraksha

import indos.suraksha.scam.ScamClassifier
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Unit tests verifying Indian scam pattern classification across all 5 archetypes.
 */
class ScamClassifierTest {

    private lateinit var classifier: ScamClassifier

    @Before
    fun setUp() {
        classifier = ScamClassifier.getInstance()
        classifier.resetCounters()
    }

    @Test
    fun testElectricityBillDisconnectionScam() {
        val message = "Dear consumer, your electricity power supply will be disconnected tonight at 9:30 PM because previous month bill was not updated. Immediately call electricity officer 9876543210."
        val result = classifier.classify("DISCOM-ALERT", message)

        assertTrue("Must be classified as scam", result.isScam)
        assertEquals(ScamClassifier.ScamCategory.ELECTRICITY_BILL, result.category)
        assertTrue("Confidence must be high", result.confidence >= 0.90f)
        assertTrue(result.explanationEn.contains("electricity bill disconnection"))
        assertTrue(result.explanationHi.contains("बिजली बिल"))
    }

    @Test
    fun testBankingPanKycScam() {
        val message = "Dear SBI User, your YONO account has been suspended today due to pending PAN card KYC update. Click here to verify pan card: bit.ly/sbi-pan-update"
        val result = classifier.classify("SBI-KYC", message)

        assertTrue(result.isScam)
        assertEquals(ScamClassifier.ScamCategory.BANK_KYC_PAN, result.category)
        assertTrue(result.confidence >= 0.95f)
        assertTrue(result.explanationEn.contains("bank PAN/KYC"))
    }

    @Test
    fun testDigitalArrestExtortionScam() {
        val message = "This is CBI officer Rajesh Sharma from Cyber Crime Cell. A customs parcel under your Aadhaar containing illegal narcotics was seized. You are placed under digital arrest. Stay on video call or police will arrive."
        val result = classifier.classify("POLICE-ALERT", message)

        assertTrue(result.isScam)
        assertEquals(ScamClassifier.ScamCategory.DIGITAL_ARREST, result.category)
        assertEquals(1.0f, result.riskScore, 0.01f)
        assertTrue(result.explanationEn.contains("Digital Arrest"))
        assertTrue(result.explanationHi.contains("डिजिटल अरेस्ट"))
    }

    @Test
    fun testFakeCourierApkScam() {
        val message = "India Post alert: Your parcel could not be delivered due to wrong delivery address. Please download tracking app india-post-delivery.apk to reschedule delivery."
        val result = classifier.classify("IND-POST", message)

        assertTrue(result.isScam)
        assertEquals(ScamClassifier.ScamCategory.FAKE_COURIER_APK, result.category)
        assertTrue(result.confidence >= 0.90f)
    }

    @Test
    fun testLotteryJobFraudScam() {
        val message = "Congratulations! Your mobile number won 25 Lakh in KBC lottery lucky draw 2026. Contact KBC manager on WhatsApp to claim prize money."
        val result = classifier.classify("KBC-LOTTERY", message)

        assertTrue(result.isScam)
        assertEquals(ScamClassifier.ScamCategory.LOTTERY_JOB_FRAUD, result.category)
    }

    @Test
    fun testCleanMessagePassThrough() {
        val message = "Bhai, sham ko 7 baje chai pe milte hain. Laptop lete aana."
        val result = classifier.classify("+919876543210", message)

        assertFalse("Clean message must not be flagged as scam", result.isScam)
        assertEquals(ScamClassifier.ScamCategory.SAFE, result.category)
        assertTrue(result.riskScore < 0.1f)
    }
}
