// ==============================================================================
// Ind OS — Neural Core IPC Interface Contract
// Subsystem: ind-neural-core / Layer 4 Privileged Service
// BIBLE v1.0.1 (Amendment A-001)
// ==============================================================================

package indos.neural.ipc;

import indos.neural.ipc.IIndStreamCallback;

interface IIndAIManager {
    /** Fast synchronous threat/scam classification (< 60 ms). */
    String classifyFast(in String payload, in String domain);

    /** Compiles NL command (En/Hi/Hinglish) + screen graph into deterministic OS actions. */
    oneway void executeZeroTouchCommand(
        in String userUtterance,
        in String screenGraphJson,
        in IIndStreamCallback callback);

    /** Immediately evicts mmap-backed GGUF weights from physical RAM pages. */
    void evictModelWeights();

    /** Real-time telemetry: RAM usage MB, tokens/sec, loaded model ID. */
    String getEngineStatusJson();
}
