# Ind OS Kernel Core on Hardware Specification (SP-73)

## 1. Architectural Mandate
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.15, Table 19.2 (REQ-HW-104, REQ-KRN-101, Gate G1 on Hardware standard).
- **Target:** Qualcomm Snapdragon 732G (`SM7150-AC`) core hardware peripheral initialization: GENI UART serial console, PDC/GICv3 interrupt controller, ARM generic timers, and Device Tree DRAM memory map.

---

## 2. Architectural Invariants

| ID | Description | Enforcement Mechanism |
| :--- | :--- | :--- |
| **INV-CORE-HW-001** | GENI UART Hardware Transport | UART serial console initialized at Qualcomm GENI base `0x0a88_0000` with 115200 8N1 configuration |
| **INV-CORE-HW-002** | PDC / GICv3 Interrupt Subsystem | Qualcomm PDC / GICv3 initialized at base `0x17a0_0000`; PPI/SPI vector routing armed |
| **INV-CORE-HW-003** | Device Tree DRAM Memory Mapping | Hardware memory map matches Moto G60 device tree (Base `0x8000_0000`, 6GB DRAM ceiling `0x2_0000_0000`) |

---

## 3. Data Structures & API Contracts

```rust
pub const HANOIP_GENI_UART_BASE: u64 = 0x0a88_0000;
pub const HANOIP_PDC_GIC_BASE: u64 = 0x17a0_0000;
pub const HANOIP_DRAM_BASE: u64 = 0x8000_0000;
pub const HANOIP_DRAM_SIZE_BYTES: u64 = 6 * 1024 * 1024 * 1024; // 6GB

pub enum HardwareCoreError {
    InvalidBaseAddress,
    MemoryRangeOverflow,
    InterruptRoutingFailed,
}

pub struct HardwareCoreHarness {
    uart_initialized: bool,
    gic_initialized: bool,
    dram_mapped: bool,
}
```
