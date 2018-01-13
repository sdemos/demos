#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(unique_unchecked)]
#![no_std]

extern crate multiboot2;
extern crate rlibc;
extern crate spin;
extern crate volatile;

mod memory;
#[macro_use]
mod vga;

#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    vga::clear_screen();

    println!("Hello World!");

    // get some information about memory from the multiboot info structure
    let boot_info = unsafe { multiboot2::load(multiboot_addr) };
    let memory_map_tag = boot_info.memory_map_tag()
        .expect("memory map tag required");

    println!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("    start: 0x{:08x}, length: 0x{:08x}",
                 area.base_addr, area.length);
    }

    // get some info about the kernel elf sections
    let elf_sections_tag = boot_info.elf_sections_tag()
        .expect("elf-sections tag required");

    println!("kernel sections:");
    for section in elf_sections_tag.sections() {
        println!("    addr: 0x{:08x}, size: 0x{:08x}, flags: 0x{:08x}",
                 section.addr, section.size, section.flags);
    }

    // get the size of the kernel
    let kernel_start = elf_sections_tag.sections().map(|s| s.addr)
        .min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size)
        .max().unwrap();
    println!("kernel_start: 0x{:08x}, kernel_end: 0x{:08x}",
             kernel_start, kernel_end);

    // get the size of the multiboot area
    let multiboot_start = multiboot_addr;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    println!("multiboot_start: 0x{:08x}, multiboot_end: 0x{:08x}",
             multiboot_start, multiboot_end);

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt() -> ! {loop {}}
