# Ind OS Architecture Specification: Storage & IndFS Page Cache Optimization (SP-112)

**Sprint:** SP-112  
**Phase:** Phase 9.5 (Performance & Usability Pass)  
**Constitutional References:**
- `Ind_OS_Development_Bible_v1.0.0.0.md` §8.16, Table 19.2; `REQ-STR-101`, `REQ-PERF-104`
- Rule N-01: Zero external crates in TCB
- Rule N-04: Zero unsafe code in new userspace modules

---

## 1. Executive Summary

Sprint SP-112 eliminates block storage hotspots identified in SP-110:
1. **IndFS Direct Page Cache:** Introduces zero-copy slab buffer caching for read/write hot blocks.
2. **Batch Doorbell Submission:** UFSHCI command ring queuing batched up to 16 requests per doorbell write, yielding sustained throughput $> 28.5\text{ MB/s}$ read and $> 18.2\text{ MB/s}$ write.

---

## 2. Architectural Invariants

| Invariant ID | Title | Description & Guarantee |
|---|---|---|
| **INV-STORE-OPT-001** | Sustained Read Throughput $\ge 25\text{ MB/s}$ | Sequential 4KB block reads sustained across 10MB test set. |
| **INV-STORE-OPT-002** | Zero-Copy Page Cache Hit $\le 2.5\ \mu\text{s}$ | Cache hits served without intermediate heap allocations. |
| **INV-STORE-OPT-003** | Power-Loss Journal Invariant Maintained | All optimizations preserve crash consistency and CRC32 validity. |

---

## 3. Data Structures & API Contracts

```rust
pub const CAP_STORAGE_OPT: u64 = 0x0000_0000_0000_0200;
pub const STORAGE_PAGE_CACHE_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedBlock {
    pub lba: u64,
    pub dirty: bool,
    pub crc32: u32,
    pub access_count: u32,
}
```
