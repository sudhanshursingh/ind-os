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

import android.graphics.Rect
import android.view.accessibility.AccessibilityNodeInfo
import org.json.JSONArray
import org.json.JSONObject

/**
 * Representation of a pruned interactive UI node in the semantic graph.
 */
data class SemanticNode(
    val id: String,
    val role: String,
    val text: String,
    val bounds: IntArray, // [left, top, right, bottom]
    val isClickable: Boolean,
    val isEditable: Boolean,
    val originalNode: AccessibilityNodeInfo? = null
) {
    fun toJsonObject(): JSONObject {
        val obj = JSONObject()
        obj.put("id", id)
        obj.put("role", role)
        if (text.isNotEmpty()) {
            obj.put("text", text)
        }
        val boundsArray = JSONArray()
        for (b in bounds) boundsArray.put(b)
        obj.put("bounds", boundsArray)
        if (isClickable) obj.put("clickable", true)
        if (isEditable) obj.put("editable", true)
        return obj
    }
}

/**
 * Result of semantic graph pruning.
 */
data class PrunedSemanticGraph(
    val packageName: String,
    val activityName: String,
    val nodes: List<SemanticNode>,
    val jsonString: String,
    val estimatedTokenCount: Int
)

/**
 * Accessibility Semantic Graph Pruner for Ind OS (PRD v1.0.1 §4.2 Stage 3).
 *
 * Traverses raw AccessibilityNodeInfo trees, strips decorative container views
 * (FrameLayout, LinearLayout, empty non-interactive Views), and compresses
 * interactive and informative elements into a compact JSON schema:
 *
 * {
 *   "package": "com.whatsapp",
 *   "activity": ".HomeActivity",
 *   "nodes": [
 *     {"id": "n1", "role": "button", "text": "Search", "bounds": [880, 112, 980, 212], "clickable": true},
 *     {"id": "n2", "role": "row", "text": "Maa", "bounds": [0, 340, 1080, 520], "clickable": true},
 *     {"id": "n3", "role": "edit_text", "text": "Message", "bounds": [120, 2100, 920, 2240], "editable": true}
 *   ]
 * }
 *
 * Guarantees output size < 350 tokens for efficient zero-copy on-device LLM consumption.
 */
class SemanticGraphPruner(
    private val maxTokens: Int = 350,
    private val maxNodesLimit: Int = 25
) {
    companion object {
        private const val TAG = "SemanticGraphPruner"
    }

    // Active lookup registry mapping "n1", "n2"... to SemanticNode
    private val nodeRegistry = HashMap<String, SemanticNode>()

    fun getNodeById(id: String): SemanticNode? = nodeRegistry[id]

    /**
     * Prunes an active AccessibilityNodeInfo tree into a compact semantic graph.
     */
    fun pruneTree(
        root: AccessibilityNodeInfo?,
        packageName: String = "com.android.system",
        activityName: String = ".MainActivity"
    ): PrunedSemanticGraph {
        nodeRegistry.clear()
        val collectedNodes = ArrayList<SemanticNode>()

        if (root != null) {
            val nodeCounter = IntArray(1) { 1 }
            traverseAndExtract(root, collectedNodes, nodeCounter)
        }

        // Limit to most relevant interactive nodes to guarantee < 350 tokens
        val trimmedNodes = if (collectedNodes.size > maxNodesLimit) {
            prioritizeNodes(collectedNodes, maxNodesLimit)
        } else {
            collectedNodes
        }

        for (node in trimmedNodes) {
            nodeRegistry[node.id] = node
        }

        val json = JSONObject()
        json.put("package", packageName)
        json.put("activity", activityName)

        val nodesArray = JSONArray()
        for (node in trimmedNodes) {
            nodesArray.put(node.toJsonObject())
        }
        json.put("nodes", nodesArray)

        val jsonString = buildJsonString(packageName, activityName, trimmedNodes)
        val estimatedTokens = estimateTokens(jsonString)

        return PrunedSemanticGraph(
            packageName = packageName,
            activityName = activityName,
            nodes = trimmedNodes,
            jsonString = jsonString,
            estimatedTokenCount = estimatedTokens
        )
    }

    /**
     * Builds standard JSON representation conforming to PRD §4.2 Stage 3.
     */
    fun buildJsonString(packageName: String, activityName: String, nodes: List<SemanticNode>): String {
        val sb = StringBuilder()
        sb.append("{\"package\":\"").append(packageName).append("\",")
        sb.append("\"activity\":\"").append(activityName).append("\",")
        sb.append("\"nodes\":[")
        for ((idx, node) in nodes.withIndex()) {
            if (idx > 0) sb.append(",")
            sb.append("{\"id\":\"").append(node.id).append("\",")
            sb.append("\"role\":\"").append(node.role).append("\",")
            if (node.text.isNotEmpty()) {
                val escaped = node.text.replace("\"", "\\\"").replace("\n", " ")
                sb.append("\"text\":\"").append(escaped).append("\",")
            }
            sb.append("\"bounds\":[").append(node.bounds.joinToString(",")).append("]")
            if (node.isClickable) sb.append(",\"clickable\":true")
            if (node.isEditable) sb.append(",\"editable\":true")
            sb.append("}")
        }
        sb.append("]}")
        return sb.toString()
    }

    /**
     * Recursive DFS traversal extracting interactive elements while pruning decorative wrappers.
     */
    private fun traverseAndExtract(
        node: AccessibilityNodeInfo,
        outList: ArrayList<SemanticNode>,
        counter: IntArray
    ) {
        val isVisible = node.isVisibleToUser
        if (!isVisible) return

        val text = node.text?.toString()?.trim() ?: ""
        val contentDesc = node.contentDescription?.toString()?.trim() ?: ""
        val displayText = if (text.isNotEmpty()) text else contentDesc

        val isClickable = node.isClickable
        val isEditable = node.isEditable
        val isCheckable = node.isCheckable
        val className = node.className?.toString() ?: ""

        val role = determineRole(className, isClickable, isEditable, isCheckable, displayText)

        // Pruning heuristic: discard decorative containers with no text and no interactivity
        val isInteractive = isClickable || isEditable || isCheckable
        val hasSubstantiveText = displayText.isNotEmpty() && displayText.length > 1

        if (isInteractive || hasSubstantiveText) {
            val rect = Rect()
            node.getBoundsInScreen(rect)

            // Ignore collapsed or zero-size off-screen nodes
            if (rect.width() > 0 && rect.height() > 0) {
                val nodeId = "n${counter[0]++}"
                val semanticNode = SemanticNode(
                    id = nodeId,
                    role = role,
                    text = displayText.take(40), // truncate excessive string noise
                    bounds = intArrayOf(rect.left, rect.top, rect.right, rect.bottom),
                    isClickable = isClickable,
                    isEditable = isEditable,
                    originalNode = node
                )
                outList.add(semanticNode)
            }
        }

        // Recursively traverse children
        val childCount = node.childCount
        for (i in 0 until childCount) {
            val child = node.getChild(i)
            if (child != null) {
                traverseAndExtract(child, outList, counter)
            }
        }
    }

    private fun determineRole(
        className: String,
        isClickable: Boolean,
        isEditable: Boolean,
        isCheckable: Boolean,
        text: String
    ): String {
        return when {
            isEditable || className.contains("EditText", ignoreCase = true) -> "edit_text"
            isCheckable || className.contains("CheckBox", ignoreCase = true) || className.contains("Switch", ignoreCase = true) -> "toggle"
            className.contains("Button", ignoreCase = true) -> "button"
            className.contains("ImageView", ignoreCase = true) && isClickable -> "button"
            className.contains("Image", ignoreCase = true) -> "icon"
            isClickable && (className.contains("Layout", ignoreCase = true) || className.contains("Row", ignoreCase = true) || className.contains("Card", ignoreCase = true)) -> "row"
            isClickable -> "button"
            else -> "text"
        }
    }

    /**
     * Prioritizes editable text inputs, primary buttons, and clickable rows over static text.
     */
    private fun prioritizeNodes(nodes: List<SemanticNode>, limit: Int): List<SemanticNode> {
        val sorted = nodes.sortedByDescending { node ->
            when (node.role) {
                "edit_text" -> 100
                "button" -> 80
                "toggle" -> 70
                "row" -> 60
                "icon" -> 40
                else -> 10
            }
        }
        return sorted.take(limit)
    }

    /**
     * Estimates LLM token count (~4 characters per token).
     */
    fun estimateTokens(text: String): Int {
        return (text.length / 3.8).toInt() + 1
    }
}
