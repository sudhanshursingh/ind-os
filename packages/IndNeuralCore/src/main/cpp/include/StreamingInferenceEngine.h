// ==============================================================================
// Ind OS — Streaming JNI Inference Engine Header
// Subsystem: ind-neural-core / Layer 3 Native AI Daemon
// Reference Silicon: Snapdragon 732G (SM7150-AC / Kryo 470 / Adreno 618)
// DoD Reference: ≥15 tok/s sustained, RSS ≤ 1050 MB, eviction < 15 ms
// ==============================================================================

#ifndef IND_STREAMING_INFERENCE_ENGINE_H
#define IND_STREAMING_INFERENCE_ENGINE_H

#include "MmapModelPager.h"

#include <functional>
#include <string>
#include <vector>

namespace indos {
namespace native {

struct InferenceBenchmarkMetrics {
    int total_tokens = 0;
    double total_generation_ms = 0.0;
    double tokens_per_second = 0.0;
    double peak_rss_mb = 0.0;
    double eviction_duration_ms = 0.0;
    double evicted_mb = 0.0;
    bool passed_dod = false;
};

class StreamingInferenceEngine {
public:
    static StreamingInferenceEngine& GetInstance();

    // Executes streaming token generation calling tokenCb for each token and completeCb at the end
    void ExecuteStreamingInference(
        const std::string& prompt,
        const std::string& screenGraphJson,
        std::function<void(const std::string& token)> tokenCb,
        std::function<void(const std::string& actionJson)> completeCb,
        std::function<void(int errCode, const std::string& msg)> errorCb
    );

    // Runs full Sprint 4 DoD benchmark and returns JSON report
    InferenceBenchmarkMetrics RunBenchmark(int tokenCount = 64);
    std::string RunBenchmarkJson(int tokenCount = 64);

private:
    StreamingInferenceEngine();
    ~StreamingInferenceEngine() = default;

    StreamingInferenceEngine(const StreamingInferenceEngine&) = delete;
    StreamingInferenceEngine& operator=(const StreamingInferenceEngine&) = delete;

    std::string CompileActionJsonFromPrompt(const std::string& prompt, const std::string& screenGraphJson);
};

} // namespace native
} // namespace indos

#endif // IND_STREAMING_INFERENCE_ENGINE_H
