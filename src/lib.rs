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

/// rust_main is the rust entrypoint of our kernel. it is the function called by
/// the assembly that handles the initial boot. at this point, it is expected
/// that we are in 64-bit (long) mode with paging enabled, and that the first
/// argument is populated with the address of the multiboot information. this
/// function then initialized all the functionality of the kernel.
#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    vga::clear_screen();

    println!("Hello World{}", "!");

    // get some information about memory from the multiboot info structure
    let boot_info = unsafe { multiboot2::load(multiboot_addr) };

    // initialize memory
    let mut memory_controller = memory::init(boot_info);

    // initialize idt
    interrupts::init(&mut memory_controller);

    println!("it didn't crash!");

    loop {}
}

/// eh_personality is a language-level function that rust expects to be
/// provided. I'm not clear on the exact purpose of this function or when it
/// gets called. I think it has something to do with llvm. right now it doesn't
/// do anything, but it needs to exist so it can be linked against.
#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

/// panic_fmt is a language-level function that rust expects to be provided. it
/// is the function called when something `panic!`s. it is given the file the
/// panic occured in, the line it occured on, and a message about what happened.
/// we print that and then loop forever, since we are in an unrecoverable state
/// but we would like to see what happened.
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
