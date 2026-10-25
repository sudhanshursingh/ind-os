# ==============================================================================
# Ind OS — Native AI Inference CMake Build Specification
# Subsystem: ind-neural-core / Layer 3 Native AI Daemon
# Target: libind_ai.so (Snapdragon 732G / Kryo 470 / Adreno 618)
# ==============================================================================

cmake_minimum_required(VERSION 3.22.1)

project("ind_ai" C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Architectural compiler flags per PRD §2 (D2, D3) & §7 Sprint 3
if (${ANDROID_ABI} STREQUAL "arm64-v8a")
    add_compile_options(
        -march=armv8.2-a+dotprod+fp16
        -O3
        -funroll-loops
        -fPIC
        -Wall
        -Wextra
        -Wno-unused-parameter
    )
    message(STATUS "[Ind OS] Compiling for arm64-v8a with ARMv8.2-A DotProd + FP16 SIMD")
else()
    add_compile_options(
        -O3
        -fPIC
        -Wall
        -Wextra
        -Wno-unused-parameter
    )
    message(STATUS "[Ind OS] Compiling for host/generic ABI: ${ANDROID_ABI}")
endif()

# Vulkan 1.1 Compute Support for Adreno 618
option(GGML_VULKAN "Enable Vulkan 1.1 compute backend for Adreno 618" ON)
if (GGML_VULKAN)
    add_definitions(-DGGML_VULKAN=1)
    message(STATUS "[Ind OS] Vulkan Compute backend enabled (-DGGML_VULKAN=1)")
endif()

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Source files for libind_ai.so
set(IND_AI_SOURCES
    ind_jni_bridge.cpp
    NativeHardwareProbe.cpp
    MmapModelPager.cpp
    StreamingInferenceEngine.cpp
)

add_library(
    ind_ai
    SHARED
    ${IND_AI_SOURCES}
)

# Find and link required Android system libraries
find_library(log-lib log)
find_library(android-lib android)

set(LINK_LIBRARIES
    ${log-lib}
    ${android-lib}
)

if (GGML_VULKAN)
    find_library(vulkan-lib vulkan)
    if (vulkan-lib)
        list(APPEND LINK_LIBRARIES ${vulkan-lib})
        message(STATUS "[Ind OS] Linked Vulkan runtime library: ${vulkan-lib}")
    endif()
endif()

target_link_libraries(
    ind_ai
    ${LINK_LIBRARIES}
)
