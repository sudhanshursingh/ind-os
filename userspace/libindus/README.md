# libindus (`indus-core`) — Ind OS Core Userspace Foundation Library

`libindus` is the standard `#![no_std]` foundation library for Ind OS userspace applications, system services, and device drivers.

## Features (`REQ-USE-202`)

1. **Deterministic Bounded Memory Allocator (`alloc`):**
   - Bump and block memory allocation over safe pre-allocated memory pools.
   - Out-of-memory guarantees without panic.
2. **Panic-Free Bounded Collections (`collections`):**
   - `StaticVec<T, N>`: Pre-allocated vector.
   - `RingBuffer<T, N>`: Circular FIFO queue.
   - `StaticString<N>`: Fixed-size UTF-8 string wrapper.
   - `StaticMap<K, V, N>`: Key-value map.
3. **Capability-First Ergonomic IPC Bindings (`ipc`):**
   - `CapabilityToken`: Strongly typed capability wrapper enforcing rights check.
   - `IpcClient`: Client communication handle over microkernel endpoints.
   - `IpcChannel`: Bidirectional request/response channel.

---

## Usage Example

```rust
use indus_core::alloc::UserHeapAllocator;
use indus_core::collections::StaticVec;
use indus_core::ipc::{CapabilityToken, IpcClient, IpcMessage, CAP_RIGHTS_READ, CAP_RIGHTS_WRITE};

fn main() {
    // 1. Initialize Bounded User Heap
    let mut heap = UserHeapAllocator::new();
    let offset = heap.allocate(1024, 8).expect("Allocation failed");

    // 2. Use Static Collections
    let mut vec = StaticVec::<u32, 16>::new();
    vec.push(42).unwrap();
    assert_eq!(vec.pop(), Some(42));

    // 3. Ergonomic Capability-First IPC
    let cap = CapabilityToken::new(101, CAP_RIGHTS_READ | CAP_RIGHTS_WRITE);
    let client = IpcClient::new(50, cap);

    let req = IpcMessage::new(1, 10).with_payload(b"PING").unwrap();
    let mut resp = IpcMessage::new(0, 0);
    client.call(&req, &mut resp).expect("IPC call failed");
}
```
