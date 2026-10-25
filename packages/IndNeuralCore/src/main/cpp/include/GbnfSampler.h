// ==============================================================================
// Ind OS — Deterministic GBNF Sampler Header
// Subsystem: ind-neural-core / Layer 3 Native AI Daemon
// BIBLE Reference: PRD §2 (D6), §4.1-B & §7 Sprint 5
// ==============================================================================

#ifndef IND_GBNF_SAMPLER_H
#define IND_GBNF_SAMPLER_H

#include <limits>
#include <string>
#include <vector>

namespace indos {
namespace native {

struct TokenCandidate {
    int token_id = 0;
    std::string text = "";
    float logit = 0.0f;
};

class GbnfSampler {
public:
    static GbnfSampler& GetInstance();

    // Initializes or resets the GBNF grammar rules (defaults to ind_action.gbnf)
    void InitGrammar(const std::string& gbnfRules = "");

    // Applies grammar transition mask; invalid token logits set to -infinity
    void ApplyGrammarMask(const std::string& currentPrefix, std::vector<TokenCandidate>& candidates);

    // Strictly validates complete generated JSON string against ind_action.gbnf schema
    bool ValidateActionJson(const std::string& jsonStr, std::string& outErrorReason) const;

    // Returns the active GBNF grammar text
    std::string GetGrammarRules() const { return active_grammar_; }

private:
    GbnfSampler();
    ~GbnfSampler() = default;

    GbnfSampler(const GbnfSampler&) = delete;
    GbnfSampler& operator=(const GbnfSampler&) = delete;

    std::string active_grammar_;
    static const char* kDefaultActionGbnf;
};

} // namespace native
} // namespace indos

#endif // IND_GBNF_SAMPLER_H
