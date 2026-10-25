// ==============================================================================
// Ind OS — Native Hardware Probe & Engine Header
// Target: Motorola Moto G60 (hanoip) / Snapdragon 732G (SM7150-AC)
// ==============================================================================

#ifndef IND_NATIVE_H
#define IND_NATIVE_H

#include <string>
#include <vector>
#include <cstdint>

#define IND_NATIVE_TAG "IND_NATIVE"

#include "MmapModelPager.h"
#include "StreamingInferenceEngine.h"
#include "GbnfSampler.h"

namespace indos {
namespace native {

struct HardwareCapabilities {
    bool has_armv8_2_dotprod = false;
    bool has_fp16 = false;
    bool has_neon = false;
    int cpu_core_count = 8;
    int optimal_kryo_threads = 4; // 2x Cortex-A76 @ 2.3 GHz + 2x Cortex-A55
    bool has_vulkan_compute = false;
    std::string gpu_name = "Adreno (TM) 618";
    uint32_t vulkan_api_version = 0;
};

struct BenchmarkMetrics {
    int thread_count = 4;
    int matrix_dim_m = 512;
    int matrix_dim_n = 512;
    int matrix_dim_k = 512;
    double duration_ms = 0.0;
    double gflops = 0.0;
    double tok_per_sec_estimate = 0.0;
};

class HardwareProbe {
public:
    static HardwareCapabilities DetectCapabilities();
    static BenchmarkMetrics RunCpuGemmBenchmark(int threads = 4, int dim = 512);
    static BenchmarkMetrics RunVulkanComputeBenchmark(int dim = 512);
    static std::string GenerateHardwareReportJson();
    static bool EvictMmapRange(void* addr, size_t length);
};

} // namespace native
} // namespace indos

#endif // IND_NATIVE_H
