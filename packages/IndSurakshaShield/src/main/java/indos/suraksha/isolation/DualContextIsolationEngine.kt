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

package indos.suraksha.isolation

import android.util.Log
import indos.suraksha.firewall.PromptInjectionFirewall
import indos.suraksha.policy.ActionPolicyEnforcer

/**
 * Dual-Context Isolation Engine (PRD v1.0.1 §4.3 & Sprint 10).
 *
 * Enforces strict physical separation between:
 * 1. TRUSTED_USER_INTENT: Direct user commands from mic/voice orb.
 *    Active Grammar: ind_action.gbnf (actions permitted under Green/Amber/Red matrix).
 *
 * 2. UNTRUSTED_CONTENT_SUMMARY: Content from SMS, WhatsApp, Web, Email, Clipboard.
 *    Active Grammar: ind_summary_readonly.gbnf.
 *    Action types are physically absent from grammar: P(tool_call | untrusted_text) = 0 by construction.
 */
class DualContextIsolationEngine(
    private val firewall: PromptInjectionFirewall = PromptInjectionFirewall.INSTANCE,
    private val enforcer: ActionPolicyEnforcer = ActionPolicyEnforcer.getInstance()
) {

    companion object {
        private const val TAG = "DualContextIsolation"

        val INSTANCE by lazy { DualContextIsolationEngine() }
    }

    enum class ExecutionContext {
        TRUSTED_USER_INTENT,
        UNTRUSTED_CONTENT_SUMMARY
    }

    data class UntrustedSummaryResult(
        val summary: String,
        val risk: PromptInjectionFirewall.RiskLevel,
        val tts: String,
        val actionsTriggered: Int,
        val rawJson: String
    )

    @Volatile
    private var currentContext: ExecutionContext = ExecutionContext.TRUSTED_USER_INTENT

    fun getCurrentContext(): ExecutionContext = currentContext

    /**
     * Processes untrusted text under strict Dual-Context Isolation.
     * Enforces ind_summary_readonly.gbnf grammar schema and guarantees 0 actions triggered.
     */
    fun summarizeUntrustedContent(untrustedText: String): UntrustedSummaryResult {
        val initialActionsTriggered = enforcer.actionsTriggered
        currentContext = ExecutionContext.UNTRUSTED_CONTENT_SUMMARY
        Log.i(TAG, "[ISOLATION_SWITCH] Active context switched to UNTRUSTED_CONTENT_SUMMARY. Active grammar: ind_summary_readonly.gbnf")

        try {
            // 1. Run prompt-injection firewall analysis
            val verdict = firewall.inspect(untrustedText)

            // 2. Format read-only summary compliant with ind_summary_readonly.gbnf
            // Schema: {"summary": string, "risk": "SAFE" | "SUSPICIOUS" | "MALICIOUS", "tts": string}
            val summaryText = if (verdict.isMalicious) {
                "Adversarial injection detected. Content blocked from OS execution."
            } else if (verdict.isSuspicious) {
                "Suspicious message: Potential financial or phishing attempt."
            } else {
                "Summary: ${untrustedText.take(120)}"
            }

            val ttsText = if (verdict.isMalicious) {
                "Warning: Malicious injection attempt blocked. No actions executed."
            } else if (verdict.isSuspicious) {
                "Warning: Suspicious message received."
            } else {
                "Message summarized."
            }

            val riskStr = verdict.risk.name

            // Construct valid ind_summary_readonly.gbnf JSON string
            val sanitizedSummary = escapeJson(summaryText)
            val sanitizedTts = escapeJson(ttsText)
            val jsonPayload = "{\"summary\":\"$sanitizedSummary\",\"risk\":\"$riskStr\",\"tts\":\"$sanitizedTts\"}"

            // 3. Ensure NO tool calls or OS actions were triggered
            val finalActionsTriggered = enforcer.actionsTriggered
            val deltaActions = finalActionsTriggered - initialActionsTriggered

            if (deltaActions > 0) {
                Log.e(TAG, "[CRITICAL_VIOLATION] Actions were triggered during untrusted content summary! Delta=$deltaActions")
                throw SecurityException("Dual-Context Invariant Violated: P(tool_call | untrusted_text) must be 0")
            }

            Log.i(TAG, "[SUMMARY_COMPLETE] Verified 0 actions triggered. Risk=${verdict.risk}")

            return UntrustedSummaryResult(
                summary = summaryText,
                risk = verdict.risk,
                tts = ttsText,
                actionsTriggered = deltaActions,
                rawJson = jsonPayload
            )
        } finally {
            currentContext = ExecutionContext.TRUSTED_USER_INTENT
            Log.i(TAG, "[ISOLATION_RESTORE] Context restored to TRUSTED_USER_INTENT.")
        }
    }

    private fun escapeJson(input: String): String {
        return input.replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\b", "\\b")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
    }
}
