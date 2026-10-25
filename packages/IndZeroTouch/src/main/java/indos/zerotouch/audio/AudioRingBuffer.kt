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

import kotlin.math.sqrt

/**
 * Thread-safe circular audio ring buffer storing 16 kHz 16-bit mono PCM samples.
 *
 * Designed for low-power continuous background audio capture and sliding-window
 * wake-word analysis as specified in Ind OS PRD v1.0.1 §4.2 Stage 1.
 *
 * Default capacity: 48,000 samples (3.0 seconds at 16,000 Hz).
 * Memory footprint: ~96 KB.
 */
class AudioRingBuffer(
    val capacity: Int = DEFAULT_CAPACITY,
    val sampleRate: Int = DEFAULT_SAMPLE_RATE
) {
    companion object {
        const val DEFAULT_SAMPLE_RATE = 16000
        const val DEFAULT_WINDOW_SECONDS = 3.0f
        const val DEFAULT_CAPACITY = (DEFAULT_SAMPLE_RATE * DEFAULT_WINDOW_SECONDS).toInt() // 48,000
    }

    private val buffer = ShortArray(capacity)
    private var writeHead = 0
    private var totalSamplesWritten = 0L
    private val lock = Any()

    /**
     * Writes new audio samples into the ring buffer.
     * Older samples are overwritten once capacity is reached.
     */
    fun write(samples: ShortArray, offset: Int = 0, length: Int = samples.size) {
        if (length <= 0) return

        synchronized(lock) {
            for (i in 0 until length) {
                buffer[writeHead] = samples[offset + i]
                writeHead = (writeHead + 1) % capacity
                totalSamplesWritten++
            }
        }
    }

    /**
     * Returns the number of valid samples currently available in the buffer,
     * capped at the maximum capacity.
     */
    fun availableSamples(): Int {
        synchronized(lock) {
            return if (totalSamplesWritten < capacity) totalSamplesWritten.toInt() else capacity
        }
    }

    /**
     * Reads the most recent [sampleCount] samples from the buffer.
     * If fewer samples are available than requested, the returned array will
     * contain zero-padded leading samples followed by the available audio.
     */
    fun readLatest(sampleCount: Int): ShortArray {
        val result = ShortArray(sampleCount)
        if (sampleCount <= 0) return result

        synchronized(lock) {
            val available = availableSamples()
            val samplesToRead = minOf(sampleCount, available)
            val padding = sampleCount - samplesToRead

            // Calculate starting read position in ring buffer
            var readHead = (writeHead - samplesToRead + capacity) % capacity
            for (i in 0 until samplesToRead) {
                result[padding + i] = buffer[readHead]
                readHead = (readHead + 1) % capacity
            }
        }
        return result
    }

    /**
     * Reads a specific window of samples ending [offsetFromLatest] samples prior to the write head.
     */
    fun readWindow(offsetFromLatest: Int, count: Int): ShortArray {
        val result = ShortArray(count)
        if (count <= 0) return result

        synchronized(lock) {
            val available = availableSamples()
            if (offsetFromLatest >= available) return result

            val effectiveEnd = minOf(offsetFromLatest + count, available)
            val samplesToRead = effectiveEnd - offsetFromLatest
            val padding = count - samplesToRead

            var readHead = (writeHead - offsetFromLatest - samplesToRead + capacity) % capacity
            for (i in 0 until samplesToRead) {
                result[padding + i] = buffer[readHead]
                readHead = (readHead + 1) % capacity
            }
        }
        return result
    }

    /**
     * Resets the buffer write head and sample counter.
     */
    fun clear() {
        synchronized(lock) {
            writeHead = 0
            totalSamplesWritten = 0L
            buffer.fill(0)
        }
    }

    /**
     * Computes the Root Mean Square (RMS) energy of an audio buffer in normalized [0.0, 1.0].
     */
    fun calculateRms(samples: ShortArray): Float {
        if (samples.isEmpty()) return 0f
        var sumSquares = 0.0
        for (sample in samples) {
            val normalized = sample / 32768.0
            sumSquares += normalized * normalized
        }
        return sqrt(sumSquares / samples.size).toFloat()
    }
}
