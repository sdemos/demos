#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(unique_unchecked)]
#![no_std]

extern crate rlibc;
extern crate spin;
extern crate volatile;

#[macro_use]
mod vga;

#[no_mangle]
pub extern fn rust_main() {
    vga::clear_screen();

    println!("Hello World{}", "!");
    println!("{}", { println!("inner"); "outer" });

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt() -> ! {loop {}}
