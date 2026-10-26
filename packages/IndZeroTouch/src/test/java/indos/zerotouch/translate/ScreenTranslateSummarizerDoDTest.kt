package indos.zerotouch.translate

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Sprint 14 Definition of Done (DoD) Test: On-Screen Translator & Summarizer
 * DoD: English government circular on screen -> Hindi 3-bullet spoken+visual summary < 3 s.
 */
class ScreenTranslateSummarizerDoDTest {

    private val summarizer = ScreenTranslateSummarizer()

    @Test
    fun testEnglishGovernmentCircular_ProducesHindiThreeBulletSummary_Under3Seconds() {
        val englishCircularText = """
            GOVERNMENT OF INDIA
            MINISTRY OF FINANCE — DEPARTMENT OF REVENUE
            CENTRAL BOARD OF DIRECT TAXES (CBDT)
            
            CIRCULAR NO. 03/2026: MANDATORY LINKING OF PAN WITH AADHAAR
            
            In accordance with the provisions of Section 139AA of the Income-tax Act, 1961, 
            every individual who has been allotted a Permanent Account Number (PAN) is required 
            to intimate their Aadhaar number to the prescribed authority.
            
            It is hereby notified that all taxpayers must complete the mandatory linking of PAN 
            with Aadhaar before the extended deadline of 31 March. In order to complete the linking, 
            a prescribed late fee of Rs. 1,000 must be deposited online through the official e-filing portal.
            
            CONSEQUENCES OF NON-COMPLIANCE:
            If a taxpayer fails to link their PAN with Aadhaar by the stipulated deadline, 
            the PAN shall become inoperative with immediate effect. An inoperative PAN will result in:
            1. Inability to file income tax returns or claim refunds.
            2. Immediate halt on banking operations, opening fixed deposits, and mutual fund transactions.
            3. Higher rate of Tax Deducted at Source (TDS) applicable on all domestic payments.
            
            All citizens are strongly advised to verify their status on the e-filing portal immediately.
        """.trimIndent()

        println("=== Starting Sprint 14 DoD: On-Screen Translator & Summarizer Test ===")
        val startTime = System.currentTimeMillis()

        // Execute on-screen translation and summarization pipeline
        val result = summarizer.summarizeText(
            rawText = englishCircularText,
            documentTitle = "Income Tax CBDT Circular 03/2026",
            overallStartTimeMs = startTime
        )

        val duration = System.currentTimeMillis() - startTime
        println("=== Sprint 14 DoD Result Telemetry ===")
        println("Document Title: ${result.title}")
        println("Raw Text Length: ${result.extractedTextLength} chars")
        println("Extraction Latency: ${result.extractionLatencyMs} ms")
        println("Summarization Latency: ${result.summarizationLatencyMs} ms")
        println("Total E2E Latency: ${result.totalLatencyMs} ms (DoD Ceiling: <3,000 ms)")
        println("Hindi Bullets Generated: ${result.hindiBullets.size}")
        result.hindiBullets.forEachIndexed { i, bullet ->
            println("  [Bullet ${i + 1}] $bullet")
        }
        println("Spoken TTS Content: '${result.spokenHindiText}'")
        println("Passes Latency DoD (<3s): ${result.passesLatencyDoD}")

        // Invariant 1: Exactly 3 Hindi bullets
        assertEquals(3, result.hindiBullets.size)

        // Invariant 2: Total Latency < 3.0 seconds (DoD requirement)
        assertTrue(
            "Total latency must be < 3,000 ms (< 3 s), was: ${result.totalLatencyMs} ms",
            result.totalLatencyMs < 3000L
        )
        assertTrue("passesLatencyDoD flag must be true", result.passesLatencyDoD)

        // Invariant 3: Semantic factual content check (PAN, Aadhaar, Deadline, Fee)
        val combinedHindi = result.hindiBullets.joinToString(" ")
        assertTrue("Must mention PAN card in Hindi", combinedHindi.contains("पैन"))
        assertTrue("Must mention Aadhaar in Hindi", combinedHindi.contains("आधार"))
        assertTrue("Must extract 31 March deadline", combinedHindi.contains("31 मार्च"))
        assertTrue("Must extract ₹1,000 fee", combinedHindi.contains("₹1,000"))
        assertTrue("Must mention inoperative or banking disruption", combinedHindi.contains("निष्क्रिय") || combinedHindi.contains("लेनदेन"))

        // Invariant 4: Spoken Hindi format for offline TTS
        assertNotNull(result.spokenHindiText)
        assertTrue(result.spokenHindiText.startsWith("बिंदु 1:"))
        assertTrue(result.spokenHindiText.contains("बिंदु 2:"))
        assertTrue(result.spokenHindiText.contains("बिंदु 3:"))
    }

    @Test
    fun testTrafficChallanNotice_ProducesHindiThreeBulletSummary() {
        val trafficNoticeText = """
            DELHI TRAFFIC POLICE — NOTICE OF VIOLATION
            E-Challan Notice No: DL-2026-90412
            Vehicle No: DL-01-AB-1234
            Offense: Overspeeding on Ring Road (Recorded Speed: 84 km/h in 60 km/h zone)
            Fine Amount: Rs. 2,000
            Due Date: 15 November
            Please pay the fine through the official Parivahan portal to avoid legal summons.
        """.trimIndent()

        val result = summarizer.summarizeText(trafficNoticeText, "Delhi Traffic E-Challan")

        assertEquals(3, result.hindiBullets.size)
        assertTrue(result.totalLatencyMs < 3000L)
        val text = result.hindiBullets.joinToString(" ")
        assertTrue("Must mention challan", text.contains("चालान"))
        assertTrue("Must mention fine amount ₹2,000", text.contains("₹2,000"))
        assertTrue("Must mention Parivahan portal or payment", text.contains("परिवहन") || text.contains("ऑनलाइन"))
    }
}
