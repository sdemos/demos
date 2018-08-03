//! the demos kernel

#![feature(const_fn)]
#![feature(panic_implementation)]
#![no_std]
#![no_main]

extern crate spin;
extern crate x86_64;

mod serial;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        // initialize serial output
        serial::init();
    }

    let hello = "hello world";
    let mut console = serial::COM1.lock();

    for byte in hello.bytes() {
        console.send(byte);
    }

    // info!("Hello World!");

    loop {}
}

/// panic_impl is a language-level function that rust expects to be provided. it
/// is the function called when something `panic!`s. it is given the file the
/// panic occured in, the line it occured on, and a message about what happened.
/// we print that and then loop forever, since we are in an unrecoverable state
/// but we would like to see what happened.
#[panic_implementation]
#[no_mangle]
pub extern fn panic_impl(_info: &PanicInfo) -> ! {
    // error!("{}", info);

    loop{}
}
