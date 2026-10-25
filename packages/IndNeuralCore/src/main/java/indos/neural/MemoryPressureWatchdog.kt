package indos.neural

import android.app.ActivityManager
import android.content.ComponentCallbacks2
import android.content.Context
import android.content.res.Configuration
import android.util.Log

/**
 * MemoryPressureWatchdog monitors OS-level memory pressure signals
 * and enforces the Invariant 1 memory partition budget (PRD §3.2).
 *
 * Listens to ComponentCallbacks2.onTrimMemory() and ActivityManager.MemoryInfo.
 * Evicts mmap model weights whenever:
 * 1. level >= TRIM_MEMORY_RUNNING_CRITICAL (15)
 * 2. level >= TRIM_MEMORY_COMPLETE (80)
 * 3. Free physical RAM drops below 600 MB (M_headroom floor)
 */
class MemoryPressureWatchdog(private val context: Context) : ComponentCallbacks2 {

    companion object {
        private const val TAG = "IND_LIFECYCLE"
        private const val RAM_PAGER_TAG = "IND_RAM_PAGER"

        // Free RAM floor threshold from PRD §3.2 Invariant 1 (600 MB)
        const val FREE_RAM_CRITICAL_THRESHOLD_BYTES = 600L * 1024L * 1024L
    }

    private val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
    private val memoryInfo = ActivityManager.MemoryInfo()

    override fun onTrimMemory(level: Int) {
        val levelStr = getTrimMemoryDescription(level)
        Log.i(TAG, "onTrimMemory received: level=$level ($levelStr)")

        val isCriticalPressure = when (level) {
            ComponentCallbacks2.TRIM_MEMORY_RUNNING_CRITICAL,
            ComponentCallbacks2.TRIM_MEMORY_COMPLETE,
            ComponentCallbacks2.TRIM_MEMORY_MODERATE -> true
            else -> false
        }

        activityManager.getMemoryInfo(memoryInfo)
        val freeMb = memoryInfo.availMem / (1024 * 1024)
        val totalMb = memoryInfo.totalMem / (1024 * 1024)

        Log.d(
            TAG,
            "System RAM status: Free=${freeMb} MB / Total=${totalMb} MB, lowMemory=${memoryInfo.lowMemory}, threshold=${memoryInfo.threshold / (1024 * 1024)} MB"
        )

        if (isCriticalPressure || memoryInfo.availMem < FREE_RAM_CRITICAL_THRESHOLD_BYTES || memoryInfo.lowMemory) {
            Log.w(
                RAM_PAGER_TAG,
                "CRITICAL MEMORY CONDITION DETECTED! Level=$level, AvailMem=${freeMb}MB (< 600MB threshold: ${memoryInfo.availMem < FREE_RAM_CRITICAL_THRESHOLD_BYTES}). Triggering immediate model weight eviction."
            )
            ModelLifecyclePager.instance.evictModelWeights("onTrimMemory: level=$level ($levelStr)")
        }
    }

    override fun onLowMemory() {
        Log.e(TAG, "onLowMemory received from system! Performing emergency eviction.")
        ModelLifecyclePager.instance.evictModelWeights("onLowMemory")
    }

    override fun onConfigurationChanged(newConfig: Configuration) {
        // No-op for memory watchdog
    }

    /**
     * Polls current MemoryInfo and executes eviction if free RAM is below the 600 MB invariant.
     * @return true if eviction was triggered, false otherwise
     */
    fun checkMemoryPressure(): Boolean {
        activityManager.getMemoryInfo(memoryInfo)
        val freeBytes = memoryInfo.availMem
        val freeMb = freeBytes / (1024 * 1024)
        if (freeBytes < FREE_RAM_CRITICAL_THRESHOLD_BYTES || memoryInfo.lowMemory) {
            Log.w(
                RAM_PAGER_TAG,
                "Memory watchdog poll: Available RAM (${freeMb} MB) < 600 MB threshold. Triggering eviction."
            )
            ModelLifecyclePager.instance.evictModelWeights("POLL_UNDER_600MB")
            return true
        }
        return false
    }

    private fun getTrimMemoryDescription(level: Int): String {
        return when (level) {
            ComponentCallbacks2.TRIM_MEMORY_COMPLETE -> "TRIM_MEMORY_COMPLETE"
            ComponentCallbacks2.TRIM_MEMORY_MODERATE -> "TRIM_MEMORY_MODERATE"
            ComponentCallbacks2.TRIM_MEMORY_BACKGROUND -> "TRIM_MEMORY_BACKGROUND"
            ComponentCallbacks2.TRIM_MEMORY_UI_HIDDEN -> "TRIM_MEMORY_UI_HIDDEN"
            ComponentCallbacks2.TRIM_MEMORY_RUNNING_CRITICAL -> "TRIM_MEMORY_RUNNING_CRITICAL"
            ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW -> "TRIM_MEMORY_RUNNING_LOW"
            ComponentCallbacks2.TRIM_MEMORY_RUNNING_MODERATE -> "TRIM_MEMORY_RUNNING_MODERATE"
            else -> "LEVEL_$level"
        }
    }
}
