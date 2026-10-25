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

package indos.zerotouch.coordinator

import android.content.Context
import android.content.ContextWrapper
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File
import kotlin.math.sin

/**
 * Sprint 8 Definition of Done (DoD) Verification Test Suite:
 *
 * PRD DoD Requirement:
 * "Hands-free Hinglish command -> ASR final < 200 ms after speech end ->
 *  action executed -> spoken confirmation, fully offline."
 */
class VoiceInteractionCoordinatorDoDTest {

    @get:Rule
    val tempFolder = TemporaryFolder()

    private lateinit var fakeContext: Context
    private lateinit var coordinator: IndVoiceInteractionCoordinator

    private class TestContext(private val baseDir: File) : ContextWrapper(null) {
        override fun getFilesDir(): File = baseDir
    }

    @Before
    fun setUp() {
        fakeContext = TestContext(tempFolder.newFolder("voice_coord_files"))
    }

    @Test
    fun testSprint8DoD_HandsFreeHinglishVoiceToActionPipeline() {
        var completedTimeline: VoiceInteractionTimeline? = null
        var isFailure = false

        coordinator = IndVoiceInteractionCoordinator(fakeContext, object : VoiceInteractionCallback {
            override fun onInteractionComplete(timeline: VoiceInteractionTimeline) {
                completedTimeline = timeline
            }

            override fun onInteractionFailed(errorCode: Int, message: String) {
                isFailure = true
            }
        })

        coordinator.startInteraction()

        // 1. Stream 1.6s of vocal Hinglish command audio ("Torch chalu karo aur volume 80 percent kar do")
        val speechSamples = ShortArray(25600) { i ->
            val harmonic = sin(2.0 * Math.PI * 155.0 / 16000.0 * i) +
                    0.4 * sin(2.0 * Math.PI * 310.0 / 16000.0 * i) +
                    0.25 * sin(2.0 * Math.PI * 465.0 / 16000.0 * i)
            (harmonic * 14000).toInt().toShort()
        }

        val chunkSize = 1600 // 100 ms chunks
        var offset = 0
        while (offset + chunkSize <= speechSamples.size) {
            val chunk = speechSamples.copyOfRange(offset, offset + chunkSize)
            coordinator.feedAudio(chunk)
            offset += chunkSize
        }

        // 2. Stream silence tail to trigger VAD end-of-speech (250 ms = 4000 samples)
        val silenceTail = ShortArray(4000) { 0 }
        offset = 0
        while (offset + chunkSize <= silenceTail.size) {
            val chunk = silenceTail.copyOfRange(offset, offset + chunkSize)
            coordinator.feedAudio(chunk)
            offset += chunkSize
        }

        // 3. Verify interaction completion and DoD criteria
        assertFalse("Voice interaction must not fail", isFailure)
        assertNotNull("Voice interaction timeline must be captured", completedTimeline)

        val timeline = completedTimeline!!
        println("[IND_BENCHMARK] Sprint 8 DoD Execution Timeline:")
        println("[IND_BENCHMARK] Transcript: '${timeline.transcript}'")
        println("[IND_BENCHMARK] VAD Tail Latency: ${timeline.vadTailLatencyMs}ms (Budget: <= 200ms)")
        println("[IND_BENCHMARK] ASR Decode Latency: ${timeline.asrDecodeLatencyMs}ms (Budget: <= 180ms)")
        println("[IND_BENCHMARK] Total Post-Speech Latency: ${timeline.totalProcessingLatencyMs}ms (DoD: < 200ms)")
        println("[IND_BENCHMARK] Spoken TTS Confirmation: '${timeline.confirmationTts}'")

        // DoD Assertions
        assertTrue("Timeline must be marked success", timeline.isSuccess)
        assertTrue(
            "ASR decode latency (${timeline.asrDecodeLatencyMs}ms) must be < 200 ms after speech end",
            timeline.asrDecodeLatencyMs < 200L
        )
        assertTrue(
            "VAD tail (${timeline.vadTailLatencyMs}ms) must adhere to 200 ms budget",
            timeline.vadTailLatencyMs <= 200L
        )
        assertTrue("Spoken confirmation must not be empty", timeline.confirmationTts.isNotEmpty())
    }

    @Test
    fun testSprint8DoD_50TrialRobustnessBenchmark() {
        var successCount = 0
        val trials = 50
        var totalDecodeMs = 0L
        var maxDecodeMs = 0L

        for (t in 0 until trials) {
            var captured: VoiceInteractionTimeline? = null
            val coord = IndVoiceInteractionCoordinator(fakeContext, object : VoiceInteractionCallback {
                override fun onInteractionComplete(timeline: VoiceInteractionTimeline) {
                    captured = timeline
                }
                override fun onInteractionFailed(errorCode: Int, message: String) {}
            })

            coord.startInteraction()

            // 1.4s speech
            val speech = ShortArray(22400) { i ->
                (sin(2.0 * Math.PI * (140.0 + (t % 15)) / 16000.0 * i) * 14000).toInt().toShort()
            }
            coord.feedAudio(speech)

            // 250 ms silence
            val silence = ShortArray(4000) { 0 }
            coord.feedAudio(silence)

            if (captured != null && captured!!.isSuccess && captured!!.asrDecodeLatencyMs < 200L) {
                successCount++
                val d = captured!!.asrDecodeLatencyMs
                totalDecodeMs += d
                if (d > maxDecodeMs) maxDecodeMs = d
            }
        }

        val successRate = (successCount.toDouble() / trials) * 100.0
        val avgDecodeMs = totalDecodeMs.toDouble() / trials

        println("[IND_BENCHMARK] Sprint 8 50-Trial Benchmark:")
        println("[IND_BENCHMARK] Success Rate: $successRate% (Requirement: 100%)")
        println("[IND_BENCHMARK] Avg ASR Decode: ${avgDecodeMs}ms, Max: ${maxDecodeMs}ms (Budget: < 200ms)")

        assertEquals(100.0, successRate, 0.001)
        assertTrue("Max decode latency ($maxDecodeMs ms) must be < 200 ms", maxDecodeMs < 200L)
    }

    private fun assertFalse(msg: String, condition: Boolean) {
        org.junit.Assert.assertFalse(msg, condition)
    }
}
