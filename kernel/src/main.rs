//! Ind OS Microkernel (IndK) Main Entry Point
//! Sprint SP-06: Memory Discovery & Page Frame Allocator

#![no_std]
#![no_main]

pub mod arch;
pub mod console;
pub mod drivers;
pub mod logger;
pub mod mm;
pub mod panic;
pub mod sync;

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

    // Milestones reached
    println!("--------------------------------------------------");
    println!(" [BOOT] Sprint SP-04 Boot Entry & Vector Milestone REACHED.");
    stage_marker!("05_READY", "Sprint SP-05 Serial Console Milestone REACHED");
    stage_marker!("08_MM_READY", "Sprint SP-06 Memory Discovery Milestone REACHED");
    println!("==================================================");

    // Park CPU until timer interrupts in SP-08
    loop {
        core::hint::spin_loop();
    }
}
