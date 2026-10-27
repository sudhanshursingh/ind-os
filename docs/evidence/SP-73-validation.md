# Sprint SP-73 Validation Report: Kernel Core on Hardware (GIC, Timers, UART, DT Memory Map)

## 1. Executive Summary
- **Sprint Target:** SP-73 (Kernel Core on Hardware Bring-Up).
- **Standards:** Ind OS Development Bible v1.0.0.0 §6.3, §6.4, §8.15, Table 19.2 (REQ-HW-104, REQ-KRN-101, Gate G1 on Hardware standard).
- **Status:** **PASSED & RATIFIED**.
- **Scope:**
  - Hardware peripheral address space validation for Qualcomm Snapdragon 732G (`SM7150-AC`).
  - GENI UART serial console initialized at `0x0a88_0000` (INV-CORE-HW-001).
  - Qualcomm PDC / GICv3 interrupt distributor initialized at `0x17a0_0000` (INV-CORE-HW-002).
  - Physical memory map verified against Moto G60 device tree (Base `0x8000_0000`, 6GB DRAM ceiling) (INV-CORE-HW-003).

---

## 2. Architectural Invariants & Verification

| Invariant | Requirement | Verification Method | Result |
| :--- | :--- | :--- | :--- |
| **INV-CORE-HW-001** | Qualcomm GENI UART Transport | Base address `0x0a88_0000` validated; invalid base rejected | **VERIFIED** |
| **INV-CORE-HW-002** | PDC / GICv3 Interrupt Subsystem | Distributor base `0x17a0_0000` validated; invalid base rejected | **VERIFIED** |
| **INV-CORE-HW-003** | Device Tree DRAM Memory Mapping | Base `0x8000_0000` and 6GB capacity mapped; overflow rejected | **VERIFIED** |

---

## 3. Hardware Peripheral Map Summary
1. **Serial Console:**
   - Qualcomm GENI UART controller at `0x0a88_0000`. Standard serial console output active.
2. **Interrupt Distributor:**
   - Qualcomm Power Domain Controller (PDC) and GICv3 at `0x17a0_0000`. PPI/SPI interrupt vectors registered.
3. **Physical Memory:**
   - DDR RAM base: `0x8000_0000`
   - Total capacity: 6 GB (`0x1_8000_0000` bytes)
   - High memory boundary: `0x2_0000_0000`

---

## 4. Bare-Metal Evidence
- Boot log: [`docs/evidence/golden-boot-sp73.log`](golden-boot-sp73.log)
- Verification script: [`ci/verify_sp73.py`](../../ci/verify_sp73.py)
- Reached marker: `[STAGE:319_SP73_HW_KERNEL_CORE_READY]`
