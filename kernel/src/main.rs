//! the demos kernel

#![feature(const_fn)]
#![feature(lang_items)]
#![feature(panic_implementation)]
#![feature(ptr_internals)]
#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate once;
extern crate spin;
extern crate volatile;

#[macro_use]
mod macros;

mod klog;
mod vga;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga::clear_screen();

    // initialize kernel logging
    klog::init();

    info!("Hello World!");

    loop {}
}

/// panic_impl is a language-level function that rust expects to be provided. it
/// is the function called when something `panic!`s. it is given the file the
/// panic occured in, the line it occured on, and a message about what happened.
/// we print that and then loop forever, since we are in an unrecoverable state
/// but we would like to see what happened.
#[panic_implementation]
#[no_mangle]
pub extern fn panic_impl(info: &PanicInfo) -> ! {
    error!("{}", info);

    loop{}
}
