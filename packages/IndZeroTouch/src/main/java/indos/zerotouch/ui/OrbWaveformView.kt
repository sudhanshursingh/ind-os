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

package indos.zerotouch.ui

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.RectF
import android.graphics.Shader
import android.util.AttributeSet
import android.view.View
import kotlin.math.sin

/**
 * Animated Glowing Waveform Pill View for Ind OS Voice Orb.
 *
 * Displays state-dependent visual feedback:
 * - LISTENING: Audio-reactive multi-bar waveform (Emerald / Cyan glow).
 * - PROCESSING: Rotating dual-gradient shimmer (Cyan / Electric Violet).
 * - SPEAKING: Rhythmic vocal cadence waves.
 */
class OrbWaveformView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = 0
) : View(context, attrs, defStyleAttr) {

    enum class OrbState {
        IDLE,
        LISTENING,
        PROCESSING,
        SPEAKING
    }

    var state: OrbState = OrbState.LISTENING
        set(value) {
            field = value
            invalidate()
        }

    var audioAmplitude: Float = 0.2f
        set(value) {
            field = value.coerceIn(0.0f, 1.0f)
            invalidate()
        }

    private val bgPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.parseColor("#E6121212") // 90% opacity deep dark pill
        style = Paint.Style.FILL
    }

    private val borderPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.parseColor("#4400E676")
        style = Paint.Style.STROKE
        strokeWidth = 3f
    }

    private val wavePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        strokeCap = Paint.Cap.ROUND
    }

    private val pillBounds = RectF()
    private var animPhase = 0.0f

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)

        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 0 || h <= 0) return

        val radius = h / 2f
        pillBounds.set(4f, 4f, w - 4f, h - 4f)

        // 1. Draw glowing pill background
        canvas.drawRoundRect(pillBounds, radius, radius, bgPaint)
        canvas.drawRoundRect(pillBounds, radius, radius, borderPaint)

        // 2. Draw animated waveform bars
        animPhase += 0.12f
        val numBars = 12
        val barSpacing = (w - 48f) / (numBars + 1)
        val centerY = h / 2f

        for (i in 0 until numBars) {
            val x = 24f + (i + 1) * barSpacing
            val barPhase = animPhase + i * 0.4f
            val baseHeight = when (state) {
                OrbState.LISTENING -> (h * 0.25f) + (h * 0.50f * audioAmplitude * (0.6f + 0.4f * sin(barPhase.toDouble()).toFloat()))
                OrbState.PROCESSING -> (h * 0.35f) * (0.5f + 0.5f * sin(barPhase.toDouble()).toFloat())
                OrbState.SPEAKING -> (h * 0.40f) * (0.7f + 0.3f * sin(barPhase * 1.5).toFloat())
                OrbState.IDLE -> 6f
            }

            val topY = centerY - baseHeight / 2f
            val bottomY = centerY + baseHeight / 2f

            // Dynamic gradient colors
            val shader = LinearGradient(
                x, topY, x, bottomY,
                when (state) {
                    OrbState.LISTENING -> intArrayOf(Color.parseColor("#00E676"), Color.parseColor("#00E5FF"))
                    OrbState.PROCESSING -> intArrayOf(Color.parseColor("#00E5FF"), Color.parseColor("#7C4DFF"))
                    OrbState.SPEAKING -> intArrayOf(Color.parseColor("#FFD600"), Color.parseColor("#FF6D00"))
                    OrbState.IDLE -> intArrayOf(Color.GRAY, Color.DKGRAY)
                },
                null,
                Shader.TileMode.CLAMP
            )

            wavePaint.shader = shader
            wavePaint.strokeWidth = 6f
            canvas.drawLine(x, topY, x, bottomY, wavePaint)
        }

        // Loop animation frame
        postInvalidateOnAnimation()
    }
}
