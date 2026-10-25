package indos.diagnostic

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.graphics.Color
import android.graphics.Typeface
import android.os.Bundle
import android.os.IBinder
import android.util.Log
import android.view.Gravity
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import indos.neural.ipc.IIndAIManager
import kotlin.concurrent.thread

/**
 * Visual diagnostic console for inspecting IndNeuralCore Binder IPC latency,
 * memory paging status, and running the 1,000-call benchmark.
 */
class DiagnosticActivity : AppCompatActivity() {

    companion object {
        private const val TAG = "IND_IPC_BENCH"
    }

    private var aiManager: IIndAIManager? = null
    private var isBound = false

    private lateinit var statusTextView: TextView
    private lateinit var resultsTextView: TextView
    private lateinit var runButton: Button
    private lateinit var evictButton: Button

    private val connection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            aiManager = IIndAIManager.Stub.asInterface(service)
            isBound = true
            updateStatus("Connected to IndNeuralCore Daemon (PID active)")
            runButton.isEnabled = true
            evictButton.isEnabled = true
            fetchEngineStatus()
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            aiManager = null
            isBound = false
            updateStatus("Disconnected from IndNeuralCore")
            runButton.isEnabled = false
            evictButton.isEnabled = false
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val rootLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(40, 60, 40, 40)
            setBackgroundColor(Color.parseColor("#121212"))
        }

        val titleView = TextView(this).apply {
            text = "Ind OS — Neural IPC Diagnostic Console"
            textSize = 20f
            setTextColor(Color.WHITE)
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER_HORIZONTAL
            setPadding(0, 0, 0, 24)
        }
        rootLayout.addView(titleView)

        statusTextView = TextView(this).apply {
            text = "Connecting to IndNeuralCore..."
            textSize = 14f
            setTextColor(Color.parseColor("#00E676"))
            setPadding(0, 0, 0, 20)
        }
        rootLayout.addView(statusTextView)

        val buttonLayout = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(0, 0, 0, 24)
        }

        runButton = Button(this).apply {
            text = "Run 1,000 IPC Benchmark"
            isEnabled = false
            setOnClickListener {
                runBenchmarkAsync()
            }
        }
        buttonLayout.addView(runButton, LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f))

        evictButton = Button(this).apply {
            text = "Trigger Eviction"
            isEnabled = false
            setOnClickListener {
                triggerEvictionAsync()
            }
        }
        buttonLayout.addView(evictButton, LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f))
        rootLayout.addView(buttonLayout)

        val scrollView = ScrollView(this).apply {
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.MATCH_PARENT
            )
        }

        resultsTextView = TextView(this).apply {
            text = "Diagnostic output will appear here.\nLogcat tag: IND_IPC_BENCH"
            textSize = 13f
            setTextColor(Color.LTGRAY)
            typeface = Typeface.MONOSPACE
            setPadding(20, 20, 20, 20)
            setBackgroundColor(Color.parseColor("#1E1E1E"))
        }
        scrollView.addView(resultsTextView)
        rootLayout.addView(scrollView)

        setContentView(rootLayout)
        bindToAiService()
    }

    override fun onDestroy() {
        super.onDestroy()
        if (isBound) {
            unbindService(connection)
            isBound = false
        }
    }

    private fun bindToAiService() {
        val intent = Intent().apply {
            component = ComponentName("in.indos.neural", "indos.neural.IndAIService")
        }
        val bound = bindService(intent, connection, Context.BIND_AUTO_CREATE)
        if (!bound) {
            updateStatus("Failed to bind to in.indos.neural/indos.neural.IndAIService")
        }
    }

    private fun updateStatus(status: String) {
        runOnUiThread {
            statusTextView.text = status
        }
    }

    private fun fetchEngineStatus() {
        thread {
            try {
                val status = aiManager?.engineStatusJson ?: "Unavailable"
                runOnUiThread {
                    resultsTextView.text = "Initial Engine Status:\n$status\n\nReady for 1,000-call benchmark."
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error fetching engine status", e)
            }
        }
    }

    private fun runBenchmarkAsync() {
        runButton.isEnabled = false
        resultsTextView.text = "Executing 1,000 round-trip Binder IPC calls... Please wait."

        thread {
            val manager = aiManager
            if (manager == null) {
                runOnUiThread {
                    resultsTextView.text = "Error: aiManager is null."
                    runButton.isEnabled = true
                }
                return@thread
            }

            try {
                val result = IpcBenchmarkRunner.runBenchmark(manager, "DiagnosticActivity", 1000)
                val summary = StringBuilder().apply {
                    appendLine("==========================================")
                    appendLine("IND OS BINDER IPC BENCHMARK RESULTS")
                    appendLine("==========================================")
                    appendLine("Total Calls: ${result.totalCalls}")
                    appendLine("Target:      Median < 2.500 ms")
                    appendLine("Result:      ${if (result.targetMet) "PASS" else "FAIL"}")
                    appendLine("------------------------------------------")
                    appendLine("Min Latency:    %.3f ms".format(result.minMs))
                    appendLine("Avg Latency:    %.3f ms".format(result.avgMs))
                    appendLine("Median Latency: %.3f ms".format(result.medianMs))
                    appendLine("P95 Latency:    %.3f ms".format(result.p95Ms))
                    appendLine("P99 Latency:    %.3f ms".format(result.p99Ms))
                    appendLine("Max Latency:    %.3f ms".format(result.maxMs))
                    appendLine("------------------------------------------")
                    appendLine("Engine Telemetry:")
                    appendLine(result.engineStatus)
                    appendLine("==========================================")
                }.toString()

                runOnUiThread {
                    resultsTextView.text = summary
                    runButton.isEnabled = true
                }
            } catch (t: Throwable) {
                runOnUiThread {
                    resultsTextView.text = "Benchmark failed: ${t.message}"
                    runButton.isEnabled = true
                }
            }
        }
    }

    private fun triggerEvictionAsync() {
        thread {
            try {
                aiManager?.evictModelWeights()
                val status = aiManager?.engineStatusJson ?: "Unavailable"
                runOnUiThread {
                    resultsTextView.text = "Model weights eviction triggered via Binder IPC.\n\nUpdated Engine Telemetry:\n$status"
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error triggering eviction", e)
            }
        }
    }
}
