#![no_std]
#![no_main]

#[macro_use]
extern crate log;
extern crate uefi;
extern crate uefi_services;

use uefi::{Handle, Status, table};

#[no_mangle]
pub extern "C" fn uefi_start(handle: Handle, st: &'static table::SystemTable) -> Status {
    // initialize uefi_services
    // this sets up logging and allocation I guess??
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

    // hold on for a bit so I can see that it worked
    bt.stall(4_000_000);

    // shutdown the computer
    let rt = st.runtime;
    rt.reset(table::runtime::ResetType::Shutdown, Status::Success, None);
}
