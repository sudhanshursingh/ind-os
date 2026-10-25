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
 * Unit test suite for SileroVadDetector:
 * 1. Speech onset detection & probability accumulation.
 * 2. Invariant 2 end-of-speech silence hangover (L_VAD_tail = 200 ms).
 * 3. Transient noise suppression.
 */
class SileroVadDetectorTest {

    private lateinit var vadDetector: SileroVadDetector

    @Before
    fun setUp() {
        vadDetector = SileroVadDetector(
            speechThreshold = 0.50f,
            silenceHangoverMs = 200L // PRD Invariant 2: 200 ms
        )
    }

    @Test
    fun testSilenceDoesNotTriggerSpeech() {
        var speechStarted = false
        vadDetector.listener = object : VadListener {
            override fun onSpeechStart(timestampMs: Long) { speechStarted = true }
            override fun onSpeechOngoing(frameAudio: ShortArray, speechProbability: Float) {}
            override fun onSpeechEnd(timestampMs: Long, completeUtterance: ShortArray) {}
        }

        // Feed 1 second of silence
        val silentAudio = ShortArray(16000) { 0 }
        vadDetector.processSamples(silentAudio)

        assertFalse("Silence must not trigger speech onset", speechStarted)
        assertEquals(SileroVadDetector.VadState.SILENCE, vadDetector.currentState)
    }

    @Test
    fun testSpeechOnsetDetectionAndEndOfUtteranceTail() {
        var speechStartTriggered = false
        var speechEndTriggered = false
        var capturedUtteranceSize = 0

        vadDetector.listener = object : VadListener {
            override fun onSpeechStart(timestampMs: Long) {
                speechStartTriggered = true
            }

            override fun onSpeechOngoing(frameAudio: ShortArray, speechProbability: Float) {
                assertTrue(speechProbability >= 0.0f)
            }

            override fun onSpeechEnd(timestampMs: Long, completeUtterance: ShortArray) {
                speechEndTriggered = true
                capturedUtteranceSize = completeUtterance.size
            }
        }

        // 1. Synthesize 1.0s of vocal speech (pitch 150 Hz)
        val speechAudio = ShortArray(16000) { i ->
            val s = sin(2.0 * Math.PI * 150.0 / 16000.0 * i) + 0.5 * sin(2.0 * Math.PI * 300.0 / 16000.0 * i)
            (s * 15000).toInt().toShort()
        }

        vadDetector.processSamples(speechAudio)
        assertTrue("Speech onset must be triggered", speechStartTriggered)
        assertEquals(SileroVadDetector.VadState.SPEECH_ACTIVE, vadDetector.currentState)

        // 2. Synthesize silence hangover (250 ms = ~8 frames of 32 ms)
        val silenceTail = ShortArray(4000) { 0 }
        vadDetector.processSamples(silenceTail)

        assertTrue("End of speech must be triggered after 200 ms silence", speechEndTriggered)
        assertEquals(SileroVadDetector.VadState.SILENCE, vadDetector.currentState)
        assertTrue("Utterance buffer must contain captured speech samples", capturedUtteranceSize >= 16000)
    }

    @Test
    fun testTransientNoiseSuppression() {
        var speechStarted = false
        vadDetector.listener = object : VadListener {
            override fun onSpeechStart(timestampMs: Long) { speechStarted = true }
            override fun onSpeechOngoing(frameAudio: ShortArray, speechProbability: Float) {}
            override fun onSpeechEnd(timestampMs: Long, completeUtterance: ShortArray) {}
        }

        // A single 32 ms transient pop/click followed by silence
        val transientFrame = ShortArray(SileroVadDetector.FRAME_SIZE) { 20000.toShort() }
        vadDetector.processFrame(transientFrame)

        val silenceFrame = ShortArray(SileroVadDetector.FRAME_SIZE) { 0 }
        vadDetector.processFrame(silenceFrame)

        assertFalse("Single isolated transient frame must not trigger speech onset", speechStarted)
    }
}
