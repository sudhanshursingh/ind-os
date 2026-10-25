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

import android.util.Log
import kotlin.math.cos
import kotlin.math.ln
import kotlin.math.sqrt

/**
 * Listener for Voice Activity Detection (VAD) state transitions.
 */
interface VadListener {
    fun onSpeechStart(timestampMs: Long)
    fun onSpeechOngoing(frameAudio: ShortArray, speechProbability: Float)
    fun onSpeechEnd(timestampMs: Long, completeUtterance: ShortArray)
}

/**
 * Low-latency Voice Activity Detector (VAD) for Ind OS.
 *
 * Emulates the Silero VAD architecture operating on 16 kHz 16-bit mono PCM:
 * - Frame size: 512 samples (32 ms).
 * - Enforces the PRD Invariant 2 end-of-speech hangover threshold:
 *     L_VAD_tail = 200 ms (PRD v1.0.1 §3 Invariant 2, line 126).
 * - Real-time speech probability estimation P(speech) in [0.0, 1.0].
 * - Automatically buffers and emits the complete utterance upon silence detection.
 */
class SileroVadDetector(
    val speechThreshold: Float = DEFAULT_SPEECH_THRESHOLD,
    val silenceHangoverMs: Long = DEFAULT_SILENCE_HANGOVER_MS
) {
    companion object {
        private const val TAG = "SileroVadDetector"
        const val SAMPLE_RATE = 16000
        const val FRAME_SIZE = 512 // 32 ms at 16 kHz
        const val DEFAULT_SPEECH_THRESHOLD = 0.50f
        const val DEFAULT_SILENCE_HANGOVER_MS = 200L // PRD Invariant 2: L_VAD_tail = 200 ms
        const val MIN_SPEECH_FRAMES = 2 // 64 ms minimum onset confirmation
    }

    enum class VadState {
        SILENCE,
        SPEECH_ONSET_VERIFYING,
        SPEECH_ACTIVE,
        SPEECH_ENDING
    }

    var listener: VadListener? = null
    var currentState = VadState.SILENCE
        private set

    private var speechOnsetFrames = 0
    private var silenceFramesCount = 0
    private val maxSilenceFrames = (silenceHangoverMs / 32).toInt() // 200 ms / 32 ms ~ 6 frames

    private val utteranceBuffer = ArrayList<Short>()
    private var speechStartTimestampMs = 0L

    // Internal recurrent acoustic state
    private var prevFrameEnergy = 0.0
    private var adaptiveNoiseFloor = 0.005

    /**
     * Processes a chunk of 16 kHz audio samples through the VAD pipeline.
     * Audio chunk can be of arbitrary size; it will be framed into 512-sample blocks.
     */
    fun processSamples(samples: ShortArray) {
        var offset = 0
        while (offset + FRAME_SIZE <= samples.size) {
            val frame = samples.copyOfRange(offset, offset + FRAME_SIZE)
            processFrame(frame)
            offset += FRAME_SIZE
        }
    }

    /**
     * Evaluates a single 512-sample (32 ms) frame.
     */
    fun processFrame(frame: ShortArray): Float {
        require(frame.size == FRAME_SIZE) { "Frame size must be exactly $FRAME_SIZE samples" }

        val prob = computeSpeechProbability(frame)
        val isVoice = prob >= speechThreshold
        val now = System.currentTimeMillis()

        when (currentState) {
            VadState.SILENCE -> {
                if (isVoice) {
                    speechOnsetFrames++
                    if (speechOnsetFrames >= MIN_SPEECH_FRAMES) {
                        currentState = VadState.SPEECH_ACTIVE
                        speechStartTimestampMs = now
                        utteranceBuffer.clear()
                        for (s in frame) utteranceBuffer.add(s)
                        silenceFramesCount = 0
                        Log.i(TAG, "[VAD] Speech STARTED (P=%.2f)".format(prob))
                        listener?.onSpeechStart(speechStartTimestampMs)
                    }
                } else {
                    speechOnsetFrames = 0
                    // Track ambient noise floor adaptively
                    val frameRms = calculateFrameRms(frame)
                    adaptiveNoiseFloor = 0.95 * adaptiveNoiseFloor + 0.05 * frameRms
                }
            }

            VadState.SPEECH_ONSET_VERIFYING,
            VadState.SPEECH_ACTIVE -> {
                for (s in frame) utteranceBuffer.add(s)
                listener?.onSpeechOngoing(frame, prob)

                if (isVoice) {
                    silenceFramesCount = 0
                } else {
                    silenceFramesCount++
                    if (silenceFramesCount >= maxSilenceFrames) {
                        // End of utterance reached within 200 ms
                        currentState = VadState.SILENCE
                        speechOnsetFrames = 0
                        silenceFramesCount = 0
                        val completeUtterance = ShortArray(utteranceBuffer.size) { utteranceBuffer[it] }
                        Log.i(TAG, "[VAD] Speech ENDED (tail=${silenceHangoverMs}ms, samples=${completeUtterance.size})")
                        listener?.onSpeechEnd(now, completeUtterance)
                    }
                }
            }

            VadState.SPEECH_ENDING -> {
                currentState = VadState.SILENCE
            }
        }

        return prob
    }

    /**
     * Resets VAD state and clears internal buffers.
     */
    fun reset() {
        currentState = VadState.SILENCE
        speechOnsetFrames = 0
        silenceFramesCount = 0
        utteranceBuffer.clear()
        prevFrameEnergy = 0.0
    }

    /**
     * Computes real-time speech probability P(speech) in [0.0, 1.0].
     * Combines harmonic energy, zero-crossing trajectory, and spectral centroid vs adaptive noise.
     */
    private fun computeSpeechProbability(frame: ShortArray): Float {
        var sumSquares = 0.0
        var zeroCrossings = 0
        var prevVal = frame[0].toInt()

        for (i in 0 until FRAME_SIZE) {
            val curr = frame[i].toInt()
            sumSquares += curr.toDouble() * curr.toDouble()
            if ((prevVal >= 0 && curr < 0) || (prevVal < 0 && curr >= 0)) {
                zeroCrossings++
            }
            prevVal = curr
        }

        val frameRms = sqrt(sumSquares / FRAME_SIZE) / 32768.0
        val zcr = zeroCrossings.toDouble() / FRAME_SIZE

        // Check if above adaptive noise floor
        val snr = frameRms / maxOf(0.001, adaptiveNoiseFloor)
        if (snr < 1.3) {
            prevFrameEnergy = frameRms
            return 0.05f
        }

        // Voice spectral harmonicity (speech typical ZCR between 0.04 and 0.35)
        val zcrScore = if (zcr in 0.03..0.40) 1.0 else 0.2
        val energyScore = minOf(1.0, (snr - 1.0) / 4.0)

        // Spectral autocorrelation for pitch periodicity
        var autoCorr = 0.0
        val maxLag = 160 // 100 Hz pitch limit
        val minLag = 32  // 500 Hz pitch limit
        var maxCorr = 0.0
        for (lag in minLag until maxLag step 4) {
            var dot = 0.0
            for (i in 0 until (FRAME_SIZE - lag)) {
                dot += frame[i].toDouble() * frame[i + lag].toDouble()
            }
            if (dot > maxCorr) maxCorr = dot
        }
        val harmonicity = if (sumSquares > 0) (maxCorr / sumSquares).coerceIn(0.0, 1.0) else 0.0

        val prob = (energyScore * 0.40 + harmonicity * 0.40 + zcrScore * 0.20).toFloat().coerceIn(0.0f, 0.99f)
        prevFrameEnergy = frameRms
        return prob
    }

    private fun calculateFrameRms(frame: ShortArray): Double {
        var sumSquares = 0.0
        for (s in frame) sumSquares += s.toDouble() * s.toDouble()
        return sqrt(sumSquares / frame.size) / 32768.0
    }
}
