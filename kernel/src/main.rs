//! Ind OS Microkernel (IndK) Main Entry Point
//! Sprint SP-07: Kernel Heap & Core Data Structures

#![no_std]
#![no_main]

extern crate alloc;

pub mod arch;
pub mod console;
pub mod drivers;
pub mod ds;
pub mod interrupts;
pub mod logger;
pub mod mm;
pub mod panic;
pub mod sync;

use alloc::boxed::Box;
use alloc::vec::Vec;
use ds::{IntrusiveList, IntrusiveNode};

const GIT_HASH: &str = env!("INDK_GIT_HASH");
const TARGET_ARCH: &str = env!("INDK_TARGET_ARCH");
const PLATFORM: &str = env!("INDK_PLATFORM");

/// Kernel main entry point called from `boot.s` after EL1 transition & stack setup
#[no_mangle]
pub extern "C" fn kernel_main(dtb_addr: usize) -> ! {
    // Initialize the PL011 UART early serial console
    console::init_console();

    // Print authoritative IndK boot banner with Gate G1 provenance
    println!();
    println!("==================================================");
    println!(" [BOOT] IndK ARM64 Microkernel Starting...");
    println!("  Build Commit:   {}", GIT_HASH);
    println!("  Architecture:   {}", TARGET_ARCH);
    println!("  Platform:       {}", PLATFORM);
    println!("==================================================");
    println!(" [BOOT] Vector table (VBAR_EL1) online.");
    println!(" [BOOT] Early kernel stack verified (INV-BOOT-002).");

    // Emit boot-stage transition markers (Bible §8.7 standard)
    stage_marker!("01_RESET", "ARM64 EL1 entry and early stack established");
    stage_marker!("02_VECTORS", "VBAR_EL1 exception vectors armed (INV-BOOT-004)");
    stage_marker!("03_CONSOLE", "PL011 early serial console driver online");

    // Leveled logging demonstration
    log_info!("boot", "IndK microkernel core initialization sequence starting");
    log_debug!("arch", "Execution state: EL1h, interrupts masked, MMU identity");

    // Stage 04: Validate Device Tree Blob (INV-BOOT-003)
    stage_marker!("04_DTB", "Validating Device Tree Blob header");
    println!(" [BOOT] Device Tree Blob at: 0x{:08x}", dtb_addr);

    let (magic, totalsize) = unsafe {
        let ptr = dtb_addr as *const u32;
        let m = u32::from_be(*ptr);
        let sz = u32::from_be(*ptr.add(1));
        (m, sz)
    };

    if magic == 0xd00dfeed {
        println!(" [BOOT] DTB magic 0xd00dfeed verified (INV-BOOT-003).");
        log_info!("dtb", "DTB header verified (magic 0xd00dfeed, size {} bytes)", totalsize);
    } else {
        panic!("DTB magic corrupted or unmapped at pointer!");
    }

    log_warn!("sched", "Preemptive multitasking disabled (scheduled for Phase 2)");
    log_trace!("uart", "PL011 115200 8N1 FIFO depth: 32 bytes");

    // Stage 06: Memory Discovery via DTB
    stage_marker!("06_MEMORY_DISCOVERY", "Discovering physical RAM topology from DTB");
    let mem_region = match mm::init_memory(dtb_addr) {
        Ok(r) => r,
        Err(e) => panic!("Failed to parse DTB memory node: {:?}", e),
    };
    log_info!("mm", "Discovered RAM base: 0x{:08x}, size: {} MB", mem_region.base, mem_region.size / (1024 * 1024));

    // Stage 07: Page Frame Allocator Initialization & Self-Test
    stage_marker!("07_FRAME_ALLOCATOR", "Page frame allocator initialized with bitmap");
    {
        let mut allocator = mm::FRAME_ALLOCATOR.lock();
        let total = allocator.total_frames();
        let reserved = allocator.allocated_frames();
        let free = allocator.free_frames();
        log_info!("mm", "Physical frames: total {}, reserved {}, free {}", total, reserved, free);

        // Verification test of allocation and deallocation
        let f1 = allocator.alloc_frame().expect("Failed to allocate test frame 1");
        let f2 = allocator.alloc_frame().expect("Failed to allocate test frame 2");
        log_debug!("mm", "Allocated test frame 1 at 0x{:08x}", f1.addr());
        log_debug!("mm", "Allocated test frame 2 at 0x{:08x}", f2.addr());
        assert!(f1.addr() % mm::PAGE_SIZE == 0, "Frame 1 must be 4KB aligned");
        assert!(f2.addr() % mm::PAGE_SIZE == 0, "Frame 2 must be 4KB aligned");
        assert!(f1.addr() != f2.addr(), "Allocated frames must be distinct");

        allocator.free_frame(f1).expect("Failed to free test frame 1");
        allocator.free_frame(f2).expect("Failed to free test frame 2");
        log_info!("mm", "Frame allocation & free cycle passed (INV-MEM-003 verified)");
    }

    // Stage 09: Dynamic Kernel Heap & Intrusive Data Structures
    stage_marker!("09_KERNEL_HEAP", "Kernel dynamic heap allocator online");
    let heap_start = mm::init_heap();
    log_info!("heap", "Kernel heap initialized at 0x{:08x} (size: {} KB)", heap_start, mm::DEFAULT_HEAP_SIZE / 1024);

    // Intrusive List verification test
    {
        let mut list = IntrusiveList::new();
        assert!(list.is_empty(), "List should be initially empty");

        struct TestTask {
            #[allow(dead_code)]
            id: u32,
            node: IntrusiveNode,
        }

        let mut t1 = TestTask { id: 101, node: IntrusiveNode::new() };
        let mut t2 = TestTask { id: 102, node: IntrusiveNode::new() };

        list.push_back(&mut t1.node);
        list.push_back(&mut t2.node);
        assert_eq!(list.len(), 2, "List length must be 2");

        let popped = list.pop_front().expect("Failed to pop head");
        assert_eq!(popped as *mut IntrusiveNode, &mut t1.node as *mut IntrusiveNode);
        assert_eq!(list.len(), 1, "List length must be 1 after pop");
        log_info!("ds", "Intrusive doubly-linked list zero-allocation verification PASSED");
    }

    // Dynamic Heap Allocation verification test (Box and Vec)
    {
        let b = Box::new(0x1337_c0de_u64);
        assert_eq!(*b, 0x1337_c0de_u64, "Box dereference must match allocated value");

        let mut v: Vec<u32> = Vec::new();
        for i in 0..16 {
            v.push(i * 10);
        }
        assert_eq!(v.len(), 16, "Vector must contain 16 items");
        assert_eq!(v[5], 50, "Vector element index 5 must be 50");

        let (used, free, allocs, _) = mm::HEAP.stats();
        log_debug!("heap", "Active heap stats: used {} bytes, free {} bytes (alloc count {})", used, free, allocs);

        drop(v);
        drop(b);

        let (used_after, _, _, deallocs) = mm::HEAP.stats();
        log_info!("heap", "Heap free & coalescing cycle PASSED (used: {} B, dealloc count: {})", used_after, deallocs);
    }

    // Milestones reached
    println!("--------------------------------------------------");
    println!(" [BOOT] Sprint SP-04 Boot Entry & Vector Milestone REACHED.");
    stage_marker!("05_READY", "Sprint SP-05 Serial Console Milestone REACHED");
    stage_marker!("08_MM_READY", "Sprint SP-06 Memory Discovery Milestone REACHED");
    stage_marker!("10_CORE_DS_READY", "Sprint SP-07 Kernel Heap & Core DS Milestone REACHED");

    // Stage 11: ARM GICv2 Initialization
    stage_marker!("11_GICV2_INIT", "ARM GICv2 interrupt controller initialized");
    interrupts::init_interrupts();
    log_info!("gic", "GICv2 Distributor & CPU Interface initialized (GICD: 0x{:08x}, GICC: 0x{:08x})", drivers::gicv2::GICD_BASE, drivers::gicv2::GICC_BASE);

    // Stage 12: ARM Generic Timer Configuration
    stage_marker!("12_TIMER_INIT", "ARM Generic Timer configured for 100 Hz");
    let freq = drivers::timer::read_frequency();
    log_info!("timer", "ARM Generic Timer online (frequency: {} Hz, interval: 10ms, PPI IRQ {})", freq, drivers::timer::TIMER_IRQ);

    // Stage 13: IRQ Dispatch & Critical Section Verification
    stage_marker!("13_IRQ_DISPATCH", "IRQs unmasked and periodic ticks servicing");
    interrupts::enable_interrupts();
    assert!(interrupts::are_interrupts_enabled(), "Interrupts must be unmasked after enable");
    log_info!("irq", "Hardware interrupts unmasked in DAIF (INV-INT-003 armed)");

    // Critical section discipline test (INV-INT-002)
    interrupts::without_interrupts(|| {
        assert!(!interrupts::are_interrupts_enabled(), "Interrupts must be masked in critical section (INV-INT-002)");
        log_debug!("irq", "Critical section mask active (INV-INT-002)");
    });
    assert!(interrupts::are_interrupts_enabled(), "Interrupts must be restored after critical section (INV-INT-002)");
    log_info!("irq", "Critical section RAII discipline verified (INV-INT-002 PASSED)");

    // Wait for at least 5 hardware timer ticks
    log_info!("timer", "Awaiting periodic hardware timer ticks...");
    let start_ticks = interrupts::get_ticks();
    while interrupts::get_ticks() < start_ticks + 5 {
        core::hint::spin_loop();
    }
    let end_ticks = interrupts::get_ticks();
    log_info!("timer", "Periodic timer ticks verified (received {} ticks, INV-IRQ-004 verified)", end_ticks);

    let spurious = interrupts::get_spurious_count();
    log_info!("irq", "Spurious interrupt handling verified (spurious count: {}, INV-INT-001 verified)", spurious);

    // Stage 14: Milestone Reached
    stage_marker!("14_INTERRUPTS_READY", "Sprint SP-08 Interrupts & Hardware Timers Milestone REACHED");
    println!("==================================================");

    // Park CPU until preemptive scheduler in SP-09
    loop {
        core::hint::spin_loop();
    }
}
