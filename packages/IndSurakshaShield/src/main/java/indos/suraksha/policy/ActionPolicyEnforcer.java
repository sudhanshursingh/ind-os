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

package indos.suraksha.policy;

import android.util.Log;

import java.util.Arrays;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * Hardcoded Green/Amber/Red Action Security Enforcer (PRD v1.0.1 §4.3-B & Sprint 10).
 *
 * CRITICAL SECURITY INVARIANT:
 * 1. The LLM's self-reported privilege tier is NEVER trusted.
 * 2. Clicks or virtual key injection on UPI PIN pads / payment confirmation dialogs are AUTO-BLOCKED.
 *    The AI agent can never click final "Pay/Submit" on UPI PIN screens under any circumstance.
 */
public class ActionPolicyEnforcer {

    private static final String TAG = "ActionPolicyEnforcer";

    public static final float GREEN_BIOMETRIC_THRESHOLD = 0.72f;
    public static final float AMBER_BIOMETRIC_THRESHOLD = 0.78f;
    public static final float RED_BIOMETRIC_THRESHOLD = 0.85f;

    // Singleton instance
    private static volatile ActionPolicyEnforcer sInstance;

    public static ActionPolicyEnforcer getInstance() {
        if (sInstance == null) {
            synchronized (ActionPolicyEnforcer.class) {
                if (sInstance == null) {
                    sInstance = new ActionPolicyEnforcer();
                }
            }
        }
        return sInstance;
    }

    // Instrumented Counters for Verification & Telemetry
    private final AtomicInteger mTotalEvaluatedActions = new AtomicInteger(0);
    private final AtomicInteger mAllowedActions = new AtomicInteger(0);
    private final AtomicInteger mBlockedActions = new AtomicInteger(0);
    private final AtomicInteger mBlockedUpiAttempts = new AtomicInteger(0);
    private final AtomicInteger mActionsTriggered = new AtomicInteger(0);

    // Known Indian UPI & BFSI Banking App Packages
    private static final Set<String> BFSI_UPI_PACKAGES = Collections.unmodifiableSet(new HashSet<>(Arrays.asList(
            "in.org.npci.upiapp",                       // BHIM UPI
            "com.google.android.apps.nbu.paisa.user",   // Google Pay (Tez)
            "com.phonepe.app",                          // PhonePe
            "net.one97.paytm",                          // Paytm
            "com.sbi.upi",                              // SBI UPI
            "com.sbi.lotusintouch",                     // YONO SBI
            "com.icicibank.imobile",                    // iMobile ICICI
            "com.msf.mptba",                            // HDFC Bank MobileBanking
            "com.axis.mobile",                          // Axis Mobile
            "com.kotak.kotakpay",                       // Kotak 811 / Kotak Pay
            "com.msf.kbank.mobile",                     // Kotak Mobile Banking
            "com.bankofbaroda.mconnect",                // bob World
            "com.pnb.pnbone",                           // PNB ONE
            "com.cred.android",                         // CRED
            "com.freecharge.android",                   // Freecharge
            "com.mobikwik_new"                          // MobiKwik
    )));

    // Sensitive UI Keywords indicating UPI PIN entry or payment approval
    private static final String[] SENSITIVE_PAYMENT_KEYWORDS = {
            "upi pin",
            "enter upi pin",
            "enter 4-digit upi pin",
            "enter 6-digit upi pin",
            "mpin",
            "atm pin",
            "cvv",
            "debit card pin",
            "net banking password",
            "confirm payment",
            "approve payment",
            "authorize payment",
            "pay ₹",
            "paying ₹",
            "transfer ₹",
            "send ₹",
            "payment confirmation"
    };

    public enum Tier {
        GREEN,
        AMBER,
        RED
    }

    public static class ActionRequest {
        public final String actionType;
        public final String target;
        public final Map<String, Object> params;
        public final float speakerScore;
        public final boolean isVoiceConfirmed;
        public final boolean isFingerprintAuthenticated;
        public final boolean isPasscodeVerified;
        public final String uiNodeText;
        public final String targetPackage;

        public ActionRequest(
                String actionType,
                String target,
                Map<String, Object> params,
                float speakerScore,
                boolean isVoiceConfirmed,
                boolean isFingerprintAuthenticated,
                boolean isPasscodeVerified,
                String uiNodeText,
                String targetPackage
        ) {
            this.actionType = (actionType != null) ? actionType.toUpperCase(Locale.ROOT) : "";
            this.target = (target != null) ? target : "";
            this.params = (params != null) ? params : Collections.<String, Object>emptyMap();
            this.speakerScore = speakerScore;
            this.isVoiceConfirmed = isVoiceConfirmed;
            this.isFingerprintAuthenticated = isFingerprintAuthenticated;
            this.isPasscodeVerified = isPasscodeVerified;
            this.uiNodeText = (uiNodeText != null) ? uiNodeText : "";
            this.targetPackage = (targetPackage != null) ? targetPackage : "";
        }
    }

    public static class EnforcementDecision {
        public final boolean isAllowed;
        public final Tier trueTier;
        public final String reason;
        public final boolean isUpiBlocked;
        public final boolean requiresVoiceConfirmation;
        public final boolean requiresPasscodeOrFingerprint;

        public EnforcementDecision(
                boolean isAllowed,
                Tier trueTier,
                String reason,
                boolean isUpiBlocked,
                boolean requiresVoiceConfirmation,
                boolean requiresPasscodeOrFingerprint
        ) {
            this.isAllowed = isAllowed;
            this.trueTier = trueTier;
            this.reason = reason;
            this.isUpiBlocked = isUpiBlocked;
            this.requiresVoiceConfirmation = requiresVoiceConfirmation;
            this.requiresPasscodeOrFingerprint = requiresPasscodeOrFingerprint;
        }

        @Override
        public String toString() {
            return "EnforcementDecision{" +
                    "isAllowed=" + isAllowed +
                    ", trueTier=" + trueTier +
                    ", reason='" + reason + '\'' +
                    ", isUpiBlocked=" + isUpiBlocked +
                    ", requiresVoiceConfirmation=" + requiresVoiceConfirmation +
                    ", requiresPasscodeOrFingerprint=" + requiresPasscodeOrFingerprint +
                    '}';
        }
    }

    /**
     * Determines whether the requested target represents a UPI PIN screen or payment confirmation dialog.
     */
    public boolean isUpiOrPaymentConfirmation(String targetPackage, String uiNodeText, String actionType) {
        String lowerPkg = (targetPackage != null) ? targetPackage.toLowerCase(Locale.ROOT).trim() : "";
        String lowerText = (uiNodeText != null) ? uiNodeText.toLowerCase(Locale.ROOT).trim() : "";
        String upperAction = (actionType != null) ? actionType.toUpperCase(Locale.ROOT).trim() : "";

        // Explicit action types
        if (upperAction.contains("UPI_PIN") ||
            upperAction.contains("PAYMENT_CONFIRM") ||
            upperAction.contains("ENTER_PIN")) {
            return true;
        }

        // Target package matches known UPI / BFSI banking app
        boolean isBfsiPkg = BFSI_UPI_PACKAGES.contains(lowerPkg);

        // Check sensitive payment / PIN keywords
        for (String kw : SENSITIVE_PAYMENT_KEYWORDS) {
            if (lowerText.contains(kw)) {
                return true;
            }
        }

        // If it's a BFSI package and the user attempts a click on "Pay" or "Confirm"
        if (isBfsiPkg) {
            if (lowerText.contains("pay") ||
                lowerText.contains("confirm") ||
                lowerText.contains("submit") ||
                lowerText.contains("proceed") ||
                lowerText.contains("send money") ||
                lowerText.contains("pin")) {
                return true;
            }
        }

        return false;
    }

    /**
     * Classifies an action into its TRUE security tier (§4.3-B), ignoring any self-reported tier.
     */
    public Tier classifyAction(String actionType, String target, String uiNodeText, String targetPackage) {
        String type = (actionType != null) ? actionType.toUpperCase(Locale.ROOT) : "";
        String tgt = (target != null) ? target.toLowerCase(Locale.ROOT) : "";

        // First check for UPI or payment confirmation -> ALWAYS RED
        if (isUpiOrPaymentConfirmation(targetPackage, uiNodeText, type)) {
            return Tier.RED;
        }

        // RED Tier Actions
        if (type.equals("UNINSTALL_APP") ||
            type.equals("FACTORY_RESET") ||
            type.equals("MODIFY_SECURITY_SETTINGS") ||
            type.equals("GRANT_PERMISSION") ||
            type.equals("READ_OTP") ||
            type.equals("AUTOFILL_OTP") ||
            type.equals("CHANGE_PASSWORD") ||
            type.equals("DISABLE_AIR_GAP")) {
            return Tier.RED;
        }

        // AMBER Tier Actions
        if (type.equals("SEND_SMS") ||
            type.equals("SEND_WHATSAPP") ||
            type.equals("PLACE_CALL") ||
            type.equals("SHARE_FILE") ||
            type.equals("ACCESS_CONTACTS") ||
            type.equals("EXPORT_DATA")) {
            return Tier.AMBER;
        }

        if (type.equals("SYS_TOGGLE")) {
            if (tgt.equals("hotspot") || tgt.equals("location") || tgt.equals("gps") || tgt.equals("mobile_data")) {
                return Tier.AMBER;
            }
            // Green toggles
            if (tgt.equals("flashlight") || tgt.equals("torch") ||
                tgt.equals("wifi") || tgt.equals("bluetooth") ||
                tgt.equals("volume") || tgt.equals("brightness") ||
                tgt.equals("dnd") || tgt.equals("ringer")) {
                return Tier.GREEN;
            }
        }

        // OPEN_APP: check if financial app
        if (type.equals("OPEN_APP")) {
            if (BFSI_UPI_PACKAGES.contains(tgt) || BFSI_UPI_PACKAGES.contains(targetPackage.toLowerCase(Locale.ROOT))) {
                return Tier.RED;
            }
            return Tier.GREEN;
        }

        // GREEN Tier Actions
        if (type.equals("SET_ALARM") ||
            type.equals("READ_SCREEN") ||
            type.equals("READ_NOTIFICATIONS") ||
            type.equals("SCROLL") ||
            type.equals("CLICK_NODE") ||
            type.equals("INPUT_TEXT")) {
            return Tier.GREEN;
        }

        // Default to AMBER for unclassified operations as fail-safe
        return Tier.AMBER;
    }

    /**
     * Evaluates whether an action request satisfies the security matrix (§4.3-B).
     */
    public EnforcementDecision evaluateAction(ActionRequest request) {
        mTotalEvaluatedActions.incrementAndGet();

        // Check hardcoded UPI / Payment block invariant first
        if (isUpiOrPaymentConfirmation(request.targetPackage, request.uiNodeText, request.actionType)) {
            mBlockedUpiAttempts.incrementAndGet();
            mBlockedActions.incrementAndGet();
            String reason = "BLOCKED_UPI_SAFETY_INVARIANT: AI automation is strictly forbidden on UPI PIN pads or payment confirmations.";
            Log.e(TAG, "[VETO_AUTO_BLOCK] " + reason);
            return new EnforcementDecision(false, Tier.RED, reason, true, false, false);
        }

        // Determine true hardcoded tier
        Tier trueTier = classifyAction(request.actionType, request.target, request.uiNodeText, request.targetPackage);

        switch (trueTier) {
            case GREEN:
                // Instant execution with wake-word and S_cos >= 0.72
                if (request.speakerScore < GREEN_BIOMETRIC_THRESHOLD) {
                    mBlockedActions.incrementAndGet();
                    String reason = "BLOCKED_LOW_BIOMETRIC: Speaker score (" + request.speakerScore + ") < GREEN threshold (" + GREEN_BIOMETRIC_THRESHOLD + ").";
                    Log.w(TAG, "[VETO_GREEN] " + reason);
                    return new EnforcementDecision(false, Tier.GREEN, reason, false, false, false);
                }
                mAllowedActions.incrementAndGet();
                return new EnforcementDecision(true, Tier.GREEN, "ALLOWED: Green tier instant execution authorized.", false, false, false);

            case AMBER:
                // Voice confirmation loop + S_cos >= 0.78
                if (request.speakerScore < AMBER_BIOMETRIC_THRESHOLD) {
                    mBlockedActions.incrementAndGet();
                    String reason = "BLOCKED_LOW_BIOMETRIC: Speaker score (" + request.speakerScore + ") < AMBER threshold (" + AMBER_BIOMETRIC_THRESHOLD + ").";
                    Log.w(TAG, "[VETO_AMBER] " + reason);
                    return new EnforcementDecision(false, Tier.AMBER, reason, false, true, false);
                }
                if (!request.isVoiceConfirmed) {
                    mBlockedActions.incrementAndGet();
                    String reason = "AWAITING_VOICE_CONFIRMATION: Amber tier action requires spoken user confirmation loop.";
                    Log.i(TAG, "[AMBER_AWAITING] " + reason);
                    return new EnforcementDecision(false, Tier.AMBER, reason, false, true, false);
                }
                mAllowedActions.incrementAndGet();
                return new EnforcementDecision(true, Tier.AMBER, "ALLOWED: Amber tier action confirmed and verified.", false, false, false);

            case RED:
                // Physical fingerprint OR S_cos >= 0.85 + spoken 6-digit passcode
                boolean redAuthorized = request.isFingerprintAuthenticated ||
                        (request.speakerScore >= RED_BIOMETRIC_THRESHOLD && request.isPasscodeVerified);

                if (!redAuthorized) {
                    mBlockedActions.incrementAndGet();
                    String reason = "BLOCKED_RED_AUTH_REQUIRED: Red tier action requires physical fingerprint or S_cos >= 0.85 with 6-digit passcode.";
                    Log.w(TAG, "[VETO_RED] " + reason);
                    return new EnforcementDecision(false, Tier.RED, reason, false, false, true);
                }
                mAllowedActions.incrementAndGet();
                return new EnforcementDecision(true, Tier.RED, "ALLOWED: Red tier action authorized via strong authentication.", false, false, false);

            default:
                mBlockedActions.incrementAndGet();
                return new EnforcementDecision(false, Tier.RED, "BLOCKED: Unknown action tier fail-safe.", false, false, false);
        }
    }

    /**
     * Attempts to trigger an action on the operating system.
     * Increments mActionsTriggered ONLY if the action passes policy enforcement.
     */
    public boolean triggerAction(ActionRequest request) {
        EnforcementDecision decision = evaluateAction(request);
        if (decision.isAllowed) {
            mActionsTriggered.incrementAndGet();
            Log.i(TAG, "[ACTION_TRIGGERED] Dispatched action '" + request.actionType + "' for target '" + request.target + "'.");
            return true;
        } else {
            Log.w(TAG, "[ACTION_DENIED] Refused action '" + request.actionType + "': " + decision.reason);
            return false;
        }
    }

    public int getTotalEvaluatedActions() {
        return mTotalEvaluatedActions.get();
    }

    public int getAllowedActions() {
        return mAllowedActions.get();
    }

    public int getBlockedActions() {
        return mBlockedActions.get();
    }

    public int getBlockedUpiAttempts() {
        return mBlockedUpiAttempts.get();
    }

    public int getActionsTriggered() {
        return mActionsTriggered.get();
    }

    public void resetCounters() {
        mTotalEvaluatedActions.set(0);
        mAllowedActions.set(0);
        mBlockedActions.set(0);
        mBlockedUpiAttempts.set(0);
        mActionsTriggered.set(0);
        Log.i(TAG, "[RESET] Telemetry counters reset.");
    }

    public Map<String, Integer> getTelemetry() {
        Map<String, Integer> stats = new HashMap<>();
        stats.put("total_evaluated", mTotalEvaluatedActions.get());
        stats.put("allowed", mAllowedActions.get());
        stats.put("blocked", mBlockedActions.get());
        stats.put("blocked_upi", mBlockedUpiAttempts.get());
        stats.put("actions_triggered", mActionsTriggered.get());
        return stats;
    }
}
