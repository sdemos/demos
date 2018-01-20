#![feature(abi_x86_interrupt)]
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
extern crate bit_field;
#[macro_use]
extern crate lazy_static;
extern crate multiboot2;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

#[macro_use]
mod vga;

mod interrupts;
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

    println!("Hello World{}", "!");

    // get some information about memory from the multiboot info structure
    let boot_info = unsafe { multiboot2::load(multiboot_addr) };

    // enable various cpu features needed for our memory management strategy
    enable_nxe_bit();
    enable_write_protect_bit();

    // initialize memory
    let mut memory_controller = memory::init(boot_info);

    // initialize idt
    interrupts::init(&mut memory_controller);

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
