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

import android.app.Service
import android.content.Context
import android.content.Intent
import android.graphics.PixelFormat
import android.os.Build
import android.os.IBinder
import android.util.Log
import android.view.Gravity
import android.view.WindowManager

/**
 * Privileged Voice Orb Floating Window Overlay Service for Ind OS (PRD v1.0.1 Sprint 8 Task 3).
 *
 * Displays a glowing animated waveform pill at the bottom of the screen using
 * WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY to provide instant visual
 * feedback during voice interaction (Listening -> Processing -> Speaking).
 */
class IndOrbOverlayService : Service() {

    companion object {
        private const val TAG = "IndOrbOverlayService"

        const val ACTION_SHOW_ORB = "indos.zerotouch.action.SHOW_ORB"
        const val ACTION_HIDE_ORB = "indos.zerotouch.action.HIDE_ORB"
        const val ACTION_SET_STATE = "indos.zerotouch.action.SET_ORB_STATE"
        const val ACTION_SET_AMPLITUDE = "indos.zerotouch.action.SET_ORB_AMPLITUDE"

        const val EXTRA_ORB_STATE = "extra_orb_state"
        const val EXTRA_AMPLITUDE = "extra_amplitude"
    }

    private var windowManager: WindowManager? = null
    private var orbView: OrbWaveformView? = null
    private var isOverlayAttached = false

    override fun onCreate() {
        super.onCreate()
        try {
            windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager
        } catch (e: Exception) {
            Log.w(TAG, "WindowManager unavailable in current environment: ${e.message}")
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_SHOW_ORB -> {
                showOrbOverlay()
            }
            ACTION_HIDE_ORB -> {
                hideOrbOverlay()
            }
            ACTION_SET_STATE -> {
                val stateName = intent.getStringExtra(EXTRA_ORB_STATE) ?: "LISTENING"
                setOrbState(stateName)
            }
            ACTION_SET_AMPLITUDE -> {
                val amp = intent.getFloatExtra(EXTRA_AMPLITUDE, 0.2f)
                orbView?.audioAmplitude = amp
            }
            else -> {
                showOrbOverlay()
            }
        }
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        hideOrbOverlay()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    /**
     * Attaches the Voice Orb overlay to the active WindowManager.
     */
    @Synchronized
    fun showOrbOverlay() {
        if (isOverlayAttached) return

        try {
            val wm = windowManager ?: return
            val density = resources.displayMetrics.density
            val widthPx = (240 * density).toInt()
            val heightPx = (52 * density).toInt()
            val yOffsetPx = (48 * density).toInt()

            val windowType = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY
            } else {
                @Suppress("DEPRECATION")
                WindowManager.LayoutParams.TYPE_PHONE
            }

            val params = WindowManager.LayoutParams(
                widthPx,
                heightPx,
                windowType,
                WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or
                        WindowManager.LayoutParams.FLAG_LAYOUT_NO_LIMITS or
                        WindowManager.LayoutParams.FLAG_HARDWARE_ACCELERATED,
                PixelFormat.TRANSLUCENT
            ).apply {
                gravity = Gravity.BOTTOM or Gravity.CENTER_HORIZONTAL
                y = yOffsetPx
            }

            orbView = OrbWaveformView(this).apply {
                setOnClickListener {
                    hideOrbOverlay()
                }
            }

            wm.addView(orbView, params)
            isOverlayAttached = true
            Log.i(TAG, "[ORB_ATTACHED] Voice Orb overlay attached via TYPE_APPLICATION_OVERLAY")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to attach Voice Orb overlay", e)
        }
    }

    /**
     * Detaches and removes the Voice Orb overlay from the window hierarchy.
     */
    @Synchronized
    fun hideOrbOverlay() {
        if (!isOverlayAttached) return
        try {
            orbView?.let { windowManager?.removeView(it) }
            orbView = null
            isOverlayAttached = false
            Log.i(TAG, "[ORB_DETACHED] Voice Orb overlay detached")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to detach Voice Orb overlay", e)
        }
    }

    private fun setOrbState(stateName: String) {
        val state = try {
            OrbWaveformView.OrbState.valueOf(stateName.uppercase())
        } catch (e: Exception) {
            OrbWaveformView.OrbState.LISTENING
        }
        orbView?.state = state
        Log.i(TAG, "[ORB_STATE] Updated Voice Orb state to $state")
    }
}
