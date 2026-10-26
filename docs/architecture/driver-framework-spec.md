# IndOS User-Mode Driver Framework Architecture Specification
**Document ID:** `INDOS-SPEC-DRV-01`  
**Sprint:** SP-26 (Phase 4)  
**Governing Standard:** Ind OS Development Bible v1.0.0.0 (§8.10), Rules N-01, N-04, N-10  
**Status:** RATIFIED  
**Requirements Satisfied:** `REQ-KRN-107` (User-Mode Driver Framework over IPC)  
**Target Invariants:** `INV-DRV-001`, `INV-DRV-002`, `INV-DRV-003`  

---

## 1. Architectural Motivation & Threat Model

In conventional monolithic kernels, device drivers execute in supervisor mode (EL1/Ring 0), where a single null pointer dereference, buffer overrun, or malicious peripheral can compromise the entire operating system. Per Bible §8.10 and `REQ-KRN-107`, Ind OS enforces a strict **microkernel user-mode driver framework**:

1. **Zero Kernel-Mode Driver Privilege (`INV-DRV-001`):** Drivers run in isolated unprivileged user-mode processes (EL0). Drivers possess zero ambient authority. All hardware access (MMIO regions, interrupt subscription, DMA window allocation) strictly requires possessing an unforgeable, typed `Capability` token.
2. **Interrupt Brokerage via IPC (`INV-DRV-002`):** Hardware interrupts are trapped by the EL1 microkernel interrupt dispatcher, masked at the GIC distributor, and translated into non-blocking asynchronous notifications or IPC messages delivered to the driver's registered endpoint. The interrupt line remains safely masked until the driver process verifies hardware state and submits an authenticated `IRQ_ACK` message.
3. **SMMU / IOMMU DMA Isolation (`INV-DRV-003`):** User-mode drivers cannot program arbitrary physical memory addresses into bus-mastering hardware. All DMA operations are restricted to bounded, kernel-audited DMA windows mapped through the system IOMMU (ARM SMMUv2/v3 on QEMU virt) and tracked via `CapType::DmaWindow`.

---

## 2. Core Subsystems

```
+-------------------------------------------------------------+
|               User-Mode Driver (EL0 Process)                |
|  - Event loop waiting on IPC Endpoint                       |
|  - Handles IRQ notifications & processes device I/O         |
|  - Communicates via DriverProtocol over IPC (REQ-KRN-107)   |
+------------------------------+------------------------------+
                               |
                   Capability-Guarded IPC
                               |
+------------------------------v------------------------------+
|             IndK Microkernel Core (EL1 Microkernel)         |
|                                                             |
|  +--------------------+  +-------------------------------+  |
|  |   Device Manager   |  |          IRQ Broker           |  |
|  | - Device Registry  |  | - IRQ -> IPC Notification     |  |
|  | - Cap Authorization|  | - GIC Mask/Unmask on IRQ_ACK  |  |
|  | - Lifecycle State  |  | - CapType::Interrupt Guard    |  |
|  +--------------------+  +-------------------------------+  |
|                                                             |
|  +--------------------+  +-------------------------------+  |
|  |     DMA Broker     |  |         Driver Protocol       |  |
|  | - SMMU/IOMMU Window|  | - Register / Bind / Status    |  |
|  | - Bounded Buffers  |  | - Read / Write / IoControl    |  |
|  | - CapType::DmaWind |  | - Fixed-size IpcMessage Regs  |  |
|  +--------------------+  +-------------------------------+  |
+-------------------------------------------------------------+
```

### 2.1 Device Manager (`DeviceManager`)
The Device Manager maintains a bounded registry of all discovered hardware peripherals and virtual devices (e.g. VirtIO block, net, console, input).
- **Device Lifecycle:** `Unregistered -> Registered -> Bound -> Active -> Faulted`.
- **Capability Guard:** Binding to a device requires presenting a valid `CapType::Device(device_id)` capability token with `CapRights::WRITE` (Bind) and `CapRights::ADMIN`.
- **Isolation:** An unprivileged process attempting to bind to a device without a valid capability is deterministically rejected with `EPERM`.

### 2.2 IRQ Broker (`IrqBroker`)
- **Subscription:** A driver process subscribes to an interrupt vector by submitting its IPC endpoint ID and presenting a valid `CapType::Interrupt(irq_num)` capability.
- **Dispatch:** When a hardware interrupt fires at EL1, the kernel interrupt handler invokes `IrqBroker::dispatch_irq(irq)`. The broker masks the GIC interrupt line, sets the subscription state to `Pending`, and delivers an asynchronous notification to the driver's endpoint.
- **Acknowledgment:** Upon servicing the device in userspace, the driver sends an `IRQ_ACK` IPC message. The kernel verifies the driver's PID and capability, clears the pending state, and unmasks the interrupt at the GIC.

### 2.3 DMA Broker & IOMMU Abstraction (`DmaBroker`)
- **Window Allocation:** Drivers request DMA windows by specifying buffer size, transfer direction (`HostToDevice`, `DeviceToHost`, `Bidirectional`), and alignment.
- **SMMU Programming:** The kernel configures IOMMU translation table entries for the driver's process address space and returns a `CapType::DmaWindow(window_id)` token.
- **Bounds Checking:** Out-of-bounds DMA access or transfers exceeding granted window bounds are rejected with `EINVAL` / `EPERM`.

### 2.4 Driver Protocol over IPC (`DriverProtocol`)
All user-mode driver interactions are multiplexed over standard Ind OS register-based `IpcMessage` structures (Bible §8.8):
- `MSG_REGISTER = 0x01`: Driver registers its service endpoint with the Device Manager.
- `MSG_BIND     = 0x02`: Driver binds to a target hardware device using its capability token.
- `MSG_IRQ_NOTIFY = 0x03`: Kernel notifies driver of a pending hardware interrupt.
- `MSG_IRQ_ACK  = 0x04`: Driver signals interrupt handling completion to unmask the line.
- `MSG_DMA_MAP  = 0x05`: Driver requests a DMA window mapping.
- `MSG_DMA_UNMAP= 0x06`: Driver releases a DMA window.
- `MSG_IO_READ  = 0x07`: Client sends read request to driver.
- `MSG_IO_WRITE = 0x08`: Client sends write request to driver.
- `MSG_STATUS   = 0x09`: Health and diagnostic query.

---

## 3. Formal Invariants Matrix

| Invariant ID | Target Subsystem | Statement | Violation Outcome |
| :--- | :--- | :--- | :--- |
| **`INV-DRV-001`** | Device Manager | Zero Kernel-Mode Driver Privilege: Device binding and MMIO access require explicit capability tokens. Unprivileged access returns `EPERM`. | Kernel rejects binding with `EPERM`; driver cannot touch hardware. |
| **`INV-DRV-002`** | IRQ Broker | IRQ Broker Isolation: Hardware IRQ lines are masked upon firing and only unmasked upon receiving authenticated `IRQ_ACK` with valid `CapType::Interrupt`. | Unauthenticated unmask rejected; rogue interrupt storms prevented. |
| **`INV-DRV-003`** | DMA Broker | DMA/IOMMU Isolation: Direct Memory Access is strictly bounded to kernel-audited DMA windows. Arbitrary physical addressing is blocked. | Invalid DMA attempts return `EINVAL`/`EPERM`; host RAM preserved. |

---

## 4. Definition of Done (DoD)

1. Pure safe Rust implementation in `kernel/src/driver_framework/` with 0 `unsafe` blocks (Rule N-04).
2. Zero external dependencies (Rule N-01).
3. Capability table integration: `CapType::Device`, `CapType::Interrupt`, `CapType::DmaWindow`.
4. Headless QEMU verification covering Stages 72–77 in `kernel/src/main.rs`.
5. CI verification harness `ci/verify_sp26.py` integrated into `make check`.
6. Golden boot log and validation report recorded.
