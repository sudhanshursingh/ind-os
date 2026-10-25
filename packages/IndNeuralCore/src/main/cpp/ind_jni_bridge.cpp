// ==============================================================================
// Ind OS — JNI Bridge for Native AI Subsystem (libind_ai.so)
// Subsystem: ind-neural-core / Layer 3 Native AI Daemon
// Reference Silicon: Snapdragon 732G (SM7150-AC / Kryo 470 / Adreno 618)
// ==============================================================================

#include "include/ind_native.h"
#include "include/MmapModelPager.h"
#include "include/StreamingInferenceEngine.h"

#include <jni.h>
#include <android/log.h>
#include <string>

using namespace indos::native;

extern "C" {

JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM* /* vm */, void* /* reserved */) {
    __android_log_print(
        ANDROID_LOG_INFO,
        IND_NATIVE_TAG,
        "[IND_NATIVE] libind_ai.so loaded successfully via JNI! Version: 1.0.0-sprint4. Target: Snapdragon 732G (Kryo 470 + Adreno 618)."
    );
    return JNI_VERSION_1_6;
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_runHardwareProbe(JNIEnv* env, jobject /* this */) {
    std::string report = HardwareProbe::GenerateHardwareReportJson();
    return env->NewStringUTF(report.c_str());
}

JNIEXPORT jdouble JNICALL
Java_indos_neural_NativeBridge_runCpuGemmBenchmark(JNIEnv* /* env */, jobject /* this */, jint threads, jint dim) {
    BenchmarkMetrics metrics = HardwareProbe::RunCpuGemmBenchmark(threads, dim);
    return static_cast<jdouble>(metrics.gflops);
}

JNIEXPORT jdouble JNICALL
Java_indos_neural_NativeBridge_runVulkanComputeBenchmark(JNIEnv* /* env */, jobject /* this */, jint dim) {
    BenchmarkMetrics metrics = HardwareProbe::RunVulkanComputeBenchmark(dim);
    return static_cast<jdouble>(metrics.gflops);
}

JNIEXPORT jboolean JNICALL
Java_indos_neural_NativeBridge_evictMmapRange(JNIEnv* /* env */, jobject /* this */, jlong addr, jlong length) {
    void* ptr = reinterpret_cast<void*>(addr);
    size_t len = static_cast<size_t>(length);
    bool success = HardwareProbe::EvictMmapRange(ptr, len);
    return success ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_getNativeStatus(JNIEnv* env, jobject /* this */) {
    std::string status = "libind_ai.so: ACTIVE (ARMv8.2-A DotProd + FP16 SIMD + Adreno 618 Vulkan + Zero-Copy mmap)";
    return env->NewStringUTF(status.c_str());
}

// ==============================================================================
// Sprint 4: Zero-Copy mmap & Streaming JNI Inference Exports
// ==============================================================================

JNIEXPORT jboolean JNICALL
Java_indos_neural_NativeBridge_loadMmapModel(JNIEnv* env, jobject /* this */, jstring jPath) {
    const char* pathStr = env->GetStringUTFChars(jPath, nullptr);
    std::string path(pathStr);
    env->ReleaseStringUTFChars(jPath, pathStr);

    bool ok = MmapModelPager::GetInstance().LoadModel(path);
    return ok ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_evictMmapModel(JNIEnv* env, jobject /* this */) {
    EvictionResult result = MmapModelPager::GetInstance().EvictWeights();
    std::string json = "{"
        "\"success\":" + std::string(result.success ? "true" : "false") + ","
        "\"durationMs\":" + std::to_string(result.duration_ms) + ","
        "\"rssBeforeMb\":" + std::to_string(result.rss_before_mb) + ","
        "\"rssAfterMb\":" + std::to_string(result.rss_after_mb) + ","
        "\"freedMb\":" + std::to_string(result.freed_mb) +
    "}";
    return env->NewStringUTF(json.c_str());
}

JNIEXPORT jboolean JNICALL
Java_indos_neural_NativeBridge_prefetchMmapModel(JNIEnv* /* env */, jobject /* this */) {
    bool ok = MmapModelPager::GetInstance().PrefetchWeights();
    return ok ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_getMmapTelemetry(JNIEnv* env, jobject /* this */) {
    std::string telemetry = MmapModelPager::GetInstance().GetTelemetryJson();
    return env->NewStringUTF(telemetry.c_str());
}

JNIEXPORT void JNICALL
Java_indos_neural_NativeBridge_executeStreamingInference(
    JNIEnv* env,
    jobject /* this */,
    jstring jPrompt,
    jstring jScreenGraph,
    jobject callback
) {
    const char* pStr = env->GetStringUTFChars(jPrompt, nullptr);
    std::string prompt = pStr ? pStr : "";
    env->ReleaseStringUTFChars(jPrompt, pStr);

    std::string screenGraph = "";
    if (jScreenGraph != nullptr) {
        const char* sgStr = env->GetStringUTFChars(jScreenGraph, nullptr);
        if (sgStr) {
            screenGraph = sgStr;
            env->ReleaseStringUTFChars(jScreenGraph, sgStr);
        }
    }

    if (callback == nullptr) {
        return;
    }

    jclass cbClass = env->GetObjectClass(callback);
    jmethodID onTokenMethod = env->GetMethodID(cbClass, "onNativeToken", "(Ljava/lang/String;)V");
    jmethodID onCompleteMethod = env->GetMethodID(cbClass, "onNativeComplete", "(Ljava/lang/String;)V");
    jmethodID onErrorMethod = env->GetMethodID(cbClass, "onNativeError", "(ILjava/lang/String;)V");

    if (onTokenMethod == nullptr || onCompleteMethod == nullptr) {
        __android_log_print(ANDROID_LOG_ERROR, IND_BENCHMARK_TAG, "Failed to resolve callback method IDs");
        return;
    }

    StreamingInferenceEngine::GetInstance().ExecuteStreamingInference(
        prompt,
        screenGraph,
        [env, callback, onTokenMethod](const std::string& token) {
            jstring jTok = env->NewStringUTF(token.c_str());
            env->CallVoidMethod(callback, onTokenMethod, jTok);
            env->DeleteLocalRef(jTok);
        },
        [env, callback, onCompleteMethod](const std::string& actionJson) {
            jstring jAct = env->NewStringUTF(actionJson.c_str());
            env->CallVoidMethod(callback, onCompleteMethod, jAct);
            env->DeleteLocalRef(jAct);
        },
        [env, callback, onErrorMethod](int errCode, const std::string& msg) {
            if (onErrorMethod != nullptr) {
                jstring jMsg = env->NewStringUTF(msg.c_str());
                env->CallVoidMethod(callback, onErrorMethod, static_cast<jint>(errCode), jMsg);
                env->DeleteLocalRef(jMsg);
            }
        }
    );
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_runSprint4Benchmark(JNIEnv* env, jobject /* this */, jint tokenCount) {
    std::string report = StreamingInferenceEngine::GetInstance().RunBenchmarkJson(tokenCount);
    return env->NewStringUTF(report.c_str());
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_validateGbnfGrammar(JNIEnv* env, jobject /* this */, jstring jsonStr) {
    const char* str = env->GetStringUTFChars(jsonStr, nullptr);
    std::string input(str ? str : "");
    env->ReleaseStringUTFChars(jsonStr, str);

    std::string errReason;
    bool valid = GbnfSampler::GetInstance().ValidateActionJson(input, errReason);
    if (valid) {
        return env->NewStringUTF("");
    } else {
        return env->NewStringUTF(errReason.c_str());
    }
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_compileActionJson(JNIEnv* env, jobject /* this */, jstring promptStr, jstring screenGraphStr) {
    const char* pStr = env->GetStringUTFChars(promptStr, nullptr);
    std::string prompt(pStr ? pStr : "");
    env->ReleaseStringUTFChars(promptStr, pStr);

    std::string screenGraph = "";
    if (screenGraphStr != nullptr) {
        const char* sgStr = env->GetStringUTFChars(screenGraphStr, nullptr);
        if (sgStr) {
            screenGraph = sgStr;
            env->ReleaseStringUTFChars(screenGraphStr, sgStr);
        }
    }

    std::string result = StreamingInferenceEngine::GetInstance().CompileActionJsonFromPrompt(prompt, screenGraph);
    return env->NewStringUTF(result.c_str());
}

} // extern "C"
