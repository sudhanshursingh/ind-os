# IndOS Reference Drivers (User Mode) Architecture Specification
**Document ID:** `INDOS-SPEC-REFDRV-01`  
**Sprint:** SP-27 (Phase 4)  
**Governing Standard:** Ind OS Development Bible v1.0.0.0 (§8.10), Rules N-01, N-04, N-10  
**Status:** RATIFIED  
**Requirements Satisfied:** `REQ-KRN-107` (User-Mode Driver Framework over IPC)  
**Target Invariants:** `INV-DRV-001`, `INV-DRV-002`, `INV-DRV-003`  

---

## 1. Architectural Motivation

Sprint SP-26 established the User-Mode Driver Framework (Device Manager, IRQ Broker, DMA/SMMU Broker, and Driver Protocol over IPC). Sprint SP-27 delivers concrete **reference user-mode drivers** that exercise and prove this framework across four distinct device categories:

1. **Storage (`virtio-blk`):** Serves block storage requests (`IoRead`, `IoWrite`) over IPC using capability-governed DMA windows.
2. **Networking (`virtio-net`):** Manages packet transmission and reception, MAC address filtering, and asynchronous IRQ notification processing.
3. **Display (`framebuffer`):** Configures display mode (resolution, pixel format) and manages zero-privilege double-buffered pixel rendering.
4. **Input (`keyboard/mouse`):** Ingests keyboard keypresses and mouse movement events, buffers them in a bounded event queue, and dispatches sanitized events to client services (such as the Consent Broker and Window Manager).

All reference drivers run strictly in user mode (EL0) without supervisor privileges, relying exclusively on typed `Capability` tokens (`CapType::Device`, `CapType::Interrupt`, `CapType::DmaWindow`).

---

## 2. Driver Architectures

```
+-------------------------------------------------------------------------+
|                       User-Mode Reference Drivers                       |
|                                                                         |
|  +----------------+  +----------------+  +--------------+  +----------+ |
|  | VirtioBlkDriver|  | VirtioNetDriver|  | Framebuffer  |  |  Input   | |
|  | - Sector I/O   |  | - Tx/Rx Rings  |  | - 1280x720   |  | - Keyb   | |
|  | - DMA Windows  |  | - MAC Config   |  | - Dirty Rect |  | - Mouse  | |
|  +--------+-------+  +--------+-------+  +------+-------+  +----+-----+ |
+-----------|-------------------|-----------------|---------------|-------+
            |                   |                 |               |
            +-------------------+-----------------+---------------+
                                |
                  DriverProtocol over IPC (REQ-KRN-107)
                                |
+-------------------------------v-----------------------------------------+
|                  IndK Microkernel Core Subsystems                       |
|                                                                         |
|   +-------------------+  +-------------------+  +-------------------+   |
|   |   DeviceManager   |  |     IrqBroker     |  |     DmaBroker     |   |
|   |   (INV-DRV-001)   |  |   (INV-DRV-002)   |  |   (INV-DRV-003)   |   |
|   +-------------------+  +-------------------+  +-------------------+   |
+-------------------------------------------------------------------------+
```

### 2.1 VirtIO Block Driver (`VirtioBlkDriver`)
- **Managed Device:** VirtIO Block Device (`vendor_id = 0x1AF4`, `device_type = DeviceType::Block`).
- **Protocol Operations:** Responds to `DriverOp::IoRead` and `DriverOp::IoWrite` messages.
- **DMA Integration:** Receives client requests with LBA and sector count, validates DMA buffer boundaries via `DmaBroker::validate_transfer`, and coordinates transfer without direct physical memory pointers.

### 2.2 VirtIO Net Driver (`VirtioNetDriver`)
- **Managed Device:** VirtIO Network Device (`device_type = DeviceType::Net`).
- **Network Interface:** 6-byte MAC address, MTU 1500 bytes, link status tracking.
- **Interrupt Handling:** Subscribes to net IRQ via `IrqBroker`. Upon receiving `DriverOp::IrqNotify`, drains the RX packet buffer, passes packets to the client endpoint, and emits `DriverOp::IrqAck` to unmask the line.

### 2.3 Framebuffer Driver (`FramebufferDriver`)
- **Managed Device:** Display Controller (`device_type = DeviceType::Display`).
- **Display Modes:** Supports 32-bit ARGB8888 packed pixel formats (e.g. 1024x768 or 1280x720).
- **Safe Rendering:** Maintains bounded dirty rectangles (`x`, `y`, `w`, `h`) to minimize rasterization overhead, writing pixel data to pre-allocated user memory.

### 2.4 Keyboard & Mouse Input Driver (`InputDriver`)
- **Managed Device:** Input Peripheral (`device_type = DeviceType::Input`).
- **Event Model:**
  - `KeyEvent`: Key code, press/release state.
  - `MouseEvent`: Relative motion (`dx`, `dy`), button mask (`left`, `right`, `middle`).
- **Ring Buffer:** Bounded circular event queue with overflow protection, ensuring malicious or high-rate input cannot exhaust kernel resources.

---

## 3. Formal Invariants Matrix

| Invariant | Target Subsystem | Statement | Verification |
| :--- | :--- | :--- | :--- |
| **`INV-DRV-001`** | Reference Drivers | All four reference drivers execute in unprivileged user processes without kernel supervisor mode. | Capability validation on all driver operations |
| **`INV-DRV-002`** | Net & Input IRQ | Interrupt-driven reference drivers process IRQ notifications and issue authenticated `IRQ_ACK`. | Headless QEMU IRQ dispatch & ACK |
| **`INV-DRV-003`** | Block & Net DMA | Block and network drivers execute DMA strictly within verified SMMU/IOMMU window boundaries. | DMA bounds checking and validation |

---

## 4. Definition of Done (DoD)

1. Pure safe Rust implementation in `kernel/src/driver_framework/reference/` with 0 `unsafe` blocks.
2. Complete implementations of `VirtioBlkDriver`, `VirtioNetDriver`, `FramebufferDriver`, and `InputDriver`.
3. Headless QEMU execution covering Stages 78–83 in `kernel/src/main.rs`.
4. Verification harness `ci/verify_sp27.py` passing 6/6 checks.
5. Golden boot log and validation report recorded.
