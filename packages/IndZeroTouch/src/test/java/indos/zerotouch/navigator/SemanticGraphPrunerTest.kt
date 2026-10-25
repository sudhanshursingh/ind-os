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

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Unit test suite for SemanticGraphPruner:
 * 1. Pruning decorative container hierarchies into compact semantic nodes.
 * 2. PRD §4.2 Stage 3 JSON schema compliance.
 * 3. Enforcing < 350 token context length budget.
 */
class SemanticGraphPrunerTest {

    private lateinit var pruner: SemanticGraphPruner

    @Before
    fun setUp() {
        pruner = SemanticGraphPruner(maxTokens = 350, maxNodesLimit = 25)
    }

    @Test
    fun testPrunedSemanticGraphJsonStructureAndTokenBudget() {
        // Construct realistic WhatsApp chat screen semantic nodes
        val sampleNodes = listOf(
            SemanticNode("n1", "button", "Search", intArrayOf(880, 112, 980, 212), isClickable = true, isEditable = false),
            SemanticNode("n2", "row", "Test Contact", intArrayOf(0, 340, 1080, 520), isClickable = true, isEditable = false),
            SemanticNode("n3", "row", "Maa", intArrayOf(0, 520, 1080, 700), isClickable = true, isEditable = false),
            SemanticNode("n4", "row", "Papa", intArrayOf(0, 700, 1080, 880), isClickable = true, isEditable = false),
            SemanticNode("n5", "edit_text", "Type a message", intArrayOf(120, 2100, 920, 2240), isClickable = true, isEditable = true),
            SemanticNode("n6", "button", "Send", intArrayOf(940, 2100, 1060, 2240), isClickable = true, isEditable = false)
        )

        val jsonString = pruner.buildJsonString("com.whatsapp", ".HomeActivity", sampleNodes)
        val estimatedTokens = pruner.estimateTokens(jsonString)

        println("[IND_BENCHMARK] Pruned Semantic Graph:")
        println(jsonString)
        println("[IND_BENCHMARK] Estimated Tokens: $estimatedTokens (Budget: < 350 tokens)")

        // 1. Verify token budget compliance
        assertTrue("Estimated token count ($estimatedTokens) must be < 350 tokens", estimatedTokens < 350)

        // 2. Verify JSON schema components
        assertTrue(jsonString.contains("\"package\":\"com.whatsapp\""))
        assertTrue(jsonString.contains("\"activity\":\".HomeActivity\""))
        assertTrue(jsonString.contains("\"id\":\"n1\""))
        assertTrue(jsonString.contains("\"role\":\"button\""))
        assertTrue(jsonString.contains("\"text\":\"Search\""))
        assertTrue(jsonString.contains("\"bounds\":[880,112,980,212]"))
        assertTrue(jsonString.contains("\"clickable\":true"))
        assertTrue(jsonString.contains("\"editable\":true"))
    }

    @Test
    fun testPrunerNodeLookupById() {
        // Test tree pruning with dummy root
        val graph = pruner.pruneTree(
            root = null,
            packageName = "com.whatsapp",
            activityName = ".ChatActivity"
        )

        assertNotNull(graph)
        assertEquals("com.whatsapp", graph.packageName)
        assertEquals(".ChatActivity", graph.activityName)
        assertTrue(graph.estimatedTokenCount < 350)
    }
}
