# Ind OS Architectural Specification: Device Manager & Hotplug Events
**Document Identifier:** `INDOS-SPEC-DEVMGR-01`  
**Sprint:** SP-34  
**Phase:** Phase 5 (Userspace & the Developer OS)  
**Status:** **APPROVED / ACTIVE**  
**Requirements Traceability:** `REQ-USE-203`  
**Invariants Enforced:** `INV-DEV-001` (Zero Ambient Device Authority), `INV-DEV-002` (Hotplug Lifecycle & Atomic Revocation)  
**Author:** Sudhanshu Raj Singh (`sudhanshurajsingh@users.noreply.github.com`)

---

## 1. Scope & Objective

This specification defines the architectural design, event protocols, and capability delegation mechanisms for the **Ind OS Userspace Device Manager (`devmgr` / `indus-devmgr`)**.

Under the microkernel architecture of Ind OS (DEF-001, DEF-002), hardware devices must not be ambiently accessible to user processes or driver services. The Device Manager serves as the authoritative arbiter of hardware discovery, dynamic bus enumeration, and granular capability handouts (`REQ-USE-203`).

---

## 2. Architectural Invariants

### 2.1 Invariant `INV-DEV-001`: Zero Ambient Device Authority
- Drivers have zero direct access to memory-mapped I/O (MMIO), interrupt lines (IRQ), or direct memory access (DMA) windows by default.
- Every driver must receive an explicitly scoped `DeviceCapToken` minted by `devmgr`.
- A driver attempting to access hardware without a valid, unattenuated capability token receives an immediate permission fault (`DeviceError::AccessDenied`).

### 2.2 Invariant `INV-DEV-002`: Hotplug Lifecycle & Atomic Revocation
- When a device departs (unplug event), `devmgr` triggers an atomic revocation event.
- All capability tokens issued to the active driver for that device instance are invalidated immediately.
- Stale driver operations on departed hardware are blocked before reaching HAL or physical MMIO/IRQ registers.

---

## 3. Hotplug Event Model (`REQ-USE-203`)

### 3.1 Device Identifiers & Types
A device in Ind OS is uniquely represented by `DeviceId`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId {
    pub bus_type: BusType,     // VirtIO, Platform, PCIe
    pub bus_index: u32,        // e.g. slot index or MMIO index
    pub device_type: DeviceType, // Block, Net, Console, Input, Gpu
    pub instance_id: u32,      // Unique instance counter
}
```

### 3.2 Event Types
Hotplug notifications are broadcast across the `HotplugEventBus`:
- `DeviceArrived(DeviceId, HardwareDescriptor)`: Emitted when a new peripheral is detected on a bus.
- `DeviceDeparted(DeviceId)`: Emitted when a peripheral is disconnected or powered down.
- `DeviceReset(DeviceId)`: Emitted when a peripheral undergoes bus reset or recovery.
- `DeviceStateChanged(DeviceId, DeviceState)`: Emitted when driver state transitions occur.

### 3.3 Event Subscriber Bus
Driver daemons register subscriptions for specific device types. The `HotplugEventBus` utilizes bounded, panic-free collections from `libindus` (`StaticVec`, `RingBuffer`) to guarantee predictable real-time dispatch without memory exhaustion.

---

## 4. Capability Handout Protocol (`REQ-USE-203`)

When a device arrives and matches an authorized driver manifest:
1. **MMIO Capability (`MmioCapToken`):** Scoped to the precise physical base address and region length allocated to that device (e.g. `0x0a000000..0x0a000200`).
2. **IRQ Capability (`IrqCapToken`):** Scoped to the designated hardware interrupt vector.
3. **DMA Window Capability (`DmaCapToken`):** Scoped to permitted physical memory buffer bounds for bus-mastering transactions.
4. **Device Authority Token (`DeviceCapToken`):** Bundles sub-tokens and authorizes control actions (`Reset`, `Configure`, `Read`, `Write`).

---

## 5. VirtIO Dynamic Plug/Unplug Test Sequence

The test cycle verifies:
1. `VirtIO-Blk` dynamic arrival: device recognized, driver matched, capabilities minted and delegated.
2. Driver reads/writes using scoped capabilities.
3. `VirtIO-Blk` unplug event: device departure broadcast, capabilities revoked.
4. Subsequent driver access with revoked token is trapped and denied with `DeviceError::DeviceDeparted`.
