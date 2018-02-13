//! boot is the efi bootloader. it contains the efi entrypoint and handles
//! talking to the firmware to set everything up. it's primary responsibility at
//! this point is to load the actual kernel into high virtual memory and jump
//! into it.

use uefi::*;

#[no_mangle]
pub extern "win64" fn efi_entry(
    image_handle: Handle,
    system_table: *const SystemTable,
) -> isize {
    set_system_table(system_table).console().reset();
    protocol::set_current_image(image_handle);
    efi_main(image_handle) as isize
}

pub fn efi_main(_image_handle: Handle) -> Status {
    let sys_table = get_system_table();
    let cons = sys_table.console();

    cons.write("hello world!");

    return Status::Success;
}
