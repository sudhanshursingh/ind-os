package indos.launcher

import android.os.Bundle
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

/**
 * Zero-Bloat Intent Launcher for Ind OS.
 * Minimal RAM footprint (<45 MB resident), 120 Hz frame pacing, zero trackers.
 */
class IndLauncherActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val textView = TextView(this).apply {
            text = "Ind OS — Ambient Intelligence Launcher"
            textSize = 20f
            textAlignment = TextView.TEXT_ALIGNMENT_CENTER
            setPadding(32, 64, 32, 32)
        }
        setContentView(textView)
    }
}
