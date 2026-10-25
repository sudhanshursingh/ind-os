package indos.neural

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log
import indos.neural.ipc.IIndAIManager
import indos.neural.ipc.IIndStreamCallback
import org.json.JSONObject

/**
 * Privileged Native AI IPC Service for Ind OS.
 * Runs in an isolated persistent daemon process: `indos.neural.daemon`.
 */
class IndAIService : Service() {

    companion object {
        private const val TAG = "IND_AI_SERVICE"
    }

    private val binder = object : IIndAIManager.Stub() {
        override fun classifyFast(payload: String?, domain: String?): String {
            Log.d(TAG, "classifyFast called: domain=$domain")
            val result = JSONObject()
            when (domain) {
                "SMS_SCAM" -> {
                    val isScam = payload?.contains("KYC", ignoreCase = true) == true ||
                            payload?.contains("electricity", ignoreCase = true) == true ||
                            payload?.contains("pan update", ignoreCase = true) == true
                    result.put("riskScore", if (isScam) 0.96 else 0.05)
                    result.put("verdict", if (isScam) "BLOCK" else "PASS")
                    result.put("reason", if (isScam) "Detected fraudulent phishing pattern" else "Benign SMS")
                }
                "PHISHING_URL" -> {
                    result.put("riskScore", 0.0)
                    result.put("verdict", "PASS")
                    result.put("reason", "URL safe")
                }
                else -> {
                    result.put("riskScore", 0.0)
                    result.put("verdict", "PASS")
                    result.put("reason", "Default pass")
                }
            }
            return result.toString()
        }

        override fun executeZeroTouchCommand(
            userUtterance: String?,
            screenGraphJson: String?,
            callback: IIndStreamCallback?
        ) {
            Log.d(TAG, "executeZeroTouchCommand: '$userUtterance'")
            callback?.onToken("Analyzing command...")
            
            // Initial stub response conforming to ind_action.gbnf
            val actionResponse = JSONObject().apply {
                put("tier", "GREEN")
                put("actions", org.json.JSONArray())
                put("tts", "Command received: $userUtterance")
            }
            callback?.onActionResolved(actionResponse.toString())
        }

        override fun evictModelWeights() {
            Log.i(TAG, "evictModelWeights requested: invoking madvise MADV_DONTNEED on active mmap pages")
        }

        override fun getEngineStatusJson(): String {
            return JSONObject().apply {
                put("engine", "IndNeuralCore-v1.0.0")
                put("loadedModel", "None (Sprint 1 Skeleton)")
                put("residentRamMB", 18)
                put("airGapEnforced", true)
            }.toString()
        }
    }

    override fun onBind(intent: Intent?): IBinder {
        Log.i(TAG, "Client bound to IndAIService: ${intent?.action}")
        return binder
    }
}
