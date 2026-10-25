package indos.neural

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.os.RemoteException
import android.util.Log
import indos.neural.ipc.IIndAIManager
import indos.neural.ipc.IIndStreamCallback
import org.json.JSONArray
import org.json.JSONObject

/**
 * Privileged Native AI IPC Service for Ind OS.
 * Runs in an isolated persistent daemon process: `indos.neural.daemon`.
 * Exposes Binder IPC interface IIndAIManager to client subsystems.
 */
class IndAIService : Service() {

    companion object {
        private const val TAG = "IND_AI_SERVICE"
        private const val IPC_BENCH_TAG = "IND_IPC_BENCH"
    }

    private lateinit var memoryPressureWatchdog: MemoryPressureWatchdog
    private val pager = ModelLifecyclePager.instance

    override fun onCreate() {
        super.onCreate()
        Log.i(TAG, "IndAIService initializing in process ${android.os.Process.myPid()}...")
        memoryPressureWatchdog = MemoryPressureWatchdog(applicationContext)
        registerComponentCallbacks(memoryPressureWatchdog)
        Log.i(TAG, "MemoryPressureWatchdog registered with ComponentCallbacks2.")

        // Native C++ NDK Pipeline & Hardware Acceleration Probe (Sprint 3 & 4)
        val isNative = NativeBridge.isNativeLoaded()
        Log.i(TAG, "Native C++ NDK Pipeline status: loaded=$isNative")
        if (isNative) {
            val probeReport = NativeBridge.getHardwareReport()
            Log.i("IND_NATIVE", "[IND_NATIVE] Hardware acceleration probe result on boot: $probeReport")
            
            // Sprint 4: Zero-copy mmap model loader
            val modelLoaded = NativeBridge.loadModel("/data/ind/models/Qwen2.5-1.5B-Instruct-Q4_K_M.gguf")
            Log.i("IND_BENCHMARK", "[IND_BENCHMARK] Zero-copy mmap model loading on boot: success=$modelLoaded, telemetry=${NativeBridge.getMmapStatus()}")
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            "indos.neural.ACTION_RUN_PROBE" -> {
                Log.i(TAG, "Triggering Native Hardware Acceleration Probe on demand...")
                val probeReport = NativeBridge.getHardwareReport()
                Log.i("IND_NATIVE", "[IND_NATIVE] Hardware acceleration probe result (on-demand): $probeReport")
            }
            "indos.neural.ACTION_RUN_BENCHMARK" -> {
                Log.i(TAG, "Triggering Sprint 4 DoD Inference & Eviction Benchmark on demand...")
                val benchReport = NativeBridge.runSprint4Benchmark(64)
                Log.i("IND_BENCHMARK", "[IND_BENCHMARK] Sprint 4 Benchmark Result: $benchReport")
            }
        }
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        super.onDestroy()
        unregisterComponentCallbacks(memoryPressureWatchdog)
        Log.i(TAG, "IndAIService destroyed; watchdog unregistered.")
    }

    private val binder = object : IIndAIManager.Stub() {

        /**
         * Fast synchronous classification (< 60 ms).
         * Used by IndSurakshaShield for incoming SMS/URL triage.
         */
        override fun classifyFast(payload: String?, domain: String?): String {
            val startNs = System.nanoTime()
            val text = payload ?: ""
            val dom = domain ?: "SMS_SCAM"

            val result = JSONObject()
            when (dom) {
                "SMS_SCAM" -> {
                    val lower = text.lowercase()
                    val isScam = lower.contains("kyc") ||
                            lower.contains("pan update") ||
                            lower.contains("electricity bill") ||
                            lower.contains("lottery") ||
                            lower.contains("bank account blocked") ||
                            lower.contains("digital arrest")

                    val risk = if (isScam) 0.96 else 0.04
                    val verdict = if (isScam) "BLOCK" else "PASS"
                    val reason = if (isScam) "Detected fraudulent phishing/impersonation keyword pattern" else "Benign SMS content"

                    result.put("riskScore", risk)
                    result.put("verdict", verdict)
                    result.put("reason", reason)
                }
                "PHISHING_URL" -> {
                    val isSuspiciousDomain = text.contains(".apk") ||
                            text.contains("bit.ly") ||
                            text.contains("tinyurl") ||
                            text.contains("-sbi-") ||
                            text.contains("-bank-")

                    result.put("riskScore", if (isSuspiciousDomain) 0.92 else 0.02)
                    result.put("verdict", if (isSuspiciousDomain) "BLOCK" else "PASS")
                    result.put("reason", if (isSuspiciousDomain) "Suspicious domain/APK payload" else "Legitimate URL")
                }
                "NOTIFICATION_PRIORITY" -> {
                    val isOtp = text.contains("OTP", ignoreCase = true) ||
                            text.contains("debit", ignoreCase = true) ||
                            text.contains("credit", ignoreCase = true) ||
                            text.contains("UPI", ignoreCase = true)

                    result.put("riskScore", 0.0)
                    result.put("verdict", if (isOtp) "HIGH_PRIORITY" else "NORMAL")
                    result.put("reason", if (isOtp) "Financial transactional message" else "Standard notification")
                }
                else -> {
                    result.put("riskScore", 0.0)
                    result.put("verdict", "PASS")
                    result.put("reason", "Domain '$dom' evaluated")
                }
            }

            val elapsedMs = (System.nanoTime() - startNs) / 1_000_000.0
            result.put("latencyMs", elapsedMs)
            return result.toString()
        }

        /**
         * Compiles NL command (En/Hi/Hinglish) + screen graph into deterministic OS actions.
         * Enforces GBNF schema format: {tier, actions, tts}.
         */
        override fun executeZeroTouchCommand(
            userUtterance: String?,
            screenGraphJson: String?,
            callback: IIndStreamCallback?
        ) {
            val command = userUtterance ?: ""
            Log.d(TAG, "executeZeroTouchCommand (Streaming JNI): '$command'")

            try {
                NativeBridge.streamInference(
                    command,
                    screenGraphJson,
                    object : NativeStreamListener {
                        override fun onNativeToken(token: String) {
                            try {
                                callback?.onToken(token)
                            } catch (e: RemoteException) {
                                Log.e(TAG, "RemoteException streaming token to client", e)
                            }
                        }

                        override fun onNativeComplete(actionJson: String) {
                            try {
                                callback?.onActionResolved(actionJson)
                            } catch (e: RemoteException) {
                                Log.e(TAG, "RemoteException resolving action to client", e)
                            }
                        }

                        override fun onNativeError(errorCode: Int, message: String) {
                            try {
                                callback?.onError(errorCode, message)
                            } catch (e: RemoteException) {
                                Log.e(TAG, "RemoteException sending error to client", e)
                            }
                        }
                    }
                )
            } catch (t: Throwable) {
                Log.e(TAG, "Error executing zero touch command", t)
                try {
                    callback?.onError(500, t.message ?: "Internal execution error")
                } catch (ignored: Exception) {}
            }
        }

        /**
         * Immediately evicts mmap-backed GGUF weights from physical RAM pages.
         */
        override fun evictModelWeights() {
            Log.i(TAG, "evictModelWeights invoked via Binder IPC.")
            pager.evictModelWeights("AIDL_IPC_CALL")
        }

        /**
         * Real-time engine telemetry JSON.
         */
        override fun getEngineStatusJson(): String {
            return JSONObject().apply {
                put("engine", "IndNeuralCore-v1.0.1")
                put("loadedModel", pager.getLoadedModelId())
                put("residentRamMB", pager.getResidentRamMb())
                put("pagerState", pager.getState().name)
                put("evictionCount", pager.getTotalEvictionCount())
                put("lastEvictionDurationMs", pager.getLastEvictionDurationMs())
                put("airGapEnforced", true)
                put("targetDevice", "Moto G60 (hanoip)")
                put("nativeLibraryLoaded", NativeBridge.isNativeLoaded())
                put("hardwareCapabilities", JSONObject(NativeBridge.getHardwareReport()))
                put("mmapTelemetry", JSONObject(NativeBridge.getMmapStatus()))
            }.toString()
        }
    }

    override fun onBind(intent: Intent?): IBinder {
        Log.i(TAG, "Client bound to IndAIService with action: ${intent?.action}")
        return binder
    }
}
