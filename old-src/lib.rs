#![feature(abi_x86_interrupt)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(global_allocator)]
#![feature(lang_items)]
#![feature(ptr_internals)]
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
#[macro_use]
extern crate log;
extern crate multiboot2;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate spin;
extern crate uefi;
extern crate volatile;
extern crate x86_64;

#[macro_use]
mod macros;

pub mod boot;
mod constants;
mod interrupts;
mod klog;
mod memory;
mod vga;

pub use constants::*;

use memory::heap_allocator::BumpAllocator;

/// because of rust-lang/rust#44113 the global allocator must be defined in the
/// root module.
#[global_allocator]
static ALLOCATOR: BumpAllocator =
    BumpAllocator::new(KERNEL_HEAP_OFFSET, KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE);

/// rust_main is the rust entrypoint of our kernel. it is the function called by
/// the assembly that handles the initial boot. at this point, it is expected
/// that we are in 64-bit (long) mode with paging enabled, and that the first
/// argument is populated with the address of the multiboot information. this
/// function then initialized all the functionality of the kernel.
#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    vga::clear_screen();

    // initialize kernel logging
    klog::init();

    info!("Hello World{}", "!");

    // get some information about memory from the multiboot info structure
    let boot_info = unsafe { multiboot2::load(multiboot_addr) };

    // initialize memory
    let mut memory_controller = memory::init(boot_info);

    // initialize idt
    interrupts::init(&mut memory_controller);

    // do a couple things to make sure various things are still working while I
    // modify the memory model
    use alloc::boxed::Box;
    let mut heap_test = Box::new(100);
    *heap_test -= 15;
    let heap_test2 = Box::new("hello");
    debug!("{:?} {:?}", heap_test, heap_test2);

    info!("it didn't crash!");

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
    error!("PANIC in {} at line {}:\n    {}", file, line, fmt);
    loop {}
}
