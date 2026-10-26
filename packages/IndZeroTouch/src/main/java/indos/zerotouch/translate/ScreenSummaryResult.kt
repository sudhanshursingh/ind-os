package indos.zerotouch.translate

// ==============================================================================
// Ind OS — Screen Translate & Summarize Result Data Model
// ==============================================================================

/**
 * Encapsulates the 3-bullet Hindi visual + spoken summary produced from active screen content.
 */
data class ScreenSummaryResult(
    val title: String,
    val extractedTextLength: Int,
    val hindiBullets: List<String>,
    val spokenHindiText: String,
    val extractionLatencyMs: Long,
    val summarizationLatencyMs: Long,
    val totalLatencyMs: Long,
    val passesLatencyDoD: Boolean
)
