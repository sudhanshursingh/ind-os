// ==============================================================================
// Ind OS — Deterministic GBNF Sampler Implementation
// Subsystem: ind-neural-core / Layer 3 Native AI Daemon
// BIBLE Reference: PRD §2 (D6), §4.1-B & §7 Sprint 5
// ==============================================================================

#include "include/GbnfSampler.h"
#include "include/ind_native.h"

#include <android/log.h>
#include <cctype>
#include <cmath>
#include <sstream>
#include <vector>

namespace indos {
namespace native {

const char* GbnfSampler::kDefaultActionGbnf =
    "root   ::= \"{\" ws \"\\\"tier\\\":\" ws tier \",\" ws \"\\\"actions\\\":\" ws \"[\" ws action (\",\" ws action)* ws \"],\" ws \"\\\"tts\\\":\" ws string ws \"}\"\n"
    "tier   ::= \"\\\"GREEN\\\"\" | \"\\\"AMBER\\\"\" | \"\\\"RED\\\"\"\n"
    "action ::= \"{\" ws \"\\\"type\\\":\" ws action-type \",\" ws \"\\\"target\\\":\" ws string \",\" ws \"\\\"params\\\":\" ws params ws \"}\"\n"
    "action-type ::= \"\\\"SYS_TOGGLE\\\"\" | \"\\\"OPEN_APP\\\"\" | \"\\\"CLICK_NODE\\\"\" | \"\\\"INPUT_TEXT\\\"\" | \"\\\"SCROLL\\\"\" | \"\\\"READ_SCREEN\\\"\" | \"\\\"PLACE_CALL\\\"\"\n"
    "params ::= \"{\" ws (\"\\\"value\\\":\" ws (string | number | boolean))? ws \"}\"\n"
    "string ::= \"\\\"\" ([^\"\\\\] | \"\\\\\" .)* \"\\\"\"\n"
    "number ::= \"-\"? [0-9]+ (\".\" [0-9]+)?\n"
    "boolean ::= \"true\" | \"false\"\n"
    "ws     ::= [ \\t\\n]*\n";

GbnfSampler& GbnfSampler::GetInstance() {
    static GbnfSampler instance;
    return instance;
}

GbnfSampler::GbnfSampler() {
    InitGrammar();
}

void GbnfSampler::InitGrammar(const std::string& gbnfRules) {
    if (!gbnfRules.empty()) {
        active_grammar_ = gbnfRules;
    } else {
        active_grammar_ = kDefaultActionGbnf;
    }
}

void GbnfSampler::ApplyGrammarMask(
    const std::string& currentPrefix,
    std::vector<TokenCandidate>& candidates
) {
    const float kNegInf = -std::numeric_limits<float>::infinity();

    for (auto& cand : candidates) {
        std::string nextText = currentPrefix + cand.text;

        // Basic structural prefix validation ensuring JSON brackets and quotes stay balanced
        int openBraces = 0;
        int openBrackets = 0;
        bool inQuote = false;
        bool escape = false;
        bool malformed = false;

        for (char c : nextText) {
            if (escape) {
                escape = false;
                continue;
            }
            if (c == '\\') {
                escape = true;
                continue;
            }
            if (c == '"') {
                inQuote = !inQuote;
                continue;
            }
            if (!inQuote) {
                if (c == '{') openBraces++;
                else if (c == '}') {
                    openBraces--;
                    if (openBraces < 0) { malformed = true; break; }
                } else if (c == '[') openBrackets++;
                else if (c == ']') {
                    openBrackets--;
                    if (openBrackets < 0) { malformed = true; break; }
                }
            }
        }

        if (malformed) {
            cand.logit = kNegInf;
        }
    }
}

static bool CheckValidActionType(const std::string& type) {
    return type == "SYS_TOGGLE" ||
           type == "OPEN_APP" ||
           type == "CLICK_NODE" ||
           type == "INPUT_TEXT" ||
           type == "SCROLL" ||
           type == "READ_SCREEN" ||
           type == "PLACE_CALL";
}

bool GbnfSampler::ValidateActionJson(const std::string& jsonStr, std::string& outErrorReason) const {
    if (jsonStr.empty()) {
        outErrorReason = "Empty JSON payload";
        return false;
    }

    // Required top-level fields
    if (jsonStr.find("\"tier\"") == std::string::npos) {
        outErrorReason = "Missing required top-level field: 'tier'";
        return false;
    }
    if (jsonStr.find("\"actions\"") == std::string::npos) {
        outErrorReason = "Missing required top-level field: 'actions'";
        return false;
    }
    if (jsonStr.find("\"tts\"") == std::string::npos) {
        outErrorReason = "Missing required top-level field: 'tts'";
        return false;
    }

    // Tier must be GREEN, AMBER, or RED
    bool validTier = (jsonStr.find("\"GREEN\"") != std::string::npos ||
                      jsonStr.find("\"AMBER\"") != std::string::npos ||
                      jsonStr.find("\"RED\"") != std::string::npos);
    if (!validTier) {
        outErrorReason = "Invalid tier value. Must be GREEN, AMBER, or RED.";
        return false;
    }

    // Verify balance of braces and brackets
    int braces = 0;
    int brackets = 0;
    bool inQuote = false;
    bool escape = false;

    for (char c : jsonStr) {
        if (escape) {
            escape = false;
            continue;
        }
        if (c == '\\') {
            escape = true;
            continue;
        }
        if (c == '"') {
            inQuote = !inQuote;
            continue;
        }
        if (!inQuote) {
            if (c == '{') braces++;
            else if (c == '}') braces--;
            else if (c == '[') brackets++;
            else if (c == ']') brackets--;
        }
    }

    if (inQuote) {
        outErrorReason = "Unterminated string literal";
        return false;
    }
    if (braces != 0) {
        outErrorReason = "Mismatched curly braces: " + std::to_string(braces);
        return false;
    }
    if (brackets != 0) {
        outErrorReason = "Mismatched square brackets: " + std::to_string(brackets);
        return false;
    }

    // If actions are specified, verify action-types
    size_t actPos = jsonStr.find("\"type\"");
    while (actPos != std::string::npos) {
        size_t colon = jsonStr.find(':', actPos);
        if (colon != std::string::npos) {
            size_t q1 = jsonStr.find('"', colon);
            if (q1 != std::string::npos) {
                size_t q2 = jsonStr.find('"', q1 + 1);
                if (q2 != std::string::npos) {
                    std::string actType = jsonStr.substr(q1 + 1, q2 - q1 - 1);
                    if (!CheckValidActionType(actType)) {
                        outErrorReason = "Invalid action-type: '" + actType + "'";
                        return false;
                    }
                }
            }
        }
        actPos = jsonStr.find("\"type\"", actPos + 6);
    }

    outErrorReason = "";
    return true;
}

} // namespace native
} // namespace indos
