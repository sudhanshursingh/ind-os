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

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.media.audiofx.AcousticEchoCanceler
import android.media.audiofx.NoiseSuppressor
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Privileged Low-Power Background Audio Capture & Wake-Word Verification Service.
 *
 * Captures 16 kHz 16-bit mono PCM via AudioRecord with VOICE_RECOGNITION hardware filtering,
 * maintains a circular 3-second AudioRingBuffer, runs continuous sliding-window KWS
 * ("Hey Ind" / "Namaste Ind"), and enforces the ECAPA-TDNN biometric voice gate (PRD v1.0.1 §4.2).
 */
class AudioRingBufferService : Service(), WakeWordListener {

    companion object {
        private const val TAG = "AudioRingBufferService"
        const val NOTIFICATION_CHANNEL_ID = "ind_zero_touch_channel"
        const val NOTIFICATION_ID = 2001

        // Intent Actions
        const val ACTION_START_LISTENING = "indos.zerotouch.action.START_LISTENING"
        const val ACTION_STOP_LISTENING = "indos.zerotouch.action.STOP_LISTENING"
        const val ACTION_SIMULATE_WAKE = "indos.zerotouch.action.SIMULATE_WAKE"

        // Broadcast Intents
        const val ACTION_WAKE_VERIFIED = "indos.zerotouch.broadcast.WAKE_VERIFIED"
        const val ACTION_WAKE_PIN_REQUIRED = "indos.zerotouch.broadcast.WAKE_PIN_REQUIRED"
        const val ACTION_WAKE_REJECTED = "indos.zerotouch.broadcast.WAKE_REJECTED"

        // Extras
        const val EXTRA_KEYWORD = "extra_keyword"
        const val EXTRA_CONFIDENCE = "extra_confidence"
        const val EXTRA_COSINE_SIMILARITY = "extra_cosine_similarity"
        const val EXTRA_DECISION = "extra_decision"
        const val EXTRA_AUDIO_SAMPLES = "extra_audio_samples"

        const val SAMPLE_RATE = 16000
        const val CHUNK_SAMPLES = 1600 // 100 ms audio chunks
    }

    private val ringBuffer = AudioRingBuffer(capacity = 48000, sampleRate = SAMPLE_RATE)
    private val wakeWordDetector = WakeWordDetector().apply { listener = this@AudioRingBufferService }
    private val speakerVerifier = EcapaSpeakerVerifier()
    private lateinit var enrollmentManager: VoiceEnrollmentManager

    private var audioRecord: AudioRecord? = null
    private var echoCanceler: AcousticEchoCanceler? = null
    private var noiseSuppressor: NoiseSuppressor? = null

    private var captureThread: Thread? = null
    private val isCapturing = AtomicBoolean(false)

    override fun onCreate() {
        super.onCreate()
        enrollmentManager = VoiceEnrollmentManager(this)
        createNotificationChannel()
        startForegroundServiceNotification()
        Log.i(TAG, "[INIT] AudioRingBufferService initialized")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START_LISTENING -> {
                startAudioCapture()
            }
            ACTION_STOP_LISTENING -> {
                stopAudioCapture()
            }
            ACTION_SIMULATE_WAKE -> {
                val keyword = intent.getStringExtra(EXTRA_KEYWORD) ?: WakeWordDetector.KEYWORD_HEY_IND
                val confidence = intent.getFloatExtra(EXTRA_CONFIDENCE, 0.85f)
                val testAudio = ShortArray(WakeWordDetector.ANALYSIS_WINDOW_SAMPLES) { (it % 100).toShort() }
                wakeWordDetector.simulateDetection(keyword, confidence, testAudio)
            }
            else -> {
                startAudioCapture()
            }
        }
        return START_STICKY
    }

    override fun onDestroy() {
        stopAudioCapture()
        super.onDestroy()
        Log.i(TAG, "[STOP] AudioRingBufferService destroyed")
    }

    override fun onBind(intent: Intent?): IBinder? = null

    /**
     * Spawns low-power audio capture thread.
     */
    @Synchronized
    fun startAudioCapture() {
        if (isCapturing.get()) return

        try {
            val minBufSize = AudioRecord.getMinBufferSize(
                SAMPLE_RATE,
                AudioFormat.CHANNEL_IN_MONO,
                AudioFormat.ENCODING_PCM_16BIT
            )
            val bufferSize = maxOf(minBufSize, CHUNK_SAMPLES * 4)

            audioRecord = AudioRecord(
                MediaRecorder.AudioSource.VOICE_RECOGNITION,
                SAMPLE_RATE,
                AudioFormat.CHANNEL_IN_MONO,
                AudioFormat.ENCODING_PCM_16BIT,
                bufferSize
            )

            // Attach hardware acoustic echo canceler if supported by target SoC (SM7150-AC)
            if (AcousticEchoCanceler.isAvailable()) {
                audioRecord?.audioSessionId?.let { sessionId ->
                    echoCanceler = AcousticEchoCanceler.create(sessionId)?.apply { enabled = true }
                    Log.i(TAG, "[HW_AEC] Hardware Acoustic Echo Canceler attached to session $sessionId")
                }
            }

            // Attach hardware noise suppressor
            if (NoiseSuppressor.isAvailable()) {
                audioRecord?.audioSessionId?.let { sessionId ->
                    noiseSuppressor = NoiseSuppressor.create(sessionId)?.apply { enabled = true }
                    Log.i(TAG, "[HW_NS] Hardware Noise Suppressor attached to session $sessionId")
                }
            }

            if (audioRecord?.state != AudioRecord.STATE_INITIALIZED) {
                Log.w(TAG, "AudioRecord not initialized (permission or mock environment)")
            } else {
                audioRecord?.startRecording()
            }

            isCapturing.set(true)
            captureThread = Thread(::audioCaptureLoop, "IndAudioCaptureThread").apply {
                priority = Thread.NORM_PRIORITY + 2
                start()
            }
            Log.i(TAG, "[CAPTURE_STARTED] 16 kHz Mono PCM audio loop active")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start AudioRecord capture", e)
        }
    }

    /**
     * Safely halts audio capture and releases hardware resources.
     */
    @Synchronized
    fun stopAudioCapture() {
        isCapturing.set(false)
        captureThread?.interrupt()
        captureThread = null

        try {
            audioRecord?.apply {
                if (recordingState == AudioRecord.RECORDSTATE_RECORDING) {
                    stop()
                }
                release()
            }
            audioRecord = null
            echoCanceler?.release()
            echoCanceler = null
            noiseSuppressor?.release()
            noiseSuppressor = null
            Log.i(TAG, "[CAPTURE_STOPPED] Audio capture released")
        } catch (e: Exception) {
            Log.e(TAG, "Error stopping audio capture", e)
        }
    }

    /**
     * Audio loop continuously writing 100 ms chunks into circular buffer and
     * executing sliding-window KWS keyword evaluation.
     */
    private fun audioCaptureLoop() {
        val chunk = ShortArray(CHUNK_SAMPLES)
        while (isCapturing.get() && !Thread.currentThread().isInterrupted) {
            val record = audioRecord
            val read = if (record != null && record.state == AudioRecord.STATE_INITIALIZED) {
                record.read(chunk, 0, CHUNK_SAMPLES)
            } else {
                // Mock reading in host/emulator without active microphone stream
                Thread.sleep(100)
                0
            }

            if (read > 0) {
                ringBuffer.write(chunk, 0, read)

                // Run sliding-window wake-word detection over the latest 1.5 seconds
                if (ringBuffer.availableSamples() >= WakeWordDetector.ANALYSIS_WINDOW_SAMPLES) {
                    val window = ringBuffer.readLatest(WakeWordDetector.ANALYSIS_WINDOW_SAMPLES)
                    wakeWordDetector.processAudioWindow(window)
                }
            }
        }
    }

    /**
     * Triggered when a wake-word ("Hey Ind" or "Namaste Ind") is detected.
     * Evaluates the ECAPA-TDNN biometric gate before deciding whether to unlock.
     */
    override fun onWakeWordDetected(event: WakeWordEvent) {
        Log.i(TAG, "[WAKE_EVENT] Detected '${event.keyword}' with confidence ${event.confidence}")

        val enrolledProfile = enrollmentManager.loadEnrollment()
        if (enrolledProfile == null) {
            // No profile enrolled yet; allow wake in setup mode
            Log.i(TAG, "[VOICE_GATE] No enrolled profile found. Allowing open wake for initial setup.")
            broadcastWakeResult(ACTION_WAKE_VERIFIED, event, VerificationDecision.VERIFIED, 1.0f)
            return
        }

        // Biometric evaluation via ECAPA-TDNN
        val verification = speakerVerifier.verify(event.audioSamples, enrolledProfile)
        when (verification.decision) {
            VerificationDecision.VERIFIED -> {
                Log.i(TAG, "[VOICE_GATE] Verification SUCCESS (S_cos=${verification.cosineSimilarity})")
                broadcastWakeResult(ACTION_WAKE_VERIFIED, event, verification.decision, verification.cosineSimilarity)
            }
            VerificationDecision.PIN_REQUIRED -> {
                Log.w(TAG, "[VOICE_GATE] PIN REQUIRED (S_cos=${verification.cosineSimilarity})")
                broadcastWakeResult(ACTION_WAKE_PIN_REQUIRED, event, verification.decision, verification.cosineSimilarity)
            }
            VerificationDecision.REJECTED -> {
                Log.w(TAG, "[VOICE_GATE] Verification REJECTED (S_cos=${verification.cosineSimilarity})")
                broadcastWakeResult(ACTION_WAKE_REJECTED, event, verification.decision, verification.cosineSimilarity)
            }
        }
    }

    private fun broadcastWakeResult(
        action: String,
        event: WakeWordEvent,
        decision: VerificationDecision,
        similarity: Float
    ) {
        val intent = Intent(action).apply {
            putExtra(EXTRA_KEYWORD, event.keyword)
            putExtra(EXTRA_CONFIDENCE, event.confidence)
            putExtra(EXTRA_DECISION, decision.name)
            putExtra(EXTRA_COSINE_SIMILARITY, similarity)
        }
        sendBroadcast(intent)
    }

    private fun startForegroundServiceNotification() {
        val notification: Notification = NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
            .setContentTitle("Ind OS Zero-Touch")
            .setContentText("Listening for 'Hey Ind' & 'Namaste Ind'")
            .setSmallIcon(android.R.drawable.ic_btn_speak_now)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
            .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
            )
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "Ind OS Voice Listener",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Always-on low-power wake-word detection"
            }
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }
}
