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

package indos.suraksha.firewall

import android.util.Log
import java.util.Locale
import java.util.regex.Pattern

/**
 * On-Device Prompt-Injection Firewall (PRD v1.0.1 §4.3 & Sprint 10).
 *
 * Analyzes untrusted text (SMS, WhatsApp, web pages, clipboard, emails) to detect:
 * 1. Indirect Prompt Injections (system overrides, instruction resets, jailbreaks)
 * 2. Data Exfiltration attempts (forwarding SMS, OTPs, contacts, credentials)
 * 3. Known Indian Social Engineering / Scam patterns
 */
class PromptInjectionFirewall {

    companion object {
        private const val TAG = "PromptFirewall"

        val INSTANCE by lazy { PromptInjectionFirewall() }
    }

    enum class RiskLevel {
        SAFE,
        SUSPICIOUS,
        MALICIOUS
    }

    data class FirewallVerdict(
        val isMalicious: Boolean,
        val isSuspicious: Boolean,
        val risk: RiskLevel,
        val matchedPatterns: List<String>,
        val explanation: String
    )

    // Regex patterns for indirect prompt injection and privilege escalation
    private val adversarialOverridePatterns = listOf(
        Pattern.compile("(?i)system\\s+override"),
        Pattern.compile("(?i)ignore\\s+(all\\s+)?previous\\s+(rules|instructions|prompts|directives)"),
        Pattern.compile("(?i)disregard\\s+(all\\s+)?previous"),
        Pattern.compile("(?i)you\\s+are\\s+now\\s+in\\s+(developer|unrestricted|jailbreak|dan)\\s+mode"),
        Pattern.compile("(?i)new\\s+system\\s+prompt"),
        Pattern.compile("(?i)admin\\s+(access|override|bypass)"),
        Pattern.compile("(?i)root\\s+privilege"),
        Pattern.compile("(?i)bypass\\s+safety\\s+(checks|rules|filter)")
    )

    // Regex patterns for data exfiltration and credential harvesting
    private val exfiltrationPatterns = listOf(
        Pattern.compile("(?i)forward\\s+(all\\s+)?(sms|otp|messages)"),
        Pattern.compile("(?i)send\\s+(all\\s+)?(sms|otp|credentials)\\s+to"),
        Pattern.compile("(?i)(sms|otp)\\s+forwarding"),
        Pattern.compile("(?i)exfiltrat(e|ion)"),
        Pattern.compile("(?i)upload\\s+(contacts|call\\s+logs|sms|otp)"),
        Pattern.compile("(?i)dump\\s+(memory|keystore|keys|tokens)")
    )

    // Regex patterns for Indian financial scams & phishing
    private val indianScamPatterns = listOf(
        Pattern.compile("(?i)electricity\\s+bill.*(disconnect|power\\s+cut)"),
        Pattern.compile("(?i)(sbi|hdfc|icici|axis|pnb).*pan\\s*(card)?.*(update|block|suspend)"),
        Pattern.compile("(?i)digital\\s+arrest"),
        Pattern.compile("(?i)lottery.*(won|claim|crore|lakh)"),
        Pattern.compile("(?i)apk.*(download|install).*bill")
    )

    /**
     * Inspects untrusted text and returns a deterministic firewall verdict.
     */
    fun inspect(untrustedContent: String?): FirewallVerdict {
        if (untrustedContent.isNullOrBlank()) {
            return FirewallVerdict(
                isMalicious = false,
                isSuspicious = false,
                risk = RiskLevel.SAFE,
                matchedPatterns = emptyList(),
                explanation = "Empty text payload."
            )
        }

        val text = untrustedContent.trim()
        val matched = mutableListOf<String>()

        // 1. Check Adversarial Override
        for (pattern in adversarialOverridePatterns) {
            val matcher = pattern.matcher(text)
            if (matcher.find()) {
                matched.add("Adversarial Override: '${matcher.group()}'")
            }
        }

        // 2. Check Exfiltration
        for (pattern in exfiltrationPatterns) {
            val matcher = pattern.matcher(text)
            if (matcher.find()) {
                matched.add("Data Exfiltration: '${matcher.group()}'")
            }
        }

        // 3. Check Indian Scam
        for (pattern in indianScamPatterns) {
            val matcher = pattern.matcher(text)
            if (matcher.find()) {
                matched.add("Scam Signature: '${matcher.group()}'")
            }
        }

        if (matched.isNotEmpty()) {
            val hasDirectAttack = matched.any { it.startsWith("Adversarial Override") || it.startsWith("Data Exfiltration") }
            val risk = if (hasDirectAttack) RiskLevel.MALICIOUS else RiskLevel.SUSPICIOUS

            val verdict = FirewallVerdict(
                isMalicious = (risk == RiskLevel.MALICIOUS),
                isSuspicious = (risk == RiskLevel.SUSPICIOUS),
                risk = risk,
                matchedPatterns = matched,
                explanation = "Firewall alert: ${matched.joinToString("; ")}"
            )
            Log.w(TAG, "[PROMPT_FIREWALL_ALERT] $verdict")
            return verdict
        }

        return FirewallVerdict(
            isMalicious = false,
            isSuspicious = false,
            risk = RiskLevel.SAFE,
            matchedPatterns = emptyList(),
            explanation = "Content clean. No adversarial injection or scam signatures detected."
        )
    }
}
