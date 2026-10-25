// ==============================================================================
// Ind OS — Zero-Copy mmap Model Pager Header
// Subsystem: ind-neural-core / Layer 3 Native AI Daemon
// Reference Silicon: Snapdragon 732G (SM7150-AC / 6 GB LPDDR4X)
// BIBLE Reference: Invariant 1 (6144 MB Partition & <15 ms madvise eviction)
// ==============================================================================

#ifndef IND_MMAP_MODEL_PAGER_H
#define IND_MMAP_MODEL_PAGER_H

#include <string>
#include <cstddef>
#include <cstdint>

#define IND_BENCHMARK_TAG "IND_BENCHMARK"

namespace indos {
namespace native {

enum class MmapState {
    UNLOADED = 0,
    MAPPED_RESIDENT = 1,
    EVICTED = 2
};

struct EvictionResult {
    bool success = false;
    double duration_ms = 0.0;
    double rss_before_mb = 0.0;
    double rss_after_mb = 0.0;
    double freed_mb = 0.0;
};

class MmapModelPager {
public:
    static MmapModelPager& GetInstance();

    // Zero-copy POSIX mmap(MAP_SHARED) with MADV_SEQUENTIAL prefetch
    bool LoadModel(const std::string& modelPath, size_t simulatedFallbackSize = 986000000ULL);

    // Evicts model weights from physical RAM in < 15 ms via madvise(MADV_DONTNEED)
    EvictionResult EvictWeights();

    // Re-activates mmap pages upon subsequent inference call
    bool PrefetchWeights();

    // Unmaps memory and closes file descriptors
    void UnloadModel();

    // Telemetry and OS memory inspection
    double GetResidentRssMb() const;
    double GetMappedSizeMb() const;
    MmapState GetState() const;
    std::string GetModelPath() const;
    std::string GetTelemetryJson() const;

    void* GetMappedAddress() const { return mapped_addr_; }
    size_t GetMappedSize() const { return mapped_size_; }

private:
    MmapModelPager();
    ~MmapModelPager();

    MmapModelPager(const MmapModelPager&) = delete;
    MmapModelPager& operator=(const MmapModelPager&) = delete;

    void* mapped_addr_ = nullptr;
    size_t mapped_size_ = 0;
    int file_fd_ = -1;
    bool is_anonymous_ = false;
    std::string model_path_;
    MmapState state_ = MmapState::UNLOADED;

    static double ReadProcRssMb();
};

} // namespace native
} // namespace indos

#endif // IND_MMAP_MODEL_PAGER_H
