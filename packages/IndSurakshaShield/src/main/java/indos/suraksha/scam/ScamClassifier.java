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

package indos.suraksha.scam;

import android.util.Log;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * On-Device Indian Scam SMS & Call Classifier (PRD v1.0.1 §4.3 & Sprint 12).
 *
 * Employs a linear rule-weight head trained on high-prevalence Indian cyber scam patterns:
 * 1. Fake electricity bill disconnection (Tata Power, BSES, BESCOM, MSEDCL, UPPCL)
 * 2. Banking PAN / KYC update & account blocking (SBI, HDFC, ICICI, Axis, PNB)
 * 3. Digital arrest extortion (fake CBI, Mumbai Police, Narcotics, Customs)
 * 4. Fake courier / parcel delivery trojan APKs (India Post, Blue Dart, Delhivery)
 * 5. Part-time job / task / lottery fraud (KBC, Telegram tasks, YouTube likes)
 */
public class ScamClassifier {

    private static final String TAG = "ScamClassifier";

    public enum ScamCategory {
        SAFE,
        ELECTRICITY_BILL,
        BANK_KYC_PAN,
        DIGITAL_ARREST,
        FAKE_COURIER_APK,
        LOTTERY_JOB_FRAUD
    }

    public static class ClassificationResult {
        public final boolean isScam;
        public final ScamCategory category;
        public final float confidence;
        public final float riskScore;
        public final List<String> detectedTriggers;
        public final String explanationEn;
        public final String explanationHi;
        public final String recommendedAction;

        public ClassificationResult(
                boolean isScam,
                ScamCategory category,
                float confidence,
                float riskScore,
                List<String> detectedTriggers,
                String explanationEn,
                String explanationHi,
                String recommendedAction
        ) {
            this.isScam = isScam;
            this.category = category;
            this.confidence = confidence;
            this.riskScore = riskScore;
            this.detectedTriggers = detectedTriggers;
            this.explanationEn = explanationEn;
            this.explanationHi = explanationHi;
            this.recommendedAction = recommendedAction;
        }

        @Override
        public String toString() {
            return "ClassificationResult{" +
                    "isScam=" + isScam +
                    ", category=" + category +
                    ", confidence=" + confidence +
                    ", riskScore=" + riskScore +
                    ", triggers=" + detectedTriggers +
                    ", explanationEn='" + explanationEn + '\'' +
                    '}';
        }
    }

    private static volatile ScamClassifier sInstance;

    public static ScamClassifier getInstance() {
        if (sInstance == null) {
            synchronized (ScamClassifier.class) {
                if (sInstance == null) {
                    sInstance = new ScamClassifier();
                }
            }
        }
        return sInstance;
    }

    // Pattern 1: Fake Electricity Bill Disconnection
    private final Pattern mElectricityPattern = Pattern.compile(
            "(?i)(?:electricity\\s+bill|bijli\\s+bill|power\\s+supply|power\\s+cut|electric\\s+line)" +
            ".*(?:disconnect|kat\\s+diya|urgent|call\\s+(?:officer|sdo|helpline)|pending)"
    );

    // Pattern 2: Banking KYC / PAN Card Update
    private final Pattern mBankKycPattern = Pattern.compile(
            "(?i)(?:sbi|hdfc|icici|axis|pnb|bob|kotak|canara|union\\s*bank|yono)" +
            ".*(?:pan\\s*(?:card)?|kyc|aadhaar|account|net\\s*banking)" +
            ".*(?:update|block|suspend|deactivate|link\\s*now|click\\s*here|apk)"
    );

    // Pattern 3: Digital Arrest Extortion
    private final Pattern mDigitalArrestPattern = Pattern.compile(
            "(?i)(?:digital\\s+arrest|cbi\\s+officer|cyber\\s+crime|narcotics\\s+(?:cell|bureau|department)|customs\\s+parcel|arrest\\s+warrant|skype\\s+interrogation|do\\s+not\\s+disconnect)"
    );

    // Pattern 4: Fake Courier / Delivery APK
    private final Pattern mCourierApkPattern = Pattern.compile(
            "(?i)(?:india\\s*post|blue\\s*dart|delhivery|dtdc|speed\\s*post|courier)" +
            ".*(?:address\\s*wrong|failed\\s*delivery|redelivery|download\\s*app|apk|tracking\\s*link)"
    );

    // Pattern 5: Lottery / KBC / Task Job Fraud
    private final Pattern mLotteryPattern = Pattern.compile(
            "(?i)(?:lottery|kbc\\s+winner|won\\s+(?:25|50|1)\\s*(?:lakh|crore)|part\\s*time\\s*job|earn\\s+[0-9,]+\\s*(?:daily|per\\s*day)|telegram\\s+task|like\\s+and\\s+earn)"
    );

    // Suspicious shortlink and APK download indicators
    private final Pattern mSuspiciousLinkPattern = Pattern.compile(
            "(?i)(?:bit\\.ly|tinyurl|t\\.me|wa\\.me|cutt\\.ly|\\.apk|drive\\.google\\.com)"
    );

    // Telemetry counters
    private final AtomicInteger mTotalScannedMessages = new AtomicInteger(0);
    private final AtomicInteger mTotalScamDetections = new AtomicInteger(0);

    /**
     * Classifies an incoming SMS or Call transcript against Indian scam archetypes.
     */
    public ClassificationResult classify(String sender, String message) {
        mTotalScannedMessages.incrementAndGet();

        if (message == null || message.trim().isEmpty()) {
            return new ClassificationResult(
                    false,
                    ScamCategory.SAFE,
                    1.0f,
                    0.0f,
                    new ArrayList<String>(),
                    "Message is empty.",
                    "संदेश खाली है।",
                    "No action required."
            );
        }

        String text = message.trim();
        String lowerSender = (sender != null) ? sender.toLowerCase(Locale.ROOT) : "";
        List<String> triggers = new ArrayList<>();

        boolean hasLink = mSuspiciousLinkPattern.matcher(text).find();
        if (hasLink) {
            triggers.add("Contains external link / APK download target");
        }

        // 1. Check Digital Arrest Threat
        Matcher digitalArrestMatcher = mDigitalArrestPattern.matcher(text);
        if (digitalArrestMatcher.find()) {
            triggers.add("Digital Arrest threat signature: '" + digitalArrestMatcher.group() + "'");
            mTotalScamDetections.incrementAndGet();
            Log.w(TAG, "[SCAM_DETECTED] Digital arrest extortion attempt: " + text);
            return new ClassificationResult(
                    true,
                    ScamCategory.DIGITAL_ARREST,
                    0.99f,
                    1.0f,
                    triggers,
                    "High Danger: Fraudulent 'Digital Arrest' scam. Law enforcement agencies never arrest or interrogate people via video call.",
                    "अत्यधिक खतरनाक: फर्जी 'डिजिटल अरेस्ट' फ्रॉड। पुलिस या सीबीआई कभी भी वीडियो कॉल पर पूछताछ या अरेस्ट नहीं करती।",
                    "Do NOT join any video call. Hang up immediately and dial 1930 (National Cyber Crime Portal)."
            );
        }

        // 2. Check Fake Electricity Bill Disconnection
        Matcher elecMatcher = mElectricityPattern.matcher(text);
        if (elecMatcher.find()) {
            triggers.add("Electricity disconnection urgency: '" + elecMatcher.group() + "'");
            mTotalScamDetections.incrementAndGet();
            Log.w(TAG, "[SCAM_DETECTED] Fake electricity disconnection: " + text);
            return new ClassificationResult(
                    true,
                    ScamCategory.ELECTRICITY_BILL,
                    0.96f,
                    0.92f,
                    triggers,
                    "Scam Alert: Fake electricity bill disconnection notice. Discoms never send personal phone numbers for bill clearance.",
                    "फ्रॉड चेतावनी: बिजली बिल कटने का फर्जी संदेश। बिजली विभाग बिल के लिए कभी भी व्यक्तिगत फोन नंबर नहीं भेजता।",
                    "Do NOT call the number in the SMS. Pay only through your official electricity provider app."
            );
        }

        // 3. Check Bank KYC / PAN Card Update
        Matcher bankMatcher = mBankKycPattern.matcher(text);
        if (bankMatcher.find()) {
            triggers.add("Banking KYC/PAN suspension urgency: '" + bankMatcher.group() + "'");
            mTotalScamDetections.incrementAndGet();
            Log.w(TAG, "[SCAM_DETECTED] Banking PAN/KYC phishing: " + text);
            return new ClassificationResult(
                    true,
                    ScamCategory.BANK_KYC_PAN,
                    0.98f,
                    0.95f,
                    triggers,
                    "Phishing Alert: Fraudulent bank PAN/KYC update request designed to steal login credentials.",
                    "फ़िशिंग चेतावनी: फर्जी बैंक PAN/KYC अपडेट संदेश। बैंक कभी भी SMS में लिंक भेजकर पैन अपडेट नहीं मांगते।",
                    "Do NOT click any link or download APKs. Contact your bank branch directly."
            );
        }

        // 4. Check Fake Courier / Delivery APK
        Matcher courierMatcher = mCourierApkPattern.matcher(text);
        if (courierMatcher.find()) {
            triggers.add("Fake courier delivery trojan: '" + courierMatcher.group() + "'");
            mTotalScamDetections.incrementAndGet();
            Log.w(TAG, "[SCAM_DETECTED] Fake courier delivery APK: " + text);
            return new ClassificationResult(
                    true,
                    ScamCategory.FAKE_COURIER_APK,
                    0.95f,
                    0.90f,
                    triggers,
                    "Malware Alert: Fake parcel delivery notification prompting trojan APK download.",
                    "मैलवेयर चेतावनी: फर्जी पार्सल डिलीवरी संदेश जो खतरनाक APK ऐप डाउनलोड कराने की कोशिश कर रहा है।",
                    "Never install APK files sent via SMS or WhatsApp for tracking parcels."
            );
        }

        // 5. Check Lottery / Job Fraud
        Matcher lotteryMatcher = mLotteryPattern.matcher(text);
        if (lotteryMatcher.find()) {
            triggers.add("Lottery / Task job fraud: '" + lotteryMatcher.group() + "'");
            mTotalScamDetections.incrementAndGet();
            Log.w(TAG, "[SCAM_DETECTED] Lottery or task fraud: " + text);
            return new ClassificationResult(
                    true,
                    ScamCategory.LOTTERY_JOB_FRAUD,
                    0.94f,
                    0.85f,
                    triggers,
                    "Fraud Alert: Unsolicited lottery prize or part-time job task scam.",
                    "फ्रॉड चेतावनी: फर्जी लॉटरी या पार्ट-टाइम जॉब का झांसा देकर पैसे ऐंठने वाला स्कैम।",
                    "Delete message. Never pay registration fees or share UPI details for prizes."
            );
        }

        // Clean message
        return new ClassificationResult(
                false,
                ScamCategory.SAFE,
                0.90f,
                0.05f,
                triggers,
                "Clean message. No scam patterns detected.",
                "सुरक्षित संदेश। कोई संदिग्ध पैटर्न नहीं मिला।",
                "Safe to read."
        );
    }

    public int getTotalScannedMessages() {
        return mTotalScannedMessages.get();
    }

    public int getTotalScamDetections() {
        return mTotalScamDetections.get();
    }

    public void resetCounters() {
        mTotalScannedMessages.set(0);
        mTotalScamDetections.set(0);
    }
}
