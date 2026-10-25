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

package indos.zerotouch

import android.accessibilityservice.AccessibilityService
import android.util.Log
import android.view.accessibility.AccessibilityEvent
import indos.zerotouch.navigator.GestureSynthesizer
import indos.zerotouch.navigator.PrunedSemanticGraph
import indos.zerotouch.navigator.SemanticGraphPruner

/**
 * Privileged Accessibility Service capturing the live AccessibilityNodeInfo hierarchy
 * and injecting virtual gestures/actions for zero-touch ambient control (PRD v1.0.1 Sprint 9).
 */
class IndAccessibilityService : AccessibilityService() {

    companion object {
        private const val TAG = "IndAccessibility"

        @Volatile
        var instance: IndAccessibilityService? = null
            private set
    }

    val pruner = SemanticGraphPruner()
    lateinit var gestureSynthesizer: GestureSynthesizer

    var activePackageName: String = "com.android.launcher"
        private set
    var activeActivityName: String = ".MainActivity"
        private set

    override fun onServiceConnected() {
        super.onServiceConnected()
        instance = this
        gestureSynthesizer = GestureSynthesizer(this)
        Log.i(TAG, "[CONNECTED] IndAccessibilityService connected and ready for UI automation.")
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        if (event == null) return

        when (event.eventType) {
            AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED -> {
                val pkg = event.packageName?.toString()
                val cls = event.className?.toString()
                if (pkg != null) activePackageName = pkg
                if (cls != null) activeActivityName = cls
                Log.d(TAG, "[WINDOW_STATE] App: $activePackageName, Activity: $activeActivityName")
            }
        }
    }

    override fun onInterrupt() {
        Log.w(TAG, "[INTERRUPT] IndAccessibilityService interrupted.")
    }

    override fun onDestroy() {
        if (instance === this) {
            instance = null
        }
        super.onDestroy()
        Log.i(TAG, "[DESTROY] IndAccessibilityService destroyed.")
    }

    /**
     * Captures and prunes the active window's accessibility hierarchy into a compact JSON graph (< 350 tokens).
     */
    fun getCurrentSemanticGraph(): PrunedSemanticGraph {
        val root = try {
            rootInActiveWindow
        } catch (e: Exception) {
            Log.w(TAG, "Failed to retrieve rootInActiveWindow: ${e.message}")
            null
        }
        return pruner.pruneTree(root, activePackageName, activeActivityName)
    }

    /**
     * Executes virtual click on a node identified by ID ("n1", "n2", etc.)
     */
    fun clickNode(nodeId: String): Boolean {
        val node = pruner.getNodeById(nodeId)
        if (node == null) {
            Log.w(TAG, "Cannot click: node '$nodeId' not found in active registry")
            return false
        }
        return gestureSynthesizer.clickNode(node)
    }

    /**
     * Types text into an editable node identified by ID ("n1", "n2", etc.)
     */
    fun typeText(nodeId: String, text: String): Boolean {
        val node = pruner.getNodeById(nodeId)
        if (node == null) {
            Log.w(TAG, "Cannot input text: node '$nodeId' not found in active registry")
            return false
        }
        return gestureSynthesizer.inputText(node, text)
    }

    /**
     * Scrolls the view forward or backward.
     */
    fun scroll(nodeId: String? = null, forward: Boolean = true): Boolean {
        val node = if (nodeId != null) pruner.getNodeById(nodeId) else null
        return gestureSynthesizer.scroll(node, forward)
    }
}
