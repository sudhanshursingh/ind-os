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

package indos.suraksha.verifier;

import android.content.Context;
import android.content.Intent;
import android.util.Log;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * Privileged Malicious APK Interceptor & Package Verifier (PRD v1.0.1 §4.3-C & Sprint 12).
 *
 * Scans manifests of sideloaded applications:
 * Blocks applications requesting both:
 * (RECEIVE_SMS OR READ_SMS) AND BIND_ACCESSIBILITY_SERVICE
 * (The textbook signature of Indian BFSI banking trojans).
 */
public class SurakshaPackageVerifier {

    private static final String TAG = "SurakshaVerifier";

    public static final String PERM_RECEIVE_SMS = "android.permission.RECEIVE_SMS";
    public static final String PERM_READ_SMS = "android.permission.READ_SMS";
    public static final String PERM_BIND_ACCESSIBILITY = "android.permission.BIND_ACCESSIBILITY_SERVICE";

    private static volatile SurakshaPackageVerifier sInstance;

    public static SurakshaPackageVerifier getInstance() {
        if (sInstance == null) {
            synchronized (SurakshaPackageVerifier.class) {
                if (sInstance == null) {
                    sInstance = new SurakshaPackageVerifier();
                }
            }
        }
        return sInstance;
    }

    public static class ApkMetadata {
        public final String packageName;
        public final String appName;
        public final int versionCode;
        public final Set<String> requestedPermissions;
        public final boolean isSideloaded;
        public final boolean isTrustedStore;

        public ApkMetadata(
                String packageName,
                String appName,
                int versionCode,
                Set<String> requestedPermissions,
                boolean isSideloaded,
                boolean isTrustedStore
        ) {
            this.packageName = (packageName != null) ? packageName : "unknown.package";
            this.appName = (appName != null) ? appName : "Unknown App";
            this.versionCode = versionCode;
            this.requestedPermissions = (requestedPermissions != null) ? requestedPermissions : Collections.<String>emptySet();
            this.isSideloaded = isSideloaded;
            this.isTrustedStore = isTrustedStore;
        }
    }

    public static class VerificationVerdict {
        public final boolean isBlocked;
        public final String threatType;
        public final String titleEn;
        public final String titleHi;
        public final String messageEn;
        public final String messageHi;
        public final List<String> dangerousPermissions;
        public final String packageName;
        public final String appName;

        public VerificationVerdict(
                boolean isBlocked,
                String threatType,
                String titleEn,
                String titleHi,
                String messageEn,
                String messageHi,
                List<String> dangerousPermissions,
                String packageName,
                String appName
        ) {
            this.isBlocked = isBlocked;
            this.threatType = threatType;
            this.titleEn = titleEn;
            this.titleHi = titleHi;
            this.messageEn = messageEn;
            this.messageHi = messageHi;
            this.dangerousPermissions = dangerousPermissions;
            this.packageName = packageName;
            this.appName = appName;
        }

        @Override
        public String toString() {
            return "VerificationVerdict{" +
                    "isBlocked=" + isBlocked +
                    ", threatType='" + threatType + '\'' +
                    ", titleEn='" + titleEn + '\'' +
                    ", dangerousPermissions=" + dangerousPermissions +
                    ", packageName='" + packageName + '\'' +
                    '}';
        }
    }

    private final AtomicInteger mTotalScannedPackages = new AtomicInteger(0);
    private final AtomicInteger mBlockedTrojanPackages = new AtomicInteger(0);

    /**
     * Inspects an APK's requested permissions and origin.
     * Blocks installation if SMS + Accessibility combo signature is detected.
     */
    public VerificationVerdict verifyPackage(ApkMetadata metadata) {
        mTotalScannedPackages.incrementAndGet();

        if (metadata == null) {
            return new VerificationVerdict(
                    false,
                    "NONE",
                    "Package Valid",
                    "सुरक्षित ऐप",
                    "Package metadata clean.",
                    "ऐप सुरक्षित है।",
                    Collections.<String>emptyList(),
                    "unknown",
                    "Unknown"
            );
        }

        Set<String> perms = metadata.requestedPermissions;
        boolean hasSms = perms.contains(PERM_RECEIVE_SMS) || perms.contains(PERM_READ_SMS);
        boolean hasAccessibility = perms.contains(PERM_BIND_ACCESSIBILITY);

        List<String> dangerous = new ArrayList<>();
        if (perms.contains(PERM_RECEIVE_SMS)) dangerous.add(PERM_RECEIVE_SMS);
        if (perms.contains(PERM_READ_SMS)) dangerous.add(PERM_READ_SMS);
        if (perms.contains(PERM_BIND_ACCESSIBILITY)) dangerous.add(PERM_BIND_ACCESSIBILITY);

        // Indian Banking Trojan Signature Detection
        if (hasSms && hasAccessibility) {
            mBlockedTrojanPackages.incrementAndGet();

            String titleEn = "MALICIOUS APPLICATION BLOCKED";
            String titleHi = "खतरनाक ऐप को ब्लॉक कर दिया गया है";
            String messageEn = "This application requests both SMS reading and Accessibility Service control. This signature matches Indian banking trojans that steal OTPs and drain bank accounts.";
            String messageHi = "यह ऐप आपके SMS पढ़ने और फोन का पूरा नियंत्रण (Accessibility) लेने की अनुमति मांग रहा है। यह भारतीय बैंकिंग फ्रॉड ऐप का संकेत है जो OTP चुराकर पैसे निकाल लेते हैं।";

            Log.e(TAG, "[TROJAN_BLOCKED] " + titleEn + ": " + metadata.packageName + " (" + metadata.appName + ")");

            return new VerificationVerdict(
                    true,
                    "INDIAN_BANKING_TROJAN",
                    titleEn,
                    titleHi,
                    messageEn,
                    messageHi,
                    dangerous,
                    metadata.packageName,
                    metadata.appName
            );
        }

        return new VerificationVerdict(
                false,
                "NONE",
                "Application Safe",
                "सुरक्षित ऐप",
                "No trojan signatures detected.",
                "कोई खतरनाक अनुमति नहीं पाई गई।",
                dangerous,
                metadata.packageName,
                metadata.appName
        );
    }

    /**
     * Launches the full-screen red Ind Suraksha Block Dialog.
     */
    public void launchBlockDialog(Context context, VerificationVerdict verdict) {
        if (context == null || !verdict.isBlocked) return;

        Intent intent = new Intent(context, SurakshaBlockActivity.class);
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
        intent.putExtra("extra_package_name", verdict.packageName);
        intent.putExtra("extra_app_name", verdict.appName);
        intent.putExtra("extra_threat_type", verdict.threatType);
        intent.putExtra("extra_title_en", verdict.titleEn);
        intent.putExtra("extra_title_hi", verdict.titleHi);
        intent.putExtra("extra_message_en", verdict.messageEn);
        intent.putExtra("extra_message_hi", verdict.messageHi);
        intent.putStringArrayListExtra("extra_dangerous_perms", new ArrayList<>(verdict.dangerousPermissions));

        context.startActivity(intent);
        Log.i(TAG, "[BLOCK_DIALOG_LAUNCHED] Displaying full-screen red warning dialog for " + verdict.packageName);
    }

    public int getTotalScannedPackages() {
        return mTotalScannedPackages.get();
    }

    public int getBlockedTrojanPackages() {
        return mBlockedTrojanPackages.get();
    }

    public void resetCounters() {
        mTotalScannedPackages.set(0);
        mBlockedTrojanPackages.set(0);
    }
}
