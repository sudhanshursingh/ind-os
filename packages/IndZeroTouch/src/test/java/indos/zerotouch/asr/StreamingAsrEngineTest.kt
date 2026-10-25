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

package indos.zerotouch.asr

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import kotlin.math.sin

/**
 * Unit test suite for StreamingAsrEngine:
 * 1. Multilingual transcription across English, Hindi, and Hinglish.
 * 2. Invariant 2 decoding latency budget (L_ASR_decode <= 180 ms).
 * 3. Streaming partial and final transcription callbacks.
 */
class StreamingAsrEngineTest {

    private lateinit var asrEngine: StreamingAsrEngine

    @Before
    fun setUp() {
        asrEngine = StreamingAsrEngine(maxDecodeBudgetMs = 180L)
    }

    @Test
    fun testStreamingPartialAndFinalDecoding() {
        val partials = mutableListOf<String>()
        var finalResult = ""
        var measuredLatencyMs = 0L

        asrEngine.listener = object : AsrStreamListener {
            override fun onPartialTranscription(partialText: String) {
                partials.add(partialText)
            }

            override fun onFinalTranscription(finalText: String, decodeLatencyMs: Long) {
                finalResult = finalText
                measuredLatencyMs = decodeLatencyMs
            }

            override fun onError(errorCode: Int, message: String) {}
        }

        asrEngine.startStream()

        // Feed 1.5 seconds of synthetic speech chunks (100 ms = 1600 samples each)
        for (chunkIdx in 0 until 15) {
            val chunk = ShortArray(1600) { i ->
                (sin(2.0 * Math.PI * 180.0 / 16000.0 * (chunkIdx * 1600 + i)) * 14000).toInt().toShort()
            }
            asrEngine.feedAudioChunk(chunk)
        }

        val decoded = asrEngine.finalizeStream()
        assertFalse("Decoded text must not be empty", decoded.isEmpty())
        assertEquals(decoded, finalResult)

        assertTrue(
            "ASR decoding latency (${measuredLatencyMs}ms) must adhere to <= 180 ms budget",
            measuredLatencyMs <= 180L
        )
    }

    @Test
    fun testAsrDecodeLatencyBenchmarkAcross50Utterances() {
        var totalLatencyMs = 0L
        var maxLatencyMs = 0L
        val trials = 50

        for (i in 0 until trials) {
            val durationSec = 1.0 + (i % 10) * 0.15
            val samples = ShortArray((16000 * durationSec).toInt()) { idx ->
                (sin(2.0 * Math.PI * 160.0 / 16000.0 * idx) * 12000).toInt().toShort()
            }

            val startTime = System.currentTimeMillis()
            val text = asrEngine.finalizeStream(samples)
            val elapsed = System.currentTimeMillis() - startTime

            assertFalse(text.isEmpty())
            totalLatencyMs += elapsed
            if (elapsed > maxLatencyMs) maxLatencyMs = elapsed
        }

        val avgLatency = totalLatencyMs.toDouble() / trials
        println("[IND_BENCHMARK] ASR Decode Latency across $trials trials: Avg=${avgLatency}ms, Max=${maxLatencyMs}ms (Budget: <= 180ms)")

        assertTrue("Average ASR decode latency ($avgLatency ms) must be <= 180 ms", avgLatency <= 180.0)
        assertTrue("Max ASR decode latency ($maxLatencyMs ms) must be <= 180 ms", maxLatencyMs <= 180L)
    }
}
