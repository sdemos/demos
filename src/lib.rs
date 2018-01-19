#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(global_allocator)]
#![feature(lang_items)]
#![feature(unique)]
#![feature(unique_unchecked)]
#![no_std]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate multiboot2;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

#[macro_use]
mod vga;
mod memory;

use memory::FrameAllocator;
use memory::heap_allocator::BumpAllocator;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// because of rust-lang/rust#44113 the global allocator must be defined in the
/// root module.
#[global_allocator]
static ALLOCATOR: BumpAllocator =
    BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);

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

    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize,
        multiboot_start, multiboot_end,
        memory_map_tag.memory_areas()
    );

    enable_nxe_bit();
    enable_write_protect_bit();
    memory::remap_the_kernel(&mut frame_allocator, boot_info);
    frame_allocator.allocate_frame();
    println!("it didn't crash!");

    loop {}
}

/// the EntryFlags::NO_EXECUTE bit is disabled by default on x86_64. this
/// function uses the Extended Feature Enable Register (EFER) to set the NXE
/// bit, which enables using the EntryFlags::NO_EXECUTE bit on page tables.
fn enable_nxe_bit() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

/// by default, the write protection bit is ignored when the cpu is in kernel
/// mode. for security and bug safety, have the cpu respect the bit even in
/// kernel mode by turning on write protection, by setting the WRITE_PROTECT bit
/// in the CR0 register.
fn enable_write_protect_bit() {
    use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

    unsafe {
        cr0_write(cr0() | Cr0::WRITE_PROTECT);
    }
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(
    fmt: core::fmt::Arguments,
    file: &'static str,
    line: u32,
) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
    loop {}
}
