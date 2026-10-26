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

package indos.suraksha.screenshare;

import android.app.Notification;
import android.os.Bundle;
import android.service.notification.NotificationListenerService;
import android.service.notification.StatusBarNotification;
import android.util.Log;

import java.util.concurrent.atomic.AtomicInteger;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Screen-Share OTP Blackout & Banking Notification Guard (PRD v1.0.1 §4.3-C & Sprint 11).
 *
 * Intercepts incoming notifications:
 * 1. Fast regex classification for banking OTPs and payment credentials.
 * 2. If screen sharing is active (AnyDesk, TeamViewer, RustDesk, WhatsApp) or screen is mirrored during a call,
 *    replaces the notification banner with Ind Suraksha Warning:
 *    "[IND SURAKSHA SHIELD]: Banking OTP hidden — your screen is being shared. Hang up if an unknown caller asked you to share your screen."
 * 3. Strictly blocks Text-to-Speech (TTS) audio readout to prevent audio eavesdropping.
 */
public class OtpNotificationInterceptor extends NotificationListenerService {

    private static final String TAG = "OtpNotificationInterceptor";

    public static final String REDACTED_OTP_BANNER =
            "[IND SURAKSHA SHIELD]: Banking OTP hidden — your screen is being shared. Hang up if an unknown caller asked you to share your screen.";

    // Banking OTP & Sensitive Financial Credential Regex Pattern (PRD §4.3-C)
    private static final Pattern SENSITIVE_OTP_PATTERN = Pattern.compile(
            "(?i)\\b(?:OTP|PIN|CVV|Aadhaar|UPI|debit|credit|One Time Password|verification code)\\b"
    );

    // Numeric OTP token pattern (4 to 8 digits)
    private static final Pattern NUMERIC_OTP_PATTERN = Pattern.compile(
            "\\b[0-9]{4,8}\\b"
    );

    // Indian Bank & Financial Sender Suffixes
    private static final Pattern BANK_SENDER_PATTERN = Pattern.compile(
            "(?i)(?:SBI|HDFC|ICICI|AXIS|PNB|BOB|KOTAK|CANARA|UNIONB|PAYTM|NPCI|GOVT|UIDAI)"
    );

    private static volatile OtpNotificationInterceptor sInstance;

    public static OtpNotificationInterceptor getInstance() {
        if (sInstance == null) {
            synchronized (OtpNotificationInterceptor.class) {
                if (sInstance == null) {
                    sInstance = new OtpNotificationInterceptor();
                }
            }
        }
        return sInstance;
    }

    private final ScreenShareDetector mScreenShareDetector;

    // Instrumented Counters for Verification & Telemetry
    private final AtomicInteger mTotalInterceptedNotifications = new AtomicInteger(0);
    private final AtomicInteger mOtpDetectedCount = new AtomicInteger(0);
    private final AtomicInteger mRedactedOtpCount = new AtomicInteger(0);
    private final AtomicInteger mBlockedTtsCount = new AtomicInteger(0);

    public OtpNotificationInterceptor() {
        this(ScreenShareDetector.getInstance(null));
    }

    public OtpNotificationInterceptor(ScreenShareDetector screenShareDetector) {
        mScreenShareDetector = screenShareDetector;
        sInstance = this;
    }

    public static class RedactionResult {
        public final boolean isOtp;
        public final boolean isRedacted;
        public final String originalText;
        public final String displayText;
        public final boolean isTtsBlocked;
        public final String reason;

        public RedactionResult(
                boolean isOtp,
                boolean isRedacted,
                String originalText,
                String displayText,
                boolean isTtsBlocked,
                String reason
        ) {
            this.isOtp = isOtp;
            this.isRedacted = isRedacted;
            this.originalText = originalText;
            this.displayText = displayText;
            this.isTtsBlocked = isTtsBlocked;
            this.reason = reason;
        }

        @Override
        public String toString() {
            return "RedactionResult{" +
                    "isOtp=" + isOtp +
                    ", isRedacted=" + isRedacted +
                    ", displayText='" + displayText + '\'' +
                    ", isTtsBlocked=" + isTtsBlocked +
                    ", reason='" + reason + '\'' +
                    '}';
        }
    }

    /**
     * Fast classification checking if a message contains banking OTP / verification credentials.
     */
    public boolean classifyFast(String title, String text) {
        String combined = ((title != null ? title : "") + " " + (text != null ? text : "")).trim();
        if (combined.isEmpty()) {
            return false;
        }

        Matcher otpMatcher = SENSITIVE_OTP_PATTERN.matcher(combined);
        boolean hasOtpKeyword = otpMatcher.find();

        Matcher numMatcher = NUMERIC_OTP_PATTERN.matcher(combined);
        boolean hasNumericCode = numMatcher.find();

        // High-confidence match: sensitive keyword + numeric code or bank sender
        if (hasOtpKeyword && (hasNumericCode || BANK_SENDER_PATTERN.matcher(combined).find())) {
            return true;
        }

        // Also check if title or text explicitly says "OTP" or "verification code"
        return hasOtpKeyword;
    }

    /**
     * Intercepts a notification payload and enforces the Screen-Share OTP blackout policy.
     */
    public RedactionResult interceptNotification(String packageName, String title, String text) {
        mTotalInterceptedNotifications.incrementAndGet();

        boolean isOtp = classifyFast(title, text);
        if (isOtp) {
            mOtpDetectedCount.incrementAndGet();
        }

        boolean screenShared = (mScreenShareDetector != null) &&
                (mScreenShareDetector.isScreenShared() || mScreenShareDetector.isScreenSharedDuringCall());

        // Check if package is a screen sharing application
        boolean isScreenShareApp = (packageName != null) &&
                ScreenShareDetector.SCREEN_SHARING_PACKAGES.contains(packageName.toLowerCase());

        boolean shouldBlackout = isOtp && (screenShared || isScreenShareApp);

        if (shouldBlackout) {
            mRedactedOtpCount.incrementAndGet();
            mBlockedTtsCount.incrementAndGet();

            String reason = "SCREEN_SHARE_ACTIVE: Banking OTP hidden to protect against remote screen-share credential theft.";
            Log.w(TAG, "[OTP_BLACKOUT_ACTIVATED] " + reason);

            return new RedactionResult(
                    true,
                    true,
                    text,
                    REDACTED_OTP_BANNER,
                    true, // TTS Blocked
                    reason
            );
        }

        return new RedactionResult(
                isOtp,
                false,
                text,
                text,
                false,
                "ALLOW: Normal notification display."
        );
    }

    /**
     * Checks if Text-to-Speech (TTS) readout of a notification is permitted.
     * Guaranteed to return FALSE if screen sharing is active and notification contains an OTP.
     */
    public boolean isTtsReadoutAllowed(String title, String text) {
        boolean isOtp = classifyFast(title, text);
        boolean screenShared = (mScreenShareDetector != null) &&
                (mScreenShareDetector.isScreenShared() || mScreenShareDetector.isScreenSharedDuringCall());

        if (isOtp && screenShared) {
            mBlockedTtsCount.incrementAndGet();
            Log.w(TAG, "[TTS_READOUT_BLOCKED] Refusing to read out OTP over speaker while screen is shared.");
            return false;
        }
        return true;
    }

    @Override
    public void onNotificationPosted(StatusBarNotification sbn) {
        if (sbn == null) return;

        Notification notification = sbn.getNotification();
        if (notification == null) return;

        Bundle extras = notification.extras;
        if (extras == null) return;

        String title = extras.getString(Notification.EXTRA_TITLE, "");
        CharSequence textSeq = extras.getCharSequence(Notification.EXTRA_TEXT);
        String text = (textSeq != null) ? textSeq.toString() : "";
        String pkg = sbn.getPackageName();

        RedactionResult result = interceptNotification(pkg, title, text);
        if (result.isRedacted) {
            Log.w(TAG, "[ON_NOTIFICATION_POSTED] Redacted OTP from package: " + pkg);
            // In Android system server, we update the notification ticker and text
            extras.putString(Notification.EXTRA_TITLE, "[IND SURAKSHA SHIELD]");
            extras.putCharSequence(Notification.EXTRA_TEXT, result.displayText);
        }
    }

    public int getTotalInterceptedNotifications() {
        return mTotalInterceptedNotifications.get();
    }

    public int getOtpDetectedCount() {
        return mOtpDetectedCount.get();
    }

    public int getRedactedOtpCount() {
        return mRedactedOtpCount.get();
    }

    public int getBlockedTtsCount() {
        return mBlockedTtsCount.get();
    }

    public void resetCounters() {
        mTotalInterceptedNotifications.set(0);
        mOtpDetectedCount.set(0);
        mRedactedOtpCount.set(0);
        mBlockedTtsCount.set(0);
        Log.i(TAG, "[RESET] OTP Interceptor counters reset.");
    }
}
