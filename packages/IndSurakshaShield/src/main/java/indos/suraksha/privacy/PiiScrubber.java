package indos.suraksha.privacy;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * High-Throughput On-Device PII Scrubber for Ind OS (PRD v1.0.1 §4.3 & Sprint 15).
 *
 * Enforces local redaction of sensitive Indian and international identifiers before any
 * transmission across the optional BYOK Cloud Bridge:
 * 1. Aadhaar Numbers (12-digit Indian national identity UID)
 * 2. Permanent Account Numbers (PAN Card 10-char alphanumeric)
 * 3. Indian & International Phone Numbers (+91 / standard 10-digit mobile)
 * 4. Bank Account Numbers (9 to 18 digits with contextual account labels)
 * 5. Credit & Debit Card Numbers (16 digits in 4-group blocks or continuous)
 * 6. UPI IDs / Virtual Payment Addresses (VPA: username@bank)
 * 7. Names with honorific titles (Mr., Mrs., Ms., Dr., Shri, Smt., Prof.)
 * 8. Passwords, PINs, and OTP verification codes
 * 9. Indian Postal PIN Codes (6-digit PIN codes)
 * 10. Email Addresses
 */
public class PiiScrubber {

    public static class ScrubResult {
        public final String originalText;
        public final String scrubbedText;
        public final int totalRedactions;
        public final Map<String, Integer> redactionCountsByType;
        public final boolean isFullyClean;

        public ScrubResult(String originalText, String scrubbedText, int totalRedactions,
                           Map<String, Integer> redactionCountsByType) {
            this.originalText = originalText;
            this.scrubbedText = scrubbedText;
            this.totalRedactions = totalRedactions;
            this.redactionCountsByType = Collections.unmodifiableMap(redactionCountsByType);
            this.isFullyClean = (totalRedactions > 0);
        }
    }

    // 1. Aadhaar Number Pattern: 12 digits (with optional spaces or dashes in groups of 4)
    // E.g. "1234 5678 9012", "1234-5678-9012", "123456789012"
    private static final Pattern AADHAAR_PATTERN = Pattern.compile(
            "\\b[2-9][0-9]{3}[\\s\\-]?[0-9]{4}[\\s\\-]?[0-9]{4}\\b"
    );

    // 2. PAN Card Pattern: 5 letters + 4 digits + 1 letter (e.g. ABCDE1234F)
    private static final Pattern PAN_PATTERN = Pattern.compile(
            "\\b[A-Z]{5}[0-9]{4}[A-Z]\\b"
    );

    // 3. Indian Phone Numbers: +91 or 0 followed by 10 digits starting with 6, 7, 8, or 9
    private static final Pattern PHONE_PATTERN = Pattern.compile(
            "(?:\\+91[\\-\\s]?|0)?[6-9][0-9]{4}[\\-\\s]?[0-9]{5}\\b"
    );

    // 4. Credit / Debit Card: 16 digits or 15-digit Amex
    private static final Pattern CARD_PATTERN = Pattern.compile(
            "\\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|6(?:011|5[0-9]{2})[0-9]{12}|3[47][0-9]{13}|3[47][0-9]{2}[\\s\\-][0-9]{6}[\\s\\-][0-9]{5}|3782[\\s\\-][0-9]{4}[\\s\\-][0-9]{4}[\\s\\-][0-9]{3}|[0-9]{4}[\\s\\-][0-9]{4}[\\s\\-][0-9]{4}[\\s\\-][0-9]{4})\\b"
    );

    // 5. Bank Account Numbers: 9 to 18 digits preceded by contextual keywords
    private static final Pattern BANK_ACCOUNT_PATTERN = Pattern.compile(
            "(?i)\\b(?:a/c|account|acct|acc)\\s*(?:no\\.?|number|#)?[:\\s]*([0-9]{9,18})\\b"
    );

    // 6. UPI ID / VPA: alphanumeric followed by @bank
    private static final Pattern UPI_VPA_PATTERN = Pattern.compile(
            "\\b[a-zA-Z0-9.\\-_]{2,50}@(okhdfcbank|okaxis|oksbi|okicici|paytm|ybl|ibl|upi|barodampay|axisbank|sbi|hdfcbank)\\b",
            Pattern.CASE_INSENSITIVE
    );

    // 7. Personal Names with Titles: (Mr., Mrs., Ms., Dr., Shri, Smt., Kumar) followed by 1-3 capitalized words
    private static final Pattern TITLED_NAME_PATTERN = Pattern.compile(
            "\\b(?:Mr\\.?|Mrs\\.?|Ms\\.?|Dr\\.?|Prof\\.?|Shri\\.?|Smt\\.?|Kumari)\\s+([A-Z][a-z]+(?:\\s+[A-Z][a-z]+){1,2})\\b"
    );

    // 8. Passwords, PINs, and OTPs with context
    private static final Pattern SECRET_PATTERN = Pattern.compile(
            "(?i)\\b(?:otp|pin|password|passcode|secret|security\\s*code|verification\\s*code)\\s*(?:is|code|:)?\\s*([0-9]{4,8}|[a-zA-Z0-9@#$%^&*]{6,16})\\b"
    );

    // 9. Indian Postal PIN Codes: 6 digits (110001, 400001, 560001) with context
    private static final Pattern PINCODE_PATTERN = Pattern.compile(
            "(?i)\\b(?:pincode|pin\\s*code|postal\\s*code)[:\\s]*([1-9][0-9]{5})\\b"
    );

    // 10. Email Addresses
    private static final Pattern EMAIL_PATTERN = Pattern.compile(
            "\\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}\\b"
    );

    private static volatile PiiScrubber sInstance;

    public static PiiScrubber getInstance() {
        if (sInstance == null) {
            synchronized (PiiScrubber.class) {
                if (sInstance == null) {
                    sInstance = new PiiScrubber();
                }
            }
        }
        return sInstance;
    }

    /**
     * Scrubs all sensitive PII identifiers from raw input text and replaces them with
     * strict anonymization tokens.
     */
    public ScrubResult scrub(String input) {
        if (input == null || input.isEmpty()) {
            return new ScrubResult("", "", 0, Collections.emptyMap());
        }

        String text = input;
        Map<String, Integer> counts = new LinkedHashMap<>();
        int total = 0;

        // 1. Scrub Card Numbers
        RedactResult res = redact(text, CARD_PATTERN, "[CARD_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("CARD", res.count);
            total += res.count;
        }

        // 2. Scrub Contextual Bank Accounts (BEFORE bare 12-digit numbers)
        res = redactContextual(text, BANK_ACCOUNT_PATTERN, "[BANK_ACCOUNT_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("BANK_ACCOUNT", res.count);
            total += res.count;
        }

        // 3. Scrub Contextual Passwords / OTPs
        res = redactContextual(text, SECRET_PATTERN, "[SECRET_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("SECRET_OTP", res.count);
            total += res.count;
        }

        // 4. Scrub Contextual PIN Codes
        res = redactContextual(text, PINCODE_PATTERN, "[PINCODE_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("PINCODE", res.count);
            total += res.count;
        }

        // 5. Scrub Aadhaar Numbers (12-digit Indian national UID)
        res = redact(text, AADHAAR_PATTERN, "[AADHAAR_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("AADHAAR", res.count);
            total += res.count;
        }

        // 6. Scrub PAN Cards
        res = redact(text, PAN_PATTERN, "[PAN_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("PAN", res.count);
            total += res.count;
        }

        // 7. Scrub UPI IDs
        res = redact(text, UPI_VPA_PATTERN, "[UPI_ID_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("UPI_ID", res.count);
            total += res.count;
        }

        // 8. Scrub Phone Numbers
        res = redact(text, PHONE_PATTERN, "[PHONE_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("PHONE", res.count);
            total += res.count;
        }

        // 9. Scrub Titled Names
        res = redactTitledNames(text, TITLED_NAME_PATTERN, "[NAME_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("NAME", res.count);
            total += res.count;
        }

        // 10. Scrub Email Addresses
        res = redact(text, EMAIL_PATTERN, "[EMAIL_REDACTED]");
        text = res.text;
        if (res.count > 0) {
            counts.put("EMAIL", res.count);
            total += res.count;
        }

        return new ScrubResult(input, text, total, counts);
    }

    /**
     * Checks if the given text contains any recognizable PII.
     */
    public boolean containsPii(String input) {
        if (input == null || input.isEmpty()) return false;
        return AADHAAR_PATTERN.matcher(input).find() ||
                PAN_PATTERN.matcher(input).find() ||
                PHONE_PATTERN.matcher(input).find() ||
                CARD_PATTERN.matcher(input).find() ||
                BANK_ACCOUNT_PATTERN.matcher(input).find() ||
                UPI_VPA_PATTERN.matcher(input).find() ||
                TITLED_NAME_PATTERN.matcher(input).find() ||
                SECRET_PATTERN.matcher(input).find() ||
                EMAIL_PATTERN.matcher(input).find();
    }

    private static class RedactResult {
        final String text;
        final int count;

        RedactResult(String text, int count) {
            this.text = text;
            this.count = count;
        }
    }

    private RedactResult redact(String text, Pattern pattern, String replacement) {
        Matcher matcher = pattern.matcher(text);
        StringBuffer sb = new StringBuffer();
        int count = 0;
        while (matcher.find()) {
            matcher.appendReplacement(sb, Matcher.quoteReplacement(replacement));
            count++;
        }
        matcher.appendTail(sb);
        return new RedactResult(sb.toString(), count);
    }

    private RedactResult redactContextual(String text, Pattern pattern, String replacement) {
        Matcher matcher = pattern.matcher(text);
        StringBuffer sb = new StringBuffer();
        int count = 0;
        while (matcher.find()) {
            String fullMatch = matcher.group();
            String sensitiveGroup = matcher.group(1);
            if (sensitiveGroup != null) {
                String replaced = fullMatch.replace(sensitiveGroup, replacement);
                matcher.appendReplacement(sb, Matcher.quoteReplacement(replaced));
                count++;
            } else {
                matcher.appendReplacement(sb, Matcher.quoteReplacement(replacement));
                count++;
            }
        }
        matcher.appendTail(sb);
        return new RedactResult(sb.toString(), count);
    }

    private RedactResult redactTitledNames(String text, Pattern pattern, String replacement) {
        Matcher matcher = pattern.matcher(text);
        StringBuffer sb = new StringBuffer();
        int count = 0;
        while (matcher.find()) {
            String fullMatch = matcher.group();
            String nameGroup = matcher.group(1);
            if (nameGroup != null) {
                String replaced = fullMatch.replace(nameGroup, replacement);
                matcher.appendReplacement(sb, Matcher.quoteReplacement(replaced));
                count++;
            } else {
                matcher.appendReplacement(sb, Matcher.quoteReplacement(replacement));
                count++;
            }
        }
        matcher.appendTail(sb);
        return new RedactResult(sb.toString(), count);
    }
}
