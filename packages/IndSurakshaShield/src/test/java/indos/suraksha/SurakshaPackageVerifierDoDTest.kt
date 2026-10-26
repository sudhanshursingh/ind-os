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

import indos.suraksha.verifier.SurakshaPackageVerifier
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Sprint 12 Definition of Done (DoD) Test:
 *
 * "Sideloaded simulated trojan APK triggers full-screen red Ind Suraksha block dialog
 * explaining dangerous permissions in English and Hindi."
 *
 * BIBLE Reference: PRD v1.0.1 §4.3-C & Sprint 12 (line 426).
 */
class SurakshaPackageVerifierDoDTest {

    private lateinit var verifier: SurakshaPackageVerifier

    @Before
    fun setUp() {
        verifier = SurakshaPackageVerifier.getInstance()
        verifier.resetCounters()
    }

    @Test
    fun testSprint12DoD_SideloadedTrojanApkBlockedWithBilingualExplanations() {
        println("[IND_BENCHMARK] ==================================================================")
        println("[IND_BENCHMARK] Starting Sprint 12 DoD Malicious Trojan APK Blocker Test")
        println("[IND_BENCHMARK] ==================================================================")

        // SPRINT 12 DoD SCENARIO:
        // Simulated sideloaded Indian banking trojan APK requesting both SMS reception and Accessibility Service.
        val trojanPermissions = setOf(
            "android.permission.INTERNET",
            "android.permission.RECEIVE_SMS",
            "android.permission.READ_SMS",
            "android.permission.BIND_ACCESSIBILITY_SERVICE",
            "android.permission.SYSTEM_ALERT_WINDOW"
        )

        val trojanMetadata = SurakshaPackageVerifier.ApkMetadata(
            "com.fake.sbi.rewards.updater",
            "SBI Rewards Updater (Sideloaded APK)",
            101,
            trojanPermissions,
            true,  // isSideloaded = true
            false  // isTrustedStore = false
        )

        // Verify package inspection
        val verdict = verifier.verifyPackage(trojanMetadata)

        // Assert 1: Installation is strictly blocked
        assertTrue("CRITICAL DoD: Sideloaded banking trojan must be BLOCKED!", verdict.isBlocked)
        assertEquals("INDIAN_BANKING_TROJAN", verdict.threatType)

        // Assert 2: Dangerous permissions intercepted contain both SMS and Accessibility
        assertTrue("Must intercept RECEIVE_SMS", verdict.dangerousPermissions.contains(SurakshaPackageVerifier.PERM_RECEIVE_SMS))
        assertTrue("Must intercept BIND_ACCESSIBILITY_SERVICE", verdict.dangerousPermissions.contains(SurakshaPackageVerifier.PERM_BIND_ACCESSIBILITY))

        // Assert 3: Title and explanation in English match PRD requirements
        assertEquals("MALICIOUS APPLICATION BLOCKED", verdict.titleEn)
        assertEquals(
            "This application requests both SMS reading and Accessibility Service control. This signature matches Indian banking trojans that steal OTPs and drain bank accounts.",
            verdict.messageEn
        )

        // Assert 4: Title and explanation in Hindi match PRD requirements
        assertEquals("खतरनाक ऐप को ब्लॉक कर दिया गया है", verdict.titleHi)
        assertEquals(
            "यह ऐप आपके SMS पढ़ने और फोन का पूरा नियंत्रण (Accessibility) लेने की अनुमति मांग रहा है। यह भारतीय बैंकिंग फ्रॉड ऐप का संकेत है जो OTP चुराकर पैसे निकाल लेते हैं।",
            verdict.messageHi
        )

        // Assert 5: Instrumented counter incremented
        assertEquals(1, verifier.blockedTrojanPackages)
        assertEquals(1, verifier.totalScannedPackages)

        println("[IND_BENCHMARK] Target App: ${verdict.appName} (${verdict.packageName})")
        println("[IND_BENCHMARK] Threat Detected: ${verdict.threatType}")
        println("[IND_BENCHMARK] English Title: '${verdict.titleEn}'")
        println("[IND_BENCHMARK] English Message: '${verdict.messageEn}'")
        println("[IND_BENCHMARK] Hindi Title: '${verdict.titleHi}'")
        println("[IND_BENCHMARK] Hindi Message: '${verdict.messageHi}'")
        println("[IND_BENCHMARK] Telemetry: Scanned=${verifier.totalScannedPackages}, BlockedTrojans=${verifier.blockedTrojanPackages}")
        println("[IND_BENCHMARK] Sprint 12 DoD PASSED: Trojan APK blocked with bilingual English & Hindi explanations.")
    }

    @Test
    fun testLegitimateApps_NotBlockedByVerifier() {
        // Legitimate messaging app: Has RECEIVE_SMS but NO BIND_ACCESSIBILITY_SERVICE
        val messagingAppMetadata = SurakshaPackageVerifier.ApkMetadata(
            "com.simplemobiletools.smsmessenger",
            "Simple SMS Messenger",
            54,
            setOf("android.permission.RECEIVE_SMS", "android.permission.READ_SMS"),
            true,
            true
        )

        val verdict1 = verifier.verifyPackage(messagingAppMetadata)
        assertFalse("Legitimate SMS app without accessibility must not be blocked", verdict1.isBlocked)

        // Legitimate accessibility tool: Has BIND_ACCESSIBILITY_SERVICE but NO SMS permissions
        val accessibilityToolMetadata = SurakshaPackageVerifier.ApkMetadata(
            "com.example.screenreader",
            "Screen Reader Utility",
            12,
            setOf("android.permission.BIND_ACCESSIBILITY_SERVICE"),
            true,
            true
        )

        val verdict2 = verifier.verifyPackage(accessibilityToolMetadata)
        assertFalse("Legitimate accessibility app without SMS permissions must not be blocked", verdict2.isBlocked)

        assertEquals(0, verifier.blockedTrojanPackages)
    }
}
