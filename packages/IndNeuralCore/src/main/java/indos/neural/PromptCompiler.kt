package indos.neural

/**
 * PromptCompiler prepares structured multi-lingual few-shot prompts
 * for Qwen2.5-1.5B / Llama-3.2-1B and enforces deterministic GBNF action routing.
 *
 * Supports:
 * - English
 * - Hindi (Devanagari)
 * - Hinglish (Roman-script Hindi)
 * - Compressed screen graph context (<screen_graph>)
 *
 * BIBLE Reference: PRD §4.1-B & §7 Sprint 5
 */
object PromptCompiler {

    private const val SYSTEM_PROMPT = """You are Ind OS Neural Core, a deterministic on-device operating system action compiler.
You strictly convert user commands into valid JSON adhering to the ind_action.gbnf grammar schema.

Schema definition:
{
  "tier": "GREEN" | "AMBER" | "RED",
  "actions": [
    {
      "type": "SYS_TOGGLE" | "OPEN_APP" | "CLICK_NODE" | "INPUT_TEXT" | "SCROLL" | "READ_SCREEN" | "PLACE_CALL",
      "target": "<string>",
      "params": { "value": <string | number | boolean> }
    }
  ],
  "tts": "<confirmation speech string>"
}

Few-shot examples:
[English]
User: "Turn on torch"
Assistant: {"tier":"GREEN","actions":[{"type":"SYS_TOGGLE","target":"flashlight","params":{"value":true}}],"tts":"Torch turned on"}

[Hindi]
User: "व्हाट्सएप खोलो और मां को कॉल लगाओ"
Assistant: {"tier":"GREEN","actions":[{"type":"OPEN_APP","target":"com.whatsapp","params":{}},{"type":"PLACE_CALL","target":"Maa","params":{}}],"tts":"Opening WhatsApp and Calling Maa"}

[Hinglish]
User: "Torch chalu karo aur volume 80 percent kar do"
Assistant: {"tier":"GREEN","actions":[{"type":"SYS_TOGGLE","target":"flashlight","params":{"value":true}},{"type":"SYS_TOGGLE","target":"volume","params":{"value":80}}],"tts":"Torch turned on and Volume set to 80%"}"""

    /**
     * Compiles user utterance and optional screen hierarchy into full LLM input prompt.
     */
    fun compilePrompt(userUtterance: String, screenGraphJson: String? = null): String {
        val screenContext = if (!screenGraphJson.isNullOrBlank() && screenGraphJson != "{}") {
            "\n<screen_graph>\n$screenGraphJson\n</screen_graph>"
        } else {
            ""
        }

        return """<system>
$SYSTEM_PROMPT
</system>
<user>$screenContext
Command: $userUtterance
</user>
<assistant>
"""
    }

    /**
     * Compiles user utterance into deterministic GBNF action JSON.
     */
    fun compileAction(userUtterance: String, screenGraphJson: String? = null): String {
        return NativeBridge.compileAction(userUtterance, screenGraphJson ?: "")
    }

    /**
     * Validates an action JSON payload against ind_action.gbnf.
     * Returns true if 100% compliant, false otherwise.
     */
    fun validateActionJson(actionJson: String): Boolean {
        val err = NativeBridge.validateGbnf(actionJson)
        return err.isEmpty()
    }
}
