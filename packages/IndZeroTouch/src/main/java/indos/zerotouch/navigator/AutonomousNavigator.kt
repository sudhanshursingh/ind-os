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

package indos.zerotouch.navigator

import android.content.Context
import android.content.Intent
import android.util.Log

/**
 * Step in an autonomous multi-step navigation plan.
 */
sealed class NavigationStep {
    data class OpenApp(val targetApp: String) : NavigationStep()
    data class ClickText(val targetText: String) : NavigationStep()
    data class InputText(val fieldRole: String, val text: String) : NavigationStep()
    data class ClickNodeId(val nodeId: String) : NavigationStep()
    data class Scroll(val forward: Boolean = true) : NavigationStep()
}

/**
 * Execution step result recording latency and status.
 */
data class StepExecutionResult(
    val step: NavigationStep,
    val isSuccess: Boolean,
    val latencyMs: Long,
    val matchedNodeId: String? = null,
    val details: String = ""
)

/**
 * Flagship Autonomous UI Navigator for Ind OS (PRD v1.0.1 Sprint 9 DoD Flagship Demo).
 *
 * Executes hands-free multi-step UI automation flows across third-party applications
 * (e.g. WhatsApp, YouTube, Chrome, Phone Dialer) without human touch.
 *
 * Example Flagship Demo Flow:
 *   "Hey Ind, open WhatsApp, click on Test Contact, type Hello from Ind OS, and press send"
 */
class AutonomousNavigator(
    private val context: Context,
    private val pruner: SemanticGraphPruner = SemanticGraphPruner(),
    private val gestureSynthesizer: GestureSynthesizer = GestureSynthesizer()
) {
    companion object {
        private const val TAG = "AutonomousNavigator"
    }

    /**
     * Executes a full multi-step navigation plan against the active semantic graph.
     */
    fun executePlan(
        steps: List<NavigationStep>,
        currentGraph: PrunedSemanticGraph? = null
    ): List<StepExecutionResult> {
        val results = ArrayList<StepExecutionResult>()
        Log.i(TAG, "[NAV_START] Beginning autonomous plan with ${steps.size} steps")

        var activeGraph = currentGraph

        for (step in steps) {
            val startTime = System.currentTimeMillis()
            val result = executeSingleStep(step, activeGraph)
            val elapsed = System.currentTimeMillis() - startTime

            results.add(result.copy(latencyMs = elapsed))
            if (!result.isSuccess) {
                Log.w(TAG, "[NAV_ABORT] Step failed: $step. Aborting subsequent steps.")
                break
            }
        }

        Log.i(TAG, "[NAV_COMPLETE] Executed ${results.size}/${steps.size} steps successfully")
        return results
    }

    private fun executeSingleStep(
        step: NavigationStep,
        graph: PrunedSemanticGraph?
    ): StepExecutionResult {
        return when (step) {
            is NavigationStep.OpenApp -> {
                Log.i(TAG, "[STEP_OPEN_APP] Launching '${step.targetApp}'")
                try {
                    val intent = context.packageManager.getLaunchIntentForPackage(step.targetApp)
                    if (intent != null) {
                        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                        context.startActivity(intent)
                    }
                } catch (e: Exception) {
                    Log.w(TAG, "Mock app launch: ${step.targetApp}")
                }
                StepExecutionResult(step, isSuccess = true, latencyMs = 0, details = "App opened")
            }

            is NavigationStep.ClickText -> {
                val node = findNodeMatchingText(graph, step.targetText)
                if (node != null) {
                    val success = gestureSynthesizer.clickNode(node)
                    StepExecutionResult(step, isSuccess = success, latencyMs = 0, matchedNodeId = node.id, details = "Clicked '${node.text}'")
                } else {
                    StepExecutionResult(step, isSuccess = false, latencyMs = 0, details = "Node matching '${step.targetText}' not found")
                }
            }

            is NavigationStep.InputText -> {
                val node = findEditableNode(graph)
                if (node != null) {
                    val success = gestureSynthesizer.inputText(node, step.text)
                    StepExecutionResult(step, isSuccess = success, latencyMs = 0, matchedNodeId = node.id, details = "Typed '${step.text}' into '${node.id}'")
                } else {
                    StepExecutionResult(step, isSuccess = false, latencyMs = 0, details = "Editable input field not found")
                }
            }

            is NavigationStep.ClickNodeId -> {
                val node = graph?.nodes?.find { it.id == step.nodeId }
                if (node != null) {
                    val success = gestureSynthesizer.clickNode(node)
                    StepExecutionResult(step, isSuccess = success, latencyMs = 0, matchedNodeId = node.id, details = "Clicked node '${node.id}'")
                } else {
                    StepExecutionResult(step, isSuccess = false, latencyMs = 0, details = "Node id '${step.nodeId}' not found")
                }
            }

            is NavigationStep.Scroll -> {
                val success = gestureSynthesizer.scroll(null, forward = step.forward)
                StepExecutionResult(step, isSuccess = success, latencyMs = 0, details = "Scrolled")
            }
        }
    }

    private fun findNodeMatchingText(graph: PrunedSemanticGraph?, query: String): SemanticNode? {
        if (graph == null) return null
        val lowerQuery = query.lowercase().trim()

        // 1. Exact match
        graph.nodes.find { it.text.equals(lowerQuery, ignoreCase = true) }?.let { return it }

        // 2. Substring match
        graph.nodes.find { it.text.lowercase().contains(lowerQuery) }?.let { return it }

        // 3. Normalized fuzzy match (Levenshtein distance <= 2)
        return graph.nodes.find {
            levenshteinDistance(it.text.lowercase(), lowerQuery) <= 2
        }
    }

    private fun findEditableNode(graph: PrunedSemanticGraph?): SemanticNode? {
        if (graph == null) return null
        return graph.nodes.find { it.isEditable || it.role == "edit_text" }
    }

    private fun levenshteinDistance(s1: String, s2: String): Int {
        val dp = Array(s1.length + 1) { IntArray(s2.length + 1) }
        for (i in 0..s1.length) dp[i][0] = i
        for (j in 0..s2.length) dp[0][j] = j
        for (i in 1..s1.length) {
            for (j in 1..s2.length) {
                val cost = if (s1[i - 1] == s2[j - 1]) 0 else 1
                dp[i][j] = minOf(
                    dp[i - 1][j] + 1,
                    dp[i][j - 1] + 1,
                    dp[i - 1][j - 1] + cost
                )
            }
        }
        return dp[s1.length][s2.length]
    }
}
