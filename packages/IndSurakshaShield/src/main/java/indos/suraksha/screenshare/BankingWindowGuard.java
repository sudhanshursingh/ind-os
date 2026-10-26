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

import android.util.Log;

import java.util.concurrent.atomic.AtomicInteger;

/**
 * Banking Window Guard (PRD v1.0.1 §4.3-C & Sprint 11).
 *
 * Prevents credential harvesting by actively guarding banking and UPI windows
 * when screen sharing or remote desktop applications are active.
 */
public class BankingWindowGuard {

    private static final String TAG = "BankingWindowGuard";

    private final ScreenShareDetector mDetector;
    private final AtomicInteger mGuardedInterceptions = new AtomicInteger(0);

    public BankingWindowGuard(ScreenShareDetector detector) {
        mDetector = detector;
    }

    public static class WindowGuardVerdict {
        public final boolean isBlocked;
        public final String warningMessage;
        public final boolean shouldBlankWindow;

        public WindowGuardVerdict(boolean isBlocked, String warningMessage, boolean shouldBlankWindow) {
            this.isBlocked = isBlocked;
            this.warningMessage = warningMessage;
            this.shouldBlankWindow = shouldBlankWindow;
        }
    }

    /**
     * Evaluates security posture when a banking or payment window is focused.
     */
    public WindowGuardVerdict evaluateWindowFocus(String targetPackage, String activityName) {
        boolean screenShared = (mDetector != null) && mDetector.isScreenShared();
        boolean inCall = (mDetector != null) && mDetector.isInCall();

        // Check if package is a banking or payment application
        boolean isBankingApp = isBankingPackage(targetPackage);

        if (isBankingApp && screenShared) {
            mGuardedInterceptions.incrementAndGet();
            String alert = "[IND SURAKSHA SHIELD]: Critical Security Warning — Banking app launched while screen is shared!";
            if (inCall) {
                alert += " An active call was detected. Scammers often use screen sharing to steal funds.";
            }
            Log.e(TAG, "[BANKING_GUARD_ALERT] " + alert + " (pkg=" + targetPackage + ")");

            return new WindowGuardVerdict(
                    true,
                    alert,
                    true // Window must be blanked from remote view
            );
        }

        return new WindowGuardVerdict(false, "Window secure.", false);
    }

    private boolean isBankingPackage(String packageName) {
        if (packageName == null) return false;
        String lower = packageName.toLowerCase();

        return lower.contains("upi") ||
               lower.contains("paisa") ||
               lower.contains("phonepe") ||
               lower.contains("paytm") ||
               lower.contains("sbi") ||
               lower.contains("hdfc") ||
               lower.contains("icici") ||
               lower.contains("axis") ||
               lower.contains("kotak") ||
               lower.contains("bank");
    }

    public int getGuardedInterceptions() {
        return mGuardedInterceptions.get();
    }

    public void reset() {
        mGuardedInterceptions.set(0);
    }
}
