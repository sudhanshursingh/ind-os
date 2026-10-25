package indos.neural

import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.util.Log

/**
 * AppIndexResolver implements fuzzy spoken-name to package-name resolution
 * for zero-touch hands-free app navigation.
 *
 * Supports multi-lingual aliases (English, Hindi, Hinglish) and fuzzy string distance matching.
 * BIBLE Reference: PRD §7 Sprint 6
 */
class AppIndexResolver(private val context: Context) {

    companion object {
        private const val TAG = "IND_APP_RESOLVER"

        // Canonical core OS and popular app package mappings
        val CORE_APP_MAPPINGS = mapOf(
            "whatsapp" to "com.whatsapp",
            "व्हाट्सएप" to "com.whatsapp",
            "youtube" to "com.google.android.youtube",
            "यूट्यूब" to "com.google.android.youtube",
            "chrome" to "com.android.chrome",
            "browser" to "com.android.chrome",
            "क्रोम" to "com.android.chrome",
            "camera" to "com.android.camera2",
            "कैमरा" to "com.android.camera2",
            "settings" to "com.android.settings",
            "सेटिंग्स" to "com.android.settings",
            "phone" to "com.android.dialer",
            "dialer" to "com.android.dialer",
            "फोन" to "com.android.dialer",
            "messages" to "com.google.android.apps.messaging",
            "sms" to "com.google.android.apps.messaging",
            "संदेश" to "com.google.android.apps.messaging",
            "clock" to "com.google.android.deskclock",
            "alarm" to "com.google.android.deskclock",
            "घड़ी" to "com.google.android.deskclock",
            "अलार्म" to "com.google.android.deskclock",
            "calculator" to "com.google.android.calculator",
            "कैलकुलेटर" to "com.google.android.calculator",
            "gallery" to "com.google.android.apps.photos",
            "photos" to "com.google.android.apps.photos",
            "तस्वीर" to "com.google.android.apps.photos",
            "maps" to "com.google.android.apps.maps",
            "नक्शा" to "com.google.android.apps.maps"
        )
    }

    private val packageIndex = mutableMapOf<String, String>()

    init {
        rebuildIndex()
    }

    /**
     * Builds or refreshes the index of installed applications from PackageManager.
     */
    fun rebuildIndex() {
        val startNs = System.nanoTime()
        packageIndex.clear()
        packageIndex.putAll(CORE_APP_MAPPINGS)

        try {
            val pm = context.packageManager
            val mainIntent = Intent(Intent.ACTION_MAIN, null).apply {
                addCategory(Intent.CATEGORY_LAUNCHER)
            }
            val resolvedApps = pm.queryIntentActivities(mainIntent, 0)
            for (info in resolvedApps) {
                val label = info.loadLabel(pm).toString().lowercase().trim()
                val pkgName = info.activityInfo.packageName
                packageIndex[label] = pkgName
            }
        } catch (t: Throwable) {
            Log.w(TAG, "PackageManager query encountered non-fatal error (e.g. test environment): ${t.message}")
        }

        val elapsedMs = (System.nanoTime() - startNs) / 1_000_000.0
        Log.i(TAG, "AppIndexResolver indexed ${packageIndex.size} package mappings in %.2f ms".format(elapsedMs))
    }

    /**
     * Resolves spoken name to Android package name in < 15 ms.
     * Checks exact match -> substring match -> fuzzy Levenshtein distance match.
     */
    fun resolvePackage(spokenName: String): String? {
        val startNs = System.nanoTime()
        val query = spokenName.lowercase().trim()
        if (query.isEmpty()) return null

        // 1. Exact match
        packageIndex[query]?.let { return it }

        // 2. Substring match
        for ((key, pkg) in packageIndex) {
            if (key.contains(query) || query.contains(key)) {
                val elapsedMs = (System.nanoTime() - startNs) / 1_000_000.0
                Log.d(TAG, "Resolved '$spokenName' -> '$pkg' via substring in %.2f ms".format(elapsedMs))
                return pkg
            }
        }

        // 3. Fuzzy Levenshtein match (max distance 2)
        var bestMatch: String? = null
        var minDistance = Int.MAX_VALUE

        for ((key, pkg) in packageIndex) {
            val dist = computeLevenshteinDistance(query, key)
            if (dist < minDistance && dist <= 2) {
                minDistance = dist
                bestMatch = pkg
            }
        }

        val elapsedMs = (System.nanoTime() - startNs) / 1_000_000.0
        if (bestMatch != null) {
            Log.d(TAG, "Resolved '$spokenName' -> '$bestMatch' via fuzzy distance ($minDistance) in %.2f ms".format(elapsedMs))
        } else {
            Log.w(TAG, "Could not resolve app name '$spokenName' (checked in %.2f ms)".format(elapsedMs))
        }
        return bestMatch
    }

    private fun computeLevenshteinDistance(s1: String, s2: String): Int {
        val dp = Array(s1.length + 1) { IntArray(s2.length + 1) }
        for (i in 0..s1.length) dp[i][0] = i
        for (j in 0..s2.length) dp[0][j] = j

        for (i in 1..s1.length) {
            for (j in 1..s2.length) {
                val cost = if (s1[i - 1] == s2[j - 1]) 0 else 1
                dp[i][j] = minOf(
                    dp[i - 1][j] + 1,
                    dp[i][j - 1] + 1,
                    dp[i - 1][j - 1] + cost
                )
            }
        }
        return dp[s1.length][s2.length]
    }
}
