//! the demos bootloader

#![no_std]
#![no_main]

#[macro_use]
extern crate log;
extern crate uefi;
extern crate uefi_services;

use core::{mem, slice};

use uefi::{Handle, Status, table, table::boot};

/// uefi_start is the entrypoint called by the uefi firmware. It is defined as
/// the entrypoint as part of the target spec. It is responsible for setting up
/// all the stuff we need to actually start our kernel, loading it, exiting boot
/// services cleanly, and then calling the kernel entrypoint. The kernel has to
/// be compiled as a separate binary that exists at a known location on disk and
/// gets loaded into memory by this function. It has to be separate because it
/// will have a different global allocator, a different executable format, a
/// different logger we want to write our own implementation of panic, oom, and
/// eh_personality.
#[no_mangle]
pub extern "C" fn uefi_start(handle: Handle, st: &'static table::SystemTable) -> Status {
    // initialize uefi_services. this sets up logging and allocation and
    // initializes a globally accessible reference to the system table.
    uefi_services::init(st);

    // get the handle to stdout, and reset it
    let stdout = st.stdout();
    stdout.reset(false).unwrap();

    // Switch to the maximum supported graphics mode.
    let best_mode = stdout.modes().last().unwrap();
    stdout.set_mode(best_mode).unwrap();

    // try printing something!
    info!("# DemOS #");
    info!("Image handle: {:?}", handle);

    let bt = st.boot;

    info!("Grabbing the memory map");
    let map_size = bt.memory_map_size();
    // in case allocating our buffer requires an additional page, we allocate
    // more space than we need.
    let buf_size = (map_size / 4096) + 1;
    let pages = bt.allocate_pages(
        boot::AllocateType::AnyPages,
        boot::MemoryType::LoaderData,
        buf_size,
    ).expect("failed to allocate memory for memory map");
    let buffer = unsafe {
        let ptr = mem::transmute::<_, *mut u8>(pages);
        slice::from_raw_parts_mut(ptr, buf_size * 4096)
    };

    let (key, descs) = bt.memory_map(buffer).expect("failed to get memory map");
    info!("memory map key: {:?}", key);
    info!("number of memory descriptors: {}", descs.len());
    // for desc in descs {
    //     info!("{:?}", desc);
    // }

    // hold on for a bit so I can see that it worked
    // bt.stall(4_000_000);
    loop {}

    // shutdown the computer
    let rt = st.runtime;
    rt.reset(table::runtime::ResetType::Shutdown, Status::Success, None);
}
