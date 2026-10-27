/*
 * IndK Linker Script for Qualcomm Snapdragon 732G (SM7150 / Moto G60 'hanoip')
 * Direct kernel execution at physical address 0x80080000
 * Compliant with Qualcomm ABL / LK Boot Architecture (INV-BOOT-001, Rule N-06)
 */

OUTPUT_ARCH(aarch64)
ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x80080000, LENGTH = 128M
}

SECTIONS
{
    . = ORIGIN(RAM);

    /* Early boot entry point & ARM64 Linux header */
    .text.boot : {
        KEEP(*(.text.boot))
    } > RAM

    /* Executable kernel code */
    .text : ALIGN(4K) {
        *(.text .text.*)
    } > RAM

    /* Exception vector table (2,048-byte aligned) */
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

    /* Early kernel stack (512 KB, 16-byte aligned) */
    . = ALIGN(16);
    __stack_bottom = .;
    . += 0x80000;
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

