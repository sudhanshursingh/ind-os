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

import indos.suraksha.policy.ActionPolicyEnforcer
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Unit tests verifying the Hardcoded Green/Amber/Red Action Security Matrix (§4.3-B)
 * and the UPI PIN pad auto-block invariant.
 */
class ActionPolicyEnforcerTest {

    private lateinit var enforcer: ActionPolicyEnforcer

    @Before
    fun setUp() {
        enforcer = ActionPolicyEnforcer.getInstance()
        enforcer.resetCounters()
    }

    @Test
    fun testGreenTier_InstantExecutionWithSufficientBiometric() {
        val request = ActionPolicyEnforcer.ActionRequest(
            "SYS_TOGGLE",
            "flashlight",
            mapOf("value" to true),
            0.75f, // S_cos >= 0.72
            false,
            false,
            false,
            "Flashlight Toggle",
            "com.android.systemui"
        )

        val decision = enforcer.evaluateAction(request)
        assertEquals(ActionPolicyEnforcer.Tier.GREEN, decision.trueTier)
        assertTrue("Green action with S_cos 0.75 must be allowed", decision.isAllowed)

        val triggered = enforcer.triggerAction(request)
        assertTrue("Trigger action must succeed", triggered)
        assertEquals(1, enforcer.actionsTriggered)
    }

    @Test
    fun testGreenTier_BlockedOnInsufficientBiometric() {
        val request = ActionPolicyEnforcer.ActionRequest(
            "SYS_TOGGLE",
            "volume",
            mapOf("value" to 80),
            0.65f, // S_cos < 0.72
            false,
            false,
            false,
            "Volume Slider",
            "com.android.settings"
        )

        val decision = enforcer.evaluateAction(request)
        assertEquals(ActionPolicyEnforcer.Tier.GREEN, decision.trueTier)
        assertFalse("Green action with S_cos 0.65 must be blocked", decision.isAllowed)

        val triggered = enforcer.triggerAction(request)
        assertFalse(triggered)
        assertEquals(0, enforcer.actionsTriggered)
    }

    @Test
    fun testAmberTier_RequiresVoiceConfirmationLoop() {
        // Request without voice confirmation
        val unconfirmedRequest = ActionPolicyEnforcer.ActionRequest(
            "SEND_SMS",
            "+919876543210",
            mapOf("text" to "Hello"),
            0.80f, // S_cos >= 0.78
            false, // isVoiceConfirmed = false
            false,
            false,
            "Send Button",
            "com.google.android.apps.messaging"
        )

        val decision1 = enforcer.evaluateAction(unconfirmedRequest)
        assertEquals(ActionPolicyEnforcer.Tier.AMBER, decision1.trueTier)
        assertFalse("Amber action must be blocked if not confirmed", decision1.isAllowed)
        assertTrue("Must indicate voice confirmation required", decision1.requiresVoiceConfirmation)

        // Request WITH voice confirmation
        val confirmedRequest = ActionPolicyEnforcer.ActionRequest(
            "SEND_SMS",
            "+919876543210",
            mapOf("text" to "Hello"),
            0.80f, // S_cos >= 0.78
            true,  // isVoiceConfirmed = true
            false,
            false,
            "Send Button",
            "com.google.android.apps.messaging"
        )

        val decision2 = enforcer.evaluateAction(confirmedRequest)
        assertEquals(ActionPolicyEnforcer.Tier.AMBER, decision2.trueTier)
        assertTrue("Confirmed Amber action with S_cos 0.80 must be allowed", decision2.isAllowed)
    }

    @Test
    fun testRedTier_RequiresFingerprintOrHighBiometricAndPasscode() {
        val redRequestNoAuth = ActionPolicyEnforcer.ActionRequest(
            "FACTORY_RESET",
            "system",
            emptyMap(),
            0.80f,
            true,
            false,
            false,
            "Erase all data",
            "com.android.settings"
        )

        val decision1 = enforcer.evaluateAction(redRequestNoAuth)
        assertEquals(ActionPolicyEnforcer.Tier.RED, decision1.trueTier)
        assertFalse("Red action without strong auth must be blocked", decision1.isAllowed)

        // Authorized with fingerprint
        val redRequestFingerprint = ActionPolicyEnforcer.ActionRequest(
            "FACTORY_RESET",
            "system",
            emptyMap(),
            0.70f,
            true,
            true, // isFingerprintAuthenticated = true
            false,
            "Erase all data",
            "com.android.settings"
        )

        val decision2 = enforcer.evaluateAction(redRequestFingerprint)
        assertTrue("Red action with fingerprint must be allowed", decision2.isAllowed)

        // Authorized with S_cos >= 0.85 + passcode
        val redRequestPasscode = ActionPolicyEnforcer.ActionRequest(
            "FACTORY_RESET",
            "system",
            emptyMap(),
            0.87f, // S_cos >= 0.85
            true,
            false,
            true, // isPasscodeVerified = true
            "Erase all data",
            "com.android.settings"
        )

        val decision3 = enforcer.evaluateAction(redRequestPasscode)
        assertTrue("Red action with high biometric and passcode must be allowed", decision3.isAllowed)
    }

    @Test
    fun testNeverTrustLlmSelfReport_OverrideLlmGreenToAmberOrRed() {
        // Even if caller claims action is GREEN, enforcer re-evaluates
        val maliciousGreenClaim = ActionPolicyEnforcer.ActionRequest(
            "PLACE_CALL", // Inherently AMBER
            "911",
            emptyMap(),
            0.73f, // S_cos 0.73 is enough for GREEN, but NOT for AMBER (requires 0.78)
            false,
            false,
            false,
            "Call 911",
            "com.google.android.dialer"
        )

        val decision = enforcer.evaluateAction(maliciousGreenClaim)
        assertEquals("Must be classified as AMBER regardless of prompt context", ActionPolicyEnforcer.Tier.AMBER, decision.trueTier)
        assertFalse("Must be blocked because AMBER requires S_cos >= 0.78 and voice confirmation", decision.isAllowed)
    }

    @Test
    fun testAutoBlockUpiPinPadsAndPaymentConfirmations_AbsoluteProhibition() {
        // Test case 1: Target package is Google Pay (Tez) and clicking "Pay ₹500"
        val gpayClick = ActionPolicyEnforcer.ActionRequest(
            "CLICK_NODE",
            "pay_btn",
            emptyMap(),
            0.99f, // Even with maximum speaker score
            true,  // Even with voice confirmation
            true,  // Even with fingerprint
            true,  // Even with passcode
            "Pay ₹500",
            "com.google.android.apps.nbu.paisa.user"
        )

        val decision1 = enforcer.evaluateAction(gpayClick)
        assertFalse("AI click on UPI payment must be AUTO-BLOCKED", decision1.isAllowed)
        assertTrue("Decision must indicate UPI blocked", decision1.isUpiBlocked)
        assertEquals(1, enforcer.blockedUpiAttempts)

        // Test case 2: BHIM UPI screen asking "Enter UPI PIN"
        val bhimPinEntry = ActionPolicyEnforcer.ActionRequest(
            "INPUT_TEXT",
            "pin_field",
            mapOf("value" to "1234"),
            0.95f,
            true,
            true,
            true,
            "Enter 6-digit UPI PIN",
            "in.org.npci.upiapp"
        )

        val decision2 = enforcer.evaluateAction(bhimPinEntry)
        assertFalse("AI key injection on UPI PIN pad must be AUTO-BLOCKED", decision2.isAllowed)
        assertTrue(decision2.isUpiBlocked)
        assertEquals(2, enforcer.blockedUpiAttempts)

        // Test case 3: PhonePe payment confirmation button
        val phonePeConfirm = ActionPolicyEnforcer.ActionRequest(
            "CLICK_NODE",
            "confirm_transfer",
            emptyMap(),
            0.90f,
            true,
            true,
            true,
            "Confirm Payment",
            "com.phonepe.app"
        )

        val decision3 = enforcer.evaluateAction(phonePeConfirm)
        assertFalse("AI click on PhonePe confirm payment must be AUTO-BLOCKED", decision3.isAllowed)
        assertEquals(3, enforcer.blockedUpiAttempts)

        // Assert 0 UPI actions were triggered
        val triggerAttempt = enforcer.triggerAction(gpayClick)
        assertFalse(triggerAttempt)
        assertEquals(0, enforcer.actionsTriggered)
    }
}
