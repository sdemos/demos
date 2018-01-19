/* tell the linker what our entrypoint label is */
ENTRY(start)

SECTIONS {
    /* set the load address of the first section to 1MiB */
    . = 1M;

    .rodata :
    {
        /* ensure that the multiboot header is at the beginning */
        KEEP(*(.multiboot_header))
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .text :
    {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .data :
    {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .bss :
    {
        *(.bss .bss.*)
        . = ALIGN(4K);
    }

    .got :
    {
        *(.got)
        . = ALIGN(4K);
    }

    .got.plt :
    {
        *(.got.plt)
        . = ALIGN(4K);
    }

    .data.rel.ro : ALIGN(4K)
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

    .gcc_except_table : ALIGN(4K)
    {
        *(.gcc_except_table)
        . = ALIGN(4K);
    }
}
