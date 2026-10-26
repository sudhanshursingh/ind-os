/*
 * IndK Linker Script for ARM64 QEMU virt
 * Direct kernel ELF execution at physical address 0x40080000
 * Ratified in docs/architecture/qemu-boot-path-spike.md & boot-spec.md
 */

OUTPUT_ARCH(aarch64)
ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x40080000, LENGTH = 128M
}

SECTIONS
{
    . = ORIGIN(RAM);

    /* Early boot entry point */
    .text.boot : {
        KEEP(*(.text.boot))
    } > RAM

    /* Executable kernel code */
    .text : ALIGN(4K) {
        *(.text .text.*)
    } > RAM

    /* Exception vector table (strictly 2,048-byte aligned per INV-BOOT-004) */
    .vectors : ALIGN(2048) {
        __vector_table = .;
        KEEP(*(.vectors))
    } > RAM

    /* Read-only data */
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    } > RAM

    /* Initialized writable data */
    .data : ALIGN(4K) {
        *(.data .data.*)
    } > RAM

    /* Uninitialized zeroed data */
    .bss : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(16);
        __bss_end = .;
    } > RAM

    /* Early kernel stack (512 KB, strictly 16-byte aligned per INV-BOOT-002) */
    . = ALIGN(16);
    __stack_bottom = .;
    . += 0x80000; /* 512 KB early stack */
    __stack_top = .;

    /* End of kernel image marker */
    . = ALIGN(4K);
    __kernel_image_end = .;

    /DISCARD/ : {
        *(.comment)
        *(.gnu*)
        *(.note*)
        *(.eh_frame*)
    }
}
