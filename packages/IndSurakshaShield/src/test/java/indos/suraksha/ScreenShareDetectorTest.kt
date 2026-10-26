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

import indos.suraksha.screenshare.ScreenShareDetector
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Unit tests verifying virtual mirror display detection and call state tracking.
 */
class ScreenShareDetectorTest {

    private lateinit var detector: ScreenShareDetector

    @Before
    fun setUp() {
        detector = ScreenShareDetector(null)
    }

    @Test
    fun testScreenShareStateTracking_SimulatedTransitions() {
        detector.setSimulatedState(false, false)
        assertFalse(detector.isScreenShared)
        assertFalse(detector.isInCall)
        assertFalse(detector.isScreenSharedDuringCall)

        // Enable screen share only
        detector.setSimulatedState(true, false)
        assertTrue(detector.isScreenShared)
        assertFalse(detector.isInCall)
        assertFalse("Screen share without call is not screen-share during call", detector.isScreenSharedDuringCall)

        // Enable active call while screen is shared
        detector.setSimulatedState(true, true)
        assertTrue(detector.isScreenShared)
        assertTrue(detector.isInCall)
        assertTrue("High risk: screen shared during active call", detector.isScreenSharedDuringCall)
    }

    @Test
    fun testStateChangeListenerCallback() {
        val callbackInvoked = AtomicBoolean(false)
        val sharedState = AtomicBoolean(false)
        val inCallState = AtomicBoolean(false)

        val listener = ScreenShareDetector.StateListener { isShared, inCall ->
            callbackInvoked.set(true)
            sharedState.set(isShared)
            inCallState.set(inCall)
        }

        detector.addListener(listener)
        detector.setSimulatedState(true, true)

        assertTrue("Listener callback must be invoked on state change", callbackInvoked.get())
        assertTrue(sharedState.get())
        assertTrue(inCallState.get())

        detector.removeListener(listener)
    }

    @Test
    fun testKnownScreenSharingPackagesCatalog() {
        val knownPackages = ScreenShareDetector.SCREEN_SHARING_PACKAGES
        assertTrue(knownPackages.contains("com.anydesk.anydeskandroid"))
        assertTrue(knownPackages.contains("com.teamviewer.host.market"))
        assertTrue(knownPackages.contains("com.carriez.rustdesk"))
        assertTrue(knownPackages.contains("com.whatsapp"))
    }
}
