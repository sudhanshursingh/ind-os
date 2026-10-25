// ==============================================================================
// Ind OS — JNI Bridge for Native AI Subsystem (libind_ai.so)
// Subsystem: ind-neural-core / Layer 3 Native AI Daemon
// Reference Silicon: Snapdragon 732G (SM7150-AC)
// ==============================================================================

#include "include/ind_native.h"

#include <jni.h>
#include <android/log.h>
#include <string>

using namespace indos::native;

extern "C" {

JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM* vm, void* reserved) {
    __android_log_print(
        ANDROID_LOG_INFO,
        IND_NATIVE_TAG,
        "[IND_NATIVE] libind_ai.so loaded successfully via JNI! Version: 1.0.0-sprint3. Reference Silicon: Snapdragon 732G (SM7150-AC / Kryo 470 / Adreno 618)."
    );
    return JNI_VERSION_1_6;
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_runHardwareProbe(JNIEnv* env, jobject /* this */) {
    __android_log_print(
        ANDROID_LOG_INFO,
        IND_NATIVE_TAG,
        "[IND_NATIVE] Java_indos_neural_NativeBridge_runHardwareProbe() invoked."
    );
    std::string report = HardwareProbe::GenerateHardwareReportJson();
    return env->NewStringUTF(report.c_str());
}

JNIEXPORT jdouble JNICALL
Java_indos_neural_NativeBridge_runCpuGemmBenchmark(JNIEnv* env, jobject /* this */, jint threads, jint dim) {
    __android_log_print(
        ANDROID_LOG_INFO,
        IND_NATIVE_TAG,
        "[IND_NATIVE] Java_indos_neural_NativeBridge_runCpuGemmBenchmark(threads=%d, dim=%d) invoked.",
        threads,
        dim
    );
    BenchmarkMetrics metrics = HardwareProbe::RunCpuGemmBenchmark(threads, dim);
    return static_cast<jdouble>(metrics.gflops);
}

JNIEXPORT jdouble JNICALL
Java_indos_neural_NativeBridge_runVulkanComputeBenchmark(JNIEnv* env, jobject /* this */, jint dim) {
    __android_log_print(
        ANDROID_LOG_INFO,
        IND_NATIVE_TAG,
        "[IND_NATIVE] Java_indos_neural_NativeBridge_runVulkanComputeBenchmark(dim=%d) invoked.",
        dim
    );
    BenchmarkMetrics metrics = HardwareProbe::RunVulkanComputeBenchmark(dim);
    return static_cast<jdouble>(metrics.gflops);
}

JNIEXPORT jboolean JNICALL
Java_indos_neural_NativeBridge_evictMmapRange(JNIEnv* env, jobject /* this */, jlong addr, jlong length) {
    void* ptr = reinterpret_cast<void*>(addr);
    size_t len = static_cast<size_t>(length);
    __android_log_print(
        ANDROID_LOG_INFO,
        IND_NATIVE_TAG,
        "[IND_NATIVE] Java_indos_neural_NativeBridge_evictMmapRange(addr=%p, len=%zu) invoked.",
        ptr,
        len
    );
    bool success = HardwareProbe::EvictMmapRange(ptr, len);
    return success ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jstring JNICALL
Java_indos_neural_NativeBridge_getNativeStatus(JNIEnv* env, jobject /* this */) {
    std::string status = "libind_ai.so: ACTIVE (ARMv8.2-A DotProd + FP16 SIMD + Adreno 618 Vulkan)";
    return env->NewStringUTF(status.c_str());
}

} // extern "C"
