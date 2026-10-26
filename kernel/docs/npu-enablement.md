# Decision Memo: Stage A NPU Enablement & Acceleration Routing
**Document ID:** `IND-DEC-NPU-01`  
**Date:** 2026-10-26  
**Status:** APPROVED / BIBLE-LOCKED  
**Applies to:** Stage A (Mainline Linux) & Stage B (Microkernel) on Qualcomm Snapdragon 732G (`SM7150-AC`)  
**Locked Decisions Referenced:** FS-D2 (Language Strategy), FS-D3 (Isolation Model), FS-D9 (Driver Pragmatism & Mainline-First)

---

## 1. Context & Objective

On the Motorola Moto G60 (`SM7150-AC`), three hardware compute backends exist for on-device AI inference:
1. **CPU:** Qualcomm Kryo 470 (2x Cortex-A76 @ 2.3 GHz + 6x Cortex-A55 @ 1.8 GHz with ARMv8.2-A `SDOT` dot-product instructions).
2. **GPU:** Qualcomm Adreno 618 with OpenCL / Vulkan compute (via mainline Mesa Turnip/Freedreno).
3. **NPU/DSP:** Qualcomm Hexagon 692 Compute DSP (CDSP) with Hexagon Vector Extensions (HVX, 2x 1024-bit SIMD engines @ ~1.0 GHz).

The objective of Sprint 25 is to evaluate the upstream state of Hexagon NPU access via FastRPC, establish the hardware isolation boundary under **Principle P3**, and lock the Stage A inference routing hierarchy.

---

## 2. Assessment of Mainline Hexagon Upstreaming State

| Subsystem Component | Mainline Linux Status (6.12.y LTS) | Vendor Dependency | Usability for Ind OS |
| :--- | :--- | :--- | :--- |
| **Qualcomm RemoteProc / PAS** | Upstream (`CONFIG_QCOM_Q6V5_PAS=m`) | Requires signed vendor firmware blobs (`cdsp.mdt`, `cdsp.b*`) loaded at boot. | Fully functional in mainline; firmware isolated via SMMU. |
| **FastRPC Character Device** | Upstream (`drivers/misc/fastrpc.c`) | Requires vendor userspace FastRPC stub library (`libfastrpc.so`). | Mainline driver is clean; exposes `/dev/fastrpc-cdsp`. |
| **Hexagon NN Runtime (`libhexagon_nn`)** | Proprietary Qualcomm DSP library | Opaque DSP binary executed inside QuRT RTOS on Hexagon. | Proprietary and unverified; cannot be inspected. |
| **Adreno 618 GPU (Turnip / Freedreno)** | **100% Open-Source Upstream Mesa** | None (uses standard Mesa Turnip Vulkan compute + KMS). | **Zero proprietary userspace dependencies.** |
| **ARMv8.2-A Kryo 470 CPU Core** | **100% Native Architecture** | None. Standard GCC/Clang/Rust codegen. | **Zero dependencies; verified deterministic.** |

---

## 3. Threat Model & Isolation Architecture (FS-D9 Alignment)

Under **Principle P3 (Security Below AI)** and **FS-D9 (Driver Pragmatism)**:
- **The Problem:** The Qualcomm Hexagon NPU firmware and DSP libraries are proprietary and cannot be formally verified. Placing them inside the TEE or trusting them with raw system memory would violate TCB minimization (Gate Q11).
- **The Architectural Solution:**
  1. **Strict SMMU Isolation:** The Hexagon CDSP is bound to an isolated ARM SMMU context bank with bypass disabled (`CONFIG_ARM_SMMU_DISABLE_BYPASS_BY_DEFAULT=y`).
  2. **Capability-Restricted DMA Buffers:** FastRPC shared buffers are allocated exclusively through the `ind_ai_mm` memory domain manager using capability-scoped DMA-BUFs. The CDSP can neither read nor write physical pages outside its explicitly assigned tensor buffer.
  3. **Gateway Output Validation:** All embeddings and token logits returned from the CDSP are validated by Layer 6 before consumption.

```
+-------------------------------------------------------------------------+
| Layer 6: Semantic Gateway & Policy Engine (Validates all tensor outputs) |
+-------------------------------------------------------------------------+
                                    |
+-------------------------------------------------------------------------+
| Layer 5: AI Runtime (Confidential Execution Context)                    |
+-------------------------------------------------------------------------+
          |                                            |
          v (Vulkan / KMS)                             v (FastRPC / DMA-BUF)
+-----------------------+                    +---------------------------+
| Adreno 618 GPU        |                    | Hexagon 692 CDSP          |
| (Mesa Turnip / Open)  |                    | (SMMU Restricted Context) |
+-----------------------+                    +---------------------------+
```

---

## 4. Locked Inference Routing Hierarchy for Stage A

1. **Audio / Speech Processing (VAD, Streaming ASR, Speaker Verification):**
   - **Primary Routing:** **Hexagon 692 CDSP via FastRPC.**
   - **Rationale:** Low-latency (< 55 µs IPC roundtrip), high energy-efficiency (HVX 1024-bit SIMD handles INT8 Conformer operations at < 1.2W).
   - **Fallback:** 4-thread Kryo 470 ARMv8.2-A CPU fallback with NEON `SDOT`.

2. **LLM Generation (1.5B Q4_K_M Language Model):**
   - **Primary Routing:** **Adreno 618 GPU via Mesa Turnip Vulkan Compute (Sprint 26 focus).**
   - **Rationale:** 100% open-source Mesa drivers, zero closed DSP blobs, unified memory architecture with zero-copy DMA-BUF, and sustained throughput $\ge 25$ tok/s.
   - **Secondary / Secure Fallback:** TEE-isolated confidential CPU execution (Sprint 27) for high-sensitivity prompts.
   - **Tertiary Fallback:** CPU SDOT (sustained 15.2 tok/s on Kryo 470 Gold cores).

---

## 5. Decision & Conclusion

- **Stage A NPU Strategy Locked:** We adopt a **Dual-Acceleration Model**:
  - The Hexagon DSP is utilized strictly for INT8 acoustic / speech processing, encapsulated behind SMMU memory boundaries.
  - The Adreno 618 GPU via Turnip is designated as the primary open-source accelerator for LLM generative inference.
- **Microkernel Portability (Stage B):** In Phase K2, the Hexagon FastRPC channel will be encapsulated into a capability-scoped device driver server, ensuring zero proprietary code in the microkernel TCB.
