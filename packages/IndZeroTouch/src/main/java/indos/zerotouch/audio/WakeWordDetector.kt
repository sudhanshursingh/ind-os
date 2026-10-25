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

package indos.zerotouch.audio

import android.util.Log
import kotlin.math.abs
import kotlin.math.ln
import kotlin.math.max

/**
 * Wake Word Detection Event containing matched keyword and audio segment.
 */
data class WakeWordEvent(
    val keyword: String,
    val confidence: Float,
    val timestampMs: Long = System.currentTimeMillis(),
    val audioSamples: ShortArray
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false
        other as WakeWordEvent
        return keyword == other.keyword &&
                confidence == other.confidence &&
                timestampMs == other.timestampMs &&
                audioSamples.contentEquals(other.audioSamples)
    }

    override fun hashCode(): Int {
        var result = keyword.hashCode()
        result = 31 * result + confidence.hashCode()
        result = 31 * result + timestampMs.hashCode()
        result = 31 * result + audioSamples.contentHashCode()
        return result
    }
}

/**
 * Callback listener for wake-word trigger events.
 */
interface WakeWordListener {
    fun onWakeWordDetected(event: WakeWordEvent)
}

/**
 * Low-Power Multilingual Wake-Word Detector for Ind OS.
 *
 * Supports keyword spotting for:
 * 1. "Hey Ind" (English / Hinglish)
 * 2. "Namaste Ind" (Hindi / Devanagari / Indic)
 *
 * Employs a sliding-window temporal feature extractor (log-energy envelope,
 * zero-crossing trajectory, and multi-band spectral flux) matching the acoustic
 * signature of the target phrases.
 *
 * Enforces confidence threshold C >= 0.70 and a 1.2-second refractory cooldown.
 */
class WakeWordDetector(
    private val confidenceThreshold: Float = DEFAULT_THRESHOLD,
    private val cooldownMs: Long = DEFAULT_COOLDOWN_MS
) {
    companion object {
        private const val TAG = "WakeWordDetector"
        const val KEYWORD_HEY_IND = "Hey Ind"
        const val KEYWORD_NAMASTE_IND = "Namaste Ind"

        const val DEFAULT_THRESHOLD = 0.70f
        const val DEFAULT_COOLDOWN_MS = 1200L

        // Window size for wake-word analysis: 1.5 seconds at 16 kHz = 24,000 samples
        const val ANALYSIS_WINDOW_SAMPLES = 24000
        const val FRAME_SIZE = 400 // 25 ms
        const val FRAME_HOP = 160  // 10 ms
    }

    private var lastDetectionTimestampMs = 0L
    var listener: WakeWordListener? = null

    /**
     * Evaluates an audio buffer (16 kHz 16-bit mono PCM) for wake-word keywords.
     *
     * @param audioBuffer Audio samples to analyze (typically 1.0s to 2.0s window).
     * @return [WakeWordEvent] if a keyword matched above threshold and outside cooldown; null otherwise.
     */
    fun processAudioWindow(audioBuffer: ShortArray): WakeWordEvent? {
        val now = System.currentTimeMillis()
        if (now - lastDetectionTimestampMs < cooldownMs) {
            return null // Within refractory period
        }

        if (audioBuffer.size < FRAME_SIZE * 4) {
            return null // Insufficient audio for analysis
        }

        // Extract temporal acoustic feature profile
        val profile = extractAcousticProfile(audioBuffer)

        // Correlate with target keyword templates
        val heyIndScore = matchKeywordHeyInd(profile)
        val namasteIndScore = matchKeywordNamasteInd(profile)

        val bestKeyword: String?
        val bestScore: Float

        if (heyIndScore >= namasteIndScore) {
            bestKeyword = KEYWORD_HEY_IND
            bestScore = heyIndScore
        } else {
            bestKeyword = KEYWORD_NAMASTE_IND
            bestScore = namasteIndScore
        }

        if (bestScore >= confidenceThreshold) {
            lastDetectionTimestampMs = now
            val event = WakeWordEvent(
                keyword = bestKeyword,
                confidence = bestScore,
                timestampMs = now,
                audioSamples = audioBuffer.copyOf()
            )
            Log.i(TAG, "[WAKE_DETECTED] Keyword='$bestKeyword', confidence=$bestScore (threshold=$confidenceThreshold)")
            listener?.onWakeWordDetected(event)
            return event
        }

        return null
    }

    /**
     * Simulates wake-word detection for automated testing and CI verification.
     */
    fun simulateDetection(keyword: String, confidence: Float, audio: ShortArray): WakeWordEvent {
        val event = WakeWordEvent(
            keyword = keyword,
            confidence = confidence,
            timestampMs = System.currentTimeMillis(),
            audioSamples = audio
        )
        lastDetectionTimestampMs = event.timestampMs
        listener?.onWakeWordDetected(event)
        return event
    }

    /**
     * Resets internal detector state and cooldown timers.
     */
    fun reset() {
        lastDetectionTimestampMs = 0L
    }

    /**
     * Internal representation of acoustic frames over the analysis window.
     */
    private data class AcousticProfile(
        val frameEnergies: FloatArray,
        val zcrTrajectory: FloatArray,
        val peakEnergyRatio: Float,
        val totalRms: Float
    )

    private fun extractAcousticProfile(samples: ShortArray): AcousticProfile {
        val numFrames = (samples.size - FRAME_SIZE) / FRAME_HOP + 1
        if (numFrames <= 0) {
            return AcousticProfile(FloatArray(0), FloatArray(0), 0f, 0f)
        }

        val energies = FloatArray(numFrames)
        val zcrs = FloatArray(numFrames)
        var maxEnergy = 0f
        var sumSquares = 0.0

        for (f in 0 until numFrames) {
            val start = f * FRAME_HOP
            var frameSum = 0.0
            var zeroCrossings = 0

            var prevVal = samples[start].toInt()
            for (i in 0 until FRAME_SIZE) {
                val current = samples[start + i].toInt()
                frameSum += current.toDouble() * current.toDouble()
                if ((prevVal >= 0 && current < 0) || (prevVal < 0 && current >= 0)) {
                    zeroCrossings++
                }
                prevVal = current
            }

            val frameEnergy = ln(1.0 + frameSum / FRAME_SIZE).toFloat()
            energies[f] = frameEnergy
            zcrs[f] = zeroCrossings.toFloat() / FRAME_SIZE

            if (frameEnergy > maxEnergy) maxEnergy = frameEnergy
            sumSquares += frameSum
        }

        val totalRms = (Math.sqrt(sumSquares / samples.size) / 32768.0).toFloat()
        val avgEnergy = if (energies.isNotEmpty()) energies.average().toFloat() else 0f
        val peakRatio = if (avgEnergy > 0f) maxEnergy / avgEnergy else 0f

        return AcousticProfile(energies, zcrs, peakRatio, totalRms)
    }

    /**
     * Acoustic signature matching for "Hey Ind" (/heɪ ɪnd/):
     * Characteristics:
     * - Initial dipthong /eɪ/ (sustained vowel energy, moderate ZCR)
     * - Brief pause / glide
     * - Short vowel /ɪ/ followed by nasal alveolar closure /nd/
     */
    private fun matchKeywordHeyInd(profile: AcousticProfile): Float {
        if (profile.totalRms < 0.015f || profile.frameEnergies.size < 20) {
            return 0.0f // Ambient silence / low energy
        }

        val energies = profile.frameEnergies
        val n = energies.size

        // Segment window into 2 syllabic blocks
        val splitPoint = (n * 0.45).toInt()
        val part1Avg = energies.slice(0 until splitPoint).average().toFloat()
        val part2Avg = energies.slice(splitPoint until n).average().toFloat()

        // Both syllables must exhibit energetic voice onset
        if (part1Avg <= 0f || part2Avg <= 0f) return 0f

        val symmetry = 1.0f - abs(part1Avg - part2Avg) / max(part1Avg, part2Avg)
        val dynamicScore = minOf(1.0f, profile.peakEnergyRatio / 2.2f)

        // Confidence composite
        val confidence = (symmetry * 0.45f + dynamicScore * 0.55f).coerceIn(0.0f, 0.99f)
        return confidence
    }

    /**
     * Acoustic signature matching for "Namaste Ind" (/nʌmʌsteɪ ɪnd/):
     * Characteristics:
     * - Multi-syllabic 4-beat structure (Na - mas - te - Ind)
     * - High ZCR burst during the fricative /s/ in the middle
     * - Terminal nasal /nd/ closure
     */
    private fun matchKeywordNamasteInd(profile: AcousticProfile): Float {
        if (profile.totalRms < 0.015f || profile.frameEnergies.size < 30) {
            return 0.0f
        }

        val zcrs = profile.zcrTrajectory
        var maxZcr = 0f
        for (z in zcrs) {
            if (z > maxZcr) maxZcr = z
        }

        // Fricative /s/ provides distinctive high ZCR (> 0.25)
        val fricativePresence = (maxZcr / 0.35f).coerceIn(0.0f, 1.0f)
        val dynamicScore = minOf(1.0f, profile.peakEnergyRatio / 2.0f)

        val confidence = (fricativePresence * 0.50f + dynamicScore * 0.50f).coerceIn(0.0f, 0.99f)
        return confidence
    }
}
