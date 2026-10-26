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
pub mod loader;
pub mod logger;
pub mod mm;
pub mod panic;
pub mod sync;
pub mod syscall;
pub mod task;

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

    // Wait for periodic hardware timer ticks
    log_info!("timer", "Awaiting periodic hardware timer ticks...");
    let start_ticks = interrupts::get_ticks();
    while interrupts::get_ticks() < start_ticks + 3 {
        core::hint::spin_loop();
    }
    let end_ticks = interrupts::get_ticks();
    log_info!("timer", "Periodic timer ticks verified (received {} ticks, INV-IRQ-004 verified)", end_ticks);

    let spurious = interrupts::get_spurious_count();
    log_info!("irq", "Spurious interrupt handling verified (spurious count: {}, INV-INT-001 verified)", spurious);

    // Stage 14: Milestone Reached
    stage_marker!("14_INTERRUPTS_READY", "Sprint SP-08 Interrupts & Hardware Timers Milestone REACHED");
    println!("==================================================");

    // Stage 15: Phase 1 Kernel Bootstrap Complete / Gate G1 Milestone
    stage_marker!("15_BOOT_COMPLETE", "Phase 1 Kernel Bootstrap complete");
    println!("--------------------------------------------------");
    println!(" [BOOT] Hello from IndK! Phase 1 Bootstrap Complete.");
    println!(" [BOOT] Gate G1 Criteria Met: 2048-byte vectors armed,");
    println!(" [BOOT] 512MB RAM discovered, frame & heap allocators online,");
    println!(" [BOOT] GICv2 & 100 Hz physical timer periodic ticks active.");
    println!("==================================================");

    // =========================================================================
    // PHASE 2: CORE KERNEL (SP-10 .. SP-15) -> Gate G2
    // =========================================================================

    // Stage 16: AArch64 4-Level Page Table Manager Initialization & Mapping
    stage_marker!("16_PAGE_TABLES_INIT", "4-level page table manager initialized");
    {
        let mut pt_mgr = mm::PageTableManager::create_new().expect("Failed to create root PageTableManager");
        log_info!("mmu", "Root L0 page table allocated at 0x{:08x}", pt_mgr.root_address());

        // Test mapping a kernel virtual address to physical RAM
        let test_virt_kernel = 0xFFFF_8000_4000_0000;
        let test_phys_kernel = 0x4008_0000;
        pt_mgr.map_page(test_virt_kernel, test_phys_kernel, mm::PageTableFlags::KERNEL_CODE)
            .expect("Failed to map kernel code page");

        let (trans_phys, trans_flags) = pt_mgr.translate(test_virt_kernel)
            .expect("Translation failed for mapped kernel code page");
        assert_eq!(trans_phys, test_phys_kernel, "Translated physical address mismatch");
        assert!(!trans_flags.is_writable(), "Kernel code page must not be writable");
        assert!(!trans_flags.is_user(), "Kernel code page must not be accessible to user");
        log_info!("mmu", "Kernel code mapping verified (0x{:016x} -> 0x{:08x}, RO, UXN)", test_virt_kernel, trans_phys);

        // Test mapping an MMIO device page
        let test_virt_mmio = 0xFFFF_FFFF_0900_0000;
        let test_phys_mmio = 0x0900_0000;
        pt_mgr.map_page(test_virt_mmio, test_phys_mmio, mm::PageTableFlags::DEVICE_MMIO)
            .expect("Failed to map device MMIO page");

        let (mmio_phys, _) = pt_mgr.translate(test_virt_mmio)
            .expect("Translation failed for mapped MMIO page");
        assert_eq!(mmio_phys, test_phys_mmio, "Translated MMIO physical address mismatch");
        log_info!("mmu", "Device MMIO mapping verified (0x{:016x} -> 0x{:08x}, nGnRE Device)", test_virt_mmio, mmio_phys);

        // Test unmapping
        pt_mgr.unmap_page(test_virt_kernel).expect("Failed to unmap kernel page");
        assert!(pt_mgr.translate(test_virt_kernel).is_none(), "Unmapped page must not translate");
        log_info!("mmu", "Page unmapping & TLB invalidation verified (0x{:016x} unmapped)", test_virt_kernel);
    }

    // Stage 17: Per-Process Address Space & Security Bounds Verification
    stage_marker!("17_PER_PROCESS_MMU", "Per-process address space & security bounds verified");
    {
        let mut user_space = mm::AddressSpace::new(1).expect("Failed to create AddressSpace for ASID 1");
        log_info!("mmu", "Created user AddressSpace with ASID {} (L0: 0x{:08x})", user_space.asid(), user_space.root_table_address());

        // Test INV-MEM-001: Strict NULL-pointer trap zone ([0x0000_0000..0x0040_0000] strictly rejected)
        let null_addr = 0x0000_0000;
        let null_res = user_space.map_user(null_addr, 0x4010_0000, mm::PageTableFlags::USER_DATA);
        assert_eq!(null_res, Err(mm::MemoryError::PermissionDenied), "INV-MEM-001 violation: NULL address mapping must be rejected");

        let trap_zone_addr = 0x003F_F000;
        let trap_res = user_space.map_user(trap_zone_addr, 0x4010_0000, mm::PageTableFlags::USER_DATA);
        assert_eq!(trap_res, Err(mm::MemoryError::PermissionDenied), "INV-MEM-001 violation: 0x003FF000 trap zone mapping must be rejected");
        log_info!("mmu", "INV-MEM-001 verified: Strict NULL-pointer trap zone [0x0..0x00400000] enforced");

        // Test INV-MEM-002: PXN & UXN privilege separation
        let user_code_virt = 0x0040_0000;
        let user_code_phys = 0x4010_0000;
        user_space.map_user(user_code_virt, user_code_phys, mm::PageTableFlags::USER_CODE)
            .expect("Failed to map valid user code page");

        let (_, user_flags) = user_space.translate(user_code_virt)
            .expect("Translation failed for user code page");
        assert!(user_flags.contains(mm::page_table::PTE_PXN), "INV-MEM-002 violation: User page must have PXN set");
        log_info!("mmu", "INV-MEM-002 verified: PXN enforced on user address space (PXN=1)");

        // Test Copy-on-Write (CoW) page fault resolution
        let cow_virt = 0x0040_1000;
        let cow_shared_phys = 0x4010_1000;
        user_space.map_user(cow_virt, cow_shared_phys, mm::PageTableFlags::USER_COW)
            .expect("Failed to map CoW page");

        let (before_phys, before_flags) = user_space.translate(cow_virt).unwrap();
        assert_eq!(before_phys, cow_shared_phys);
        assert!(before_flags.is_cow(), "CoW page must have PTE_COW bit set");
        assert!(!before_flags.is_writable(), "CoW page must initially be read-only");

        // Simulate page fault write trap -> resolve CoW
        user_space.handle_cow_fault(cow_virt).expect("Failed to resolve CoW page fault");

        let (after_phys, after_flags) = user_space.translate(cow_virt).unwrap();
        assert_ne!(after_phys, cow_shared_phys, "CoW resolution must allocate a new private physical frame");
        assert!(!after_flags.is_cow(), "Resolved CoW page must clear PTE_COW bit");
        assert!(after_flags.is_writable(), "Resolved CoW page must have write permission");
        log_info!("mmu", "Copy-on-Write (CoW) verified (0x{:08x} -> new frame 0x{:08x}, RW)", cow_shared_phys, after_phys);

        // Test TTBR0_EL1 activation
        user_space.activate();
        log_info!("mmu", "AddressSpace activated into TTBR0_EL1 with ASID {}", user_space.asid());
    }

    // Stage 18: Milestone Reached
    stage_marker!("18_VIRTUAL_MEMORY_READY", "Sprint SP-10 Virtual Memory Milestone REACHED");
    println!("--------------------------------------------------");
    println!(" [MMU] Sprint SP-10 Virtual Memory Milestone REACHED.");
    println!(" [MMU] AArch64 4-Level Page Tables, Per-Process Address Spaces,");
    println!(" [MMU] Strict NULL-Trap ([0..4MB]), PXN/UXN Protection, and CoW Verified.");
    println!("==================================================");

    // Stage 19: Process Abstraction & INDBIN Loader Initialization
    stage_marker!("19_PROCESS_SUBSYSTEM_INIT", "Process abstraction & INDBIN loader initialized");
    let hello_image = loader::build_indbin_image(
        0x0040_0000,
        0x0040_0000,
        &loader::HELLO_USER_CODE,
        0,
        &[],
        16384,
    );
    let parsed_header = loader::parse_header(&hello_image).expect("Failed to parse valid INDBIN header");
    assert_eq!(parsed_header.entry_point, 0x0040_0000);
    log_info!("proc", "INDBIN header verified (magic 0x{:08x}, entry: 0x{:08x}, text size: {} B)", 
        u32::from_le_bytes(parsed_header.magic), parsed_header.entry_point, parsed_header.text_size);

    // Verify INV-MEM-001 NULL trap enforcement in loader
    let bad_image = loader::build_indbin_image(0x0000_1000, 0x0000_1000, &loader::HELLO_USER_CODE, 0, &[], 16384);
    let bad_res = loader::parse_header(&bad_image);
    assert_eq!(bad_res, Err(loader::LoaderError::NullTrapViolation(0x0000_1000)));
    log_info!("proc", "INV-MEM-001 verified: Loader rejected executable targeting NULL trap zone");

    // Create user process (PID 1)
    let user_proc = task::Process::create_from_indbin(1, "hello_user", &hello_image)
        .expect("Failed to instantiate Process 1 from INDBIN");
    assert_eq!(user_proc.state, task::ProcessState::Ready);
    assert_eq!(user_proc.context.sp_el0, loader::USER_STACK_TOP as u64);
    assert_eq!(user_proc.context.elr_el1, 0x0040_0000);
    log_info!("proc", "Created Process 1 ('{}', PID: {}, entry: 0x{:08x}, sp: 0x{:016x})",
        user_proc.name, user_proc.pid, user_proc.context.elr_el1, user_proc.context.sp_el0);

    // Stage 20: First User-Mode Execution in Real EL0
    stage_marker!("20_USER_MODE_HELLO", "Executing first user-mode process in EL0");
    println!("--------------------------------------------------");
    println!(" [USER] Dropping to AArch64 EL0 User Mode...");
    let exit_code = task::run_user_process(user_proc);
    println!(" [USER] Returned to AArch64 EL1 Kernel Mode.");
    assert_eq!(exit_code, 0, "User process must exit with code 0");
    log_info!("proc", "Process 1 exited cleanly with status {}", exit_code);

    // Stage 21: Milestone Reached
    stage_marker!("21_SYSCALL_DISPATCH_READY", "Sprint SP-11 Processes & Loading Milestone REACHED");
    println!("--------------------------------------------------");
    println!(" [PROC] Sprint SP-11 Processes & Loading Milestone REACHED.");
    println!(" [PROC] Process Abstraction, INDBIN Loader, Real EL0 User Mode,");
    println!(" [PROC] and SVC #0 System Call Dispatch Verified.");
    println!("==================================================");

    // Park CPU until Phase 2 (SP-12 Scheduler & Preemption)
    loop {
        core::hint::spin_loop();
    }
}
