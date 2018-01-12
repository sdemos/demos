/* tell the linker what our entrypoint label is */
ENTRY(start)

SECTIONS {
    /* set the load address of the first section to 1MiB */
    . = 1M;

    .boot :
    {
        /* ensure that the multiboot header is at the beginning */
        *(.multiboot_header)
    }

    .text :
    {
        *(.text)
    }
}
