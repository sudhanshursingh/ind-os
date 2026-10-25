package indos.neural

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Sprint 5 DoD Verification Test Suite:
 * Validates 200/200 diverse natural language prompts across English, Hindi, and Hinglish.
 * Asserts 100% valid JSON adherence to ind_action.gbnf on first attempt.
 */
class PromptCompilerGbnfTest {

    @Test
    fun testPromptCompilationStructure() {
        val prompt = PromptCompiler.compilePrompt("Torch chalu karo", "{\"nodes\":[]}")
        assertNotNull(prompt)
        assertTrue("Must include system tag", prompt.contains("<system>"))
        assertTrue("Must include user tag", prompt.contains("<user>"))
        assertTrue("Must include command", prompt.contains("Torch chalu karo"))
        assertTrue("Must include screen graph", prompt.contains("<screen_graph>"))
        assertTrue("Must include assistant prompt tag", prompt.contains("<assistant>"))
    }

    @Test
    fun testTwoHundredDiversePromptsAdhereToGbnfGrammar() {
        val testPrompts = mutableListOf<String>()

        // 1. English Prompts (70 prompts)
        val enActions = listOf(
            "Turn on flashlight", "Turn off flashlight", "Switch on torch", "Disable torch",
            "Turn on Wi-Fi", "Turn off Wi-Fi", "Disconnect Wi-Fi", "Connect to Wi-Fi",
            "Enable Bluetooth", "Disable Bluetooth", "Turn on Bluetooth", "Turn off Bluetooth",
            "Set volume to 80 percent", "Set volume to 50 percent", "Set volume to 100 percent", "Mute volume",
            "Increase brightness to 80 percent", "Set brightness to 50 percent", "Dim brightness to 30 percent",
            "Turn on Do Not Disturb", "Disable DND", "Enable DND mode",
            "Open WhatsApp", "Launch YouTube", "Open Google Chrome", "Open Camera", "Open Settings",
            "Call Maa", "Call Papa", "Call Police", "Dial 112", "Call Ambulance", "Dial 108",
            "Read screen", "Read notification", "Read the current screen"
        )
        testPrompts.addAll(enActions)
        // Compound English prompts
        for (i in 0 until 34) {
            val a = enActions[i % enActions.size]
            val b = enActions[(i + 7) % enActions.size]
            testPrompts.add("$a and $b")
        }

        // 2. Hindi Devanagari Prompts (70 prompts)
        val hiActions = listOf(
            "टॉर्च चालू करो", "टॉर्च बंद करो", "फ़्लैशलाइट ऑन करो", "टॉर्च बुझा दो",
            "वाईफाई चालू करो", "वाईफाई बंद करो", "वाई-फाई ऑन करो", "वाईफाई डिस्कनेक्ट करो",
            "ब्लूटूथ चालू करो", "ब्लूटूथ बंद करो", "ब्लूटूथ ऑन करो", "ब्लूटूथ ऑफ करो",
            "वॉल्यूम अस्सी प्रतिशत करो", "वॉल्यूम पचास प्रतिशत करो", "वॉल्यूम सौ प्रतिशत करो", "आवाज बंद करो",
            "ब्राइटनेस बढ़ाओ", "स्क्रीन की रोशनी पचास प्रतिशत करो", "स्क्रीन ब्राइटनेस कम करो",
            "डू नॉट डिस्टर्ब चालू करो", "शांत मोड ऑन करो", "डीएनडी बंद करो",
            "व्हाट्सएप खोलो", "यूट्यूब खोलो", "क्रोम ब्राउज़र खोलो", "कैमरा खोलो", "सेटिंग्स खोलो",
            "मां को फोन लगाओ", "पापा को कॉल करो", "पुलिस को फोन करो", "एंबुलेंस को कॉल करो", "112 पर फोन लगाओ",
            "स्क्रीन पढ़ो", "स्क्रीन पर क्या लिखा है बताओ"
        )
        testPrompts.addAll(hiActions)
        // Compound Hindi prompts
        for (i in 0 until 36) {
            val a = hiActions[i % hiActions.size]
            val b = hiActions[(i + 5) % hiActions.size]
            testPrompts.add("$a और $b")
        }

        // 3. Hinglish Code-Switched Prompts (60 prompts)
        val hinglishActions = listOf(
            "Torch chalu karo", "Torch band kar do", "Flashlight on karo please", "Torch bujha do",
            "WiFi on karo", "WiFi band karo", "WiFi disconnect kar do", "WiFi chalu karo",
            "Bluetooth on karo", "Bluetooth band kar do", "Bluetooth connect karo",
            "Volume 80 percent kar do", "Volume aadha kar do", "Volume full kar do", "Sound mute karo",
            "Screen brightness badhao", "Roshni 50 percent karo", "Brightness kam kar do",
            "DND mode on karo", "Do not disturb chalu kar do",
            "WhatsApp kholo", "YouTube open karo", "Chrome browser kholo", "Camera on karo", "Settings kholo",
            "Maa ko call lagao", "Papa ko phone karo", "Police ko call lagao", "Ambulance bulao 108",
            "Screen padho", "Screen padh ke sunao"
        )
        testPrompts.addAll(hinglishActions)
        // Compound Hinglish prompts (e.g. flagship prompt from PRD)
        testPrompts.add("Torch chalu karo aur volume 80 percent kar do")
        testPrompts.add("WiFi on karo aur Bluetooth on karo")
        testPrompts.add("WhatsApp kholo aur Maa ko phone lagao")
        testPrompts.add("Screen brightness 70 percent karo aur volume 80 percent karo")
        testPrompts.add("Torch on karo aur camera open karo")
        for (i in 0 until 24) {
            val a = hinglishActions[i % hinglishActions.size]
            val b = hinglishActions[(i + 4) % hinglishActions.size]
            testPrompts.add("$a aur $b")
        }

        assertEquals("Total test prompts must equal exactly 200", 200, testPrompts.size)

        var passedCount = 0
        val failures = mutableListOf<String>()

        for ((index, prompt) in testPrompts.withIndex()) {
            val actionJson = PromptCompiler.compileAction(prompt)
            assertNotNull("Output JSON must not be null for prompt #$index: '$prompt'", actionJson)
            assertTrue("Output JSON must not be empty for prompt #$index: '$prompt'", actionJson.isNotBlank())

            // Validate against ind_action.gbnf
            val isValid = PromptCompiler.validateActionJson(actionJson)
            val nativeErr = NativeBridge.validateGbnf(actionJson)

            if (isValid && nativeErr.isEmpty()) {
                passedCount++
            } else {
                failures.add("Prompt #$index failed: '$prompt' -> JSON: '$actionJson' (Error: $nativeErr)")
            }
        }

        if (failures.isNotEmpty()) {
            println("--- SPRINT 5 GBNF VALIDATION FAILURES (${failures.size}/200) ---")
            failures.forEach { println(it) }
        }

        assertEquals(
            "DoD Requirement: 200/200 diverse prompts must parse as valid GBNF JSON on first attempt",
            200,
            passedCount
        )
        println("[IND_BENCHMARK] Sprint 5 GBNF Prompt Validation Result: $passedCount / 200 PASSED (100.0% Success)")
    }
}
