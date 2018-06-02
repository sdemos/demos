//! boot is the efi bootloader. it contains the efi entrypoint and handles
//! talking to the firmware to set everything up. it's primary responsibility at
//! this point is to load the actual kernel into high virtual memory and jump
//! into it.

use core::fmt::Write;
use uefi::*;

#[no_mangle]
pub extern "win64" fn efi_entry(
    image_handle: Handle,
    system_table: *const SystemTable,
) -> isize {
    set_system_table(system_table).console().reset();
    protocol::set_current_image(image_handle).unwrap();
    match efi_main(image_handle) {
        Ok(()) => Status::Success as isize,
        Err(status) => status as isize,
    }
}

pub fn efi_main(_image_handle: Handle) -> Result<(), Status> {
    let sys_table = get_system_table();
    let mut cons = sys_table.console();

    cons.write("hello world!\r\n");
    cons.write("1 does this work??\r\n");
    // write!(cons, "hello rusty world!\r\n");
    cons.write("2 does this work??\r\n");
    // cons.write_fmt(format_args!("hello {} world!\r\n", "rusty"));
    ::core::fmt::write(&mut cons, format_args!("hello {} world!\r\n", "rusty"));
    cons.write("3 does this work??\r\n");
    let res = cons.write_str("hello there \r\n");
    cons.write("4 does this work??\r\n");
    match res {
        Ok(_) => cons.write("write_str success\r\n"),
        Err(status) => cons.write("write_str failure\r\n"),
    };
    loop {}

    let boot_services = sys_table.boot_services();
    cons.write("we got the boot services table\r\n");
    let memory_map = match boot_services.get_memory_map() {
        Ok(memory_map) => memory_map,
        Err(status) => {
            cons.write("our function was yelling into the void\r\n");
            loop {}
        }
    };
    cons.write("we got the memory map\r\n");

    for entry in memory_map.descriptors {
        // write!(cons, "{:?}\r\n", entry);
        cons.write("an entry you can't read. haha!\r\n");
    }

    cons.write("we are done with this part of your life\r\n");
    Ok(())
}
