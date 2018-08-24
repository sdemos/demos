//! the demos bootloader

#![no_std]
#![no_main]

extern crate goblin;
#[macro_use]
extern crate log;
extern crate memory;
extern crate uefi;
extern crate uefi_services;
extern crate uefi_utils;

mod efi;
mod kernel;

use efi::Efi;
use kernel::Kernel;
use uefi::{Handle, Status, table};

/// uefi_start is the entrypoint called by the uefi firmware. It is defined as
/// the entrypoint as part of the target spec. It is responsible for setting up
/// all the stuff we need to actually start our kernel, loading it, exiting boot
/// services cleanly, and then calling the kernel entrypoint. The kernel has to
/// be compiled as a separate binary that exists at a known location on disk and
/// gets loaded into memory by this function. It has to be separate because it
/// will have a different global allocator, a different executable format, a
/// different logger, and we want to write our own implementation of panic, oom,
/// and eh_personality.
#[no_mangle]
pub extern "C" fn uefi_start(
    handle: Handle,
    st: &'static table::SystemTable,
) -> Status {
    // initialize our runtime environment
    let efi = Efi::init(handle, st);

    // welcome! to the bootloader
    info!("# DemOS #");
    info!("Image handle: {:?}", handle);

    // load the kernel into memory
    let kernel = Kernel::load();

    // grab the memory map from the firmware
    let (key, desc) = efi.get_memory_map();

    // in my experience, using logging functions changes the memory map key,
    // once we get the memory map, we don't log anymore. either way, once we
    // exit boot services, we can't use the uefi logging functionality anyway.

    // exit boot services.
    efi.exit_boot_services(key);

    // we are now fully in control of the system, and therefore responsible for
    // all i/o and memory functionality. it's time to remap the kernel to it's
    // expected location in the higher half of memory.
    kernel.remap();

    // start the kernel
    kernel.enter();

    // enter doesn't return!
    // unreachable!();
}
