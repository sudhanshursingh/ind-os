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

import indos.suraksha.screenshare.BankingWindowGuard
import indos.suraksha.screenshare.ScreenShareDetector
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Unit tests verifying Banking Window Guard behavior during active screen sharing.
 */
class BankingWindowGuardTest {

    private lateinit var detector: ScreenShareDetector
    private lateinit var guard: BankingWindowGuard

    @Before
    fun setUp() {
        detector = ScreenShareDetector(null)
        guard = BankingWindowGuard(detector)
        guard.reset()
    }

    @Test
    fun testBankingWindowProtectedWhenScreenIsShared() {
        // Screen share active
        detector.setSimulatedState(true, true)

        val yonoSbiVerdict = guard.evaluateWindowFocus("com.sbi.lotusintouch", ".HomeActivity")
        assertTrue("YONO SBI must be blocked from remote mirror", yonoSbiVerdict.isBlocked)
        assertTrue("Window must be blanked", yonoSbiVerdict.shouldBlankWindow)
        assertTrue("Must include warning message", yonoSbiVerdict.warningMessage.contains("Banking app launched while screen is shared"))

        val phonePeVerdict = guard.evaluateWindowFocus("com.phonepe.app", ".MainActivity")
        assertTrue("PhonePe must be blocked from remote mirror", phonePeVerdict.isBlocked)

        assertEquals(2, guard.guardedInterceptions)
    }

    @Test
    fun testBankingWindowAllowedWhenScreenIsPrivate() {
        // Private screen
        detector.setSimulatedState(false, false)

        val verdict = guard.evaluateWindowFocus("com.sbi.lotusintouch", ".HomeActivity")
        assertFalse("Banking window allowed when screen is not shared", verdict.isBlocked)
        assertFalse(verdict.shouldBlankWindow)
        assertEquals(0, guard.guardedInterceptions)
    }

    @Test
    fun testNonBankingWindowAllowedDuringScreenShare() {
        detector.setSimulatedState(true, true)

        val verdict = guard.evaluateWindowFocus("com.google.android.youtube", ".WatchActivity")
        assertFalse("YouTube allowed during screen share", verdict.isBlocked)
        assertFalse(verdict.shouldBlankWindow)
        assertEquals(0, guard.guardedInterceptions)
    }
}
