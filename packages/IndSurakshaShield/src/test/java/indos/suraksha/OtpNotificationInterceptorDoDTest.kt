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

import indos.suraksha.screenshare.OtpNotificationInterceptor
import indos.suraksha.screenshare.ScreenShareDetector
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Sprint 11 Definition of Done (DoD) Test:
 *
 * "Start screen-share, send test bank OTP SMS -> banner redacted, TTS OTP readout blocked (verify via notification listener logs)."
 *
 * BIBLE Reference: PRD v1.0.1 §4.3-C & Sprint 11 (lines 248, 419-420).
 */
class OtpNotificationInterceptorDoDTest {

    private lateinit var detector: ScreenShareDetector
    private lateinit var interceptor: OtpNotificationInterceptor

    @Before
    fun setUp() {
        detector = ScreenShareDetector(null)
        interceptor = OtpNotificationInterceptor(detector)
        interceptor.resetCounters()
    }

    @Test
    fun testSprint11DoD_ScreenShareOtpBlackoutAndTtsBlock() {
        println("[IND_BENCHMARK] ==================================================================")
        println("[IND_BENCHMARK] Starting Sprint 11 DoD Screen-Share OTP Blackout & TTS Guard Test")
        println("[IND_BENCHMARK] ==================================================================")

        // 1. Initial State: Screen share is INACTIVE
        detector.setSimulatedState(false, false)

        val testBankOtpSender = "VK-SBIINB"
        val testBankOtpMessage = "Your SBI OTP is 492018 for transaction of Rs 5,000 to Amazon Pay. Valid for 10 mins. Do not share with anyone."

        // Fast classification check
        assertTrue("Message must be classified as sensitive banking OTP", interceptor.classifyFast(testBankOtpSender, testBankOtpMessage))

        // When screen is NOT shared: Notification passes through normally
        val normalResult = interceptor.interceptNotification("com.google.android.apps.messaging", testBankOtpSender, testBankOtpMessage)
        assertFalse("Banner must not be redacted when screen is private", normalResult.isRedacted)
        assertEquals(testBankOtpMessage, normalResult.displayText)
        assertFalse("TTS should be permitted when screen is private", normalResult.isTtsBlocked)
        assertTrue(interceptor.isTtsReadoutAllowed(testBankOtpSender, testBankOtpMessage))

        println("[IND_BENCHMARK] Step 1 (Private Screen): OTP notification displayed normally, TTS allowed.")

        // 2. SPRINT 11 DoD SCENARIO: Start screen-share (e.g. AnyDesk / TeamViewer active during a phone call)
        detector.setSimulatedState(true, true)
        println("[IND_BENCHMARK] Step 2: Screen-share activated during phone call (simulating remote assistance session).")

        // 3. Send test bank OTP SMS while screen sharing is active
        val interceptedResult = interceptor.interceptNotification(
            "com.google.android.apps.messaging",
            testBankOtpSender,
            testBankOtpMessage
        )

        // Verify:
        // A. Notification banner is REDACTED to the exact Ind Suraksha Shield security banner
        assertTrue("CRITICAL DoD: Banner must be redacted when screen is shared!", interceptedResult.isRedacted)
        assertEquals(
            "CRITICAL DoD: Must display exact Ind Suraksha Shield blackout warning banner",
            OtpNotificationInterceptor.REDACTED_OTP_BANNER,
            interceptedResult.displayText
        )

        println("[IND_BENCHMARK] Redacted Notification Banner Text:")
        println("[IND_BENCHMARK] '${interceptedResult.displayText}'")

        // B. TTS Readout is STRICTLY BLOCKED to prevent audio leakage
        assertTrue("CRITICAL DoD: TTS readout must be blocked", interceptedResult.isTtsBlocked)
        assertFalse(
            "CRITICAL DoD: isTtsReadoutAllowed must return false when screen is shared",
            interceptor.isTtsReadoutAllowed(testBankOtpSender, testBankOtpMessage)
        )

        // C. Verify telemetry counters
        assertEquals(1, interceptor.redactedOtpCount)
        assertTrue("Blocked TTS count must be at least 1", interceptor.blockedTtsCount >= 1)

        println("[IND_BENCHMARK] Telemetry: Intercepted=2, OtpDetected=2, Redacted=${interceptor.redactedOtpCount}, BlockedTts=${interceptor.blockedTtsCount}")
        println("[IND_BENCHMARK] Sprint 11 DoD PASSED: Screen-share OTP blackout and TTS readout block verified.")
    }

    @Test
    fun testIndianBankingOtpCorpus_AllRedactedDuringScreenShare() {
        detector.setSimulatedState(true, false)

        val testBankMessages = listOf(
            Pair("AD-HDFCBK", "482910 is your HDFC Bank NetBanking OTP for Rs 12,500. Do not share OTP with bank staff or callers."),
            Pair("VM-ICICIB", "Dear Customer, 918234 is your ICICI Bank verification code to authorize payment. Valid for 5 mins."),
            Pair("AXISBK", "OTP for Axis Bank Credit Card transaction of INR 3,499 at Flipkart is 394812."),
            Pair("PAYTM", "Use OTP 572910 to login to your Paytm account. Never share your OTP with anyone on call."),
            Pair("UIDAI", "Your Aadhaar OTP for authentication is 829104. Generated at 10:45 AM.")
        )

        for ((sender, message) in testBankMessages) {
            val result = interceptor.interceptNotification("com.google.android.apps.messaging", sender, message)
            assertTrue("Message from $sender must be classified as OTP", result.isOtp)
            assertTrue("Message from $sender must be redacted during screen share", result.isRedacted)
            assertEquals(OtpNotificationInterceptor.REDACTED_OTP_BANNER, result.displayText)
            assertFalse(interceptor.isTtsReadoutAllowed(sender, message))
        }

        assertEquals(5, interceptor.redactedOtpCount)
        println("[IND_BENCHMARK] Indian Banking OTP Corpus: 5/5 banking OTPs successfully redacted during screen share.")
    }

    @Test
    fun testNonOtpNotifications_NotRedactedDuringScreenShare() {
        detector.setSimulatedState(true, true)

        val nonOtpMessages = listOf(
            Pair("Swiggy", "Your Biryani is on the way! Delivery partner is 5 mins away."),
            Pair("WhatsApp", "Maa: Ghar kab aaoge beta?"),
            Pair("YouTube", "New upload from Geekyranjit: Moto G60 Full Review")
        )

        for ((sender, message) in nonOtpMessages) {
            val result = interceptor.interceptNotification("com.example.app", sender, message)
            assertFalse("Non-OTP message from $sender must NOT be redacted", result.isRedacted)
            assertEquals(message, result.displayText)
            assertFalse(result.isTtsBlocked)
        }

        assertEquals(0, interceptor.redactedOtpCount)
    }
}
