//! the demos bootloader

#![no_std]
#![no_main]

extern crate goblin;
#[macro_use]
extern crate log;
extern crate uefi;
extern crate uefi_services;
extern crate uefi_utils;

use core::{mem, slice};

use goblin::elf;
use uefi::{Handle, Status, table, table::boot, proto::media};
use uefi_utils::proto::find_protocol;

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

    // this is essentially going to be a uefi os bootloader. it shouldn't be too
    // complicated. all it has to do is load the kernel into memory, collect any
    // information the kernel needs to run, then call the kernel with that
    // information.
    //
    // first, we need to load the kernel from disk into memory, using one of the
    // uefi functions for reading files from disk. then we need to somehow
    // obtain a reference to that memory with the type &[u8] that we can pass to
    // goblin. goblin will then parse the elf headers of our kernel. we take the
    // e_entry from the elf headers and convert that to a function that we can
    // call as the entry point to our kernel. then we can collect the rest of
    // the information, which for now mostly means retrieving the memory map,
    // then call exit boot services and invoke our discovered `_start` function.
    let entry = load_kernel();

    info!("Grabbing the memory map");
    let bt = st.boot;
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
    // info!("map size, in bytes: {}", map_size);
    // info!("pages required for the memory map: {}", buf_size);
    // info!("memory map key: {:?}", key);
    // info!("number of memory descriptors: {}", descs.len());
    // info!("descriptors:");
    for desc in descs {
        // info!("{:?}", desc);
        info!("start: {:#010x}\tpages: {}\ttype: {:?}",
              desc.phys_start, desc.page_count, desc.ty);
    }

    // exit boot services
    // info!("exiting boot services and starting the kernel");
    unsafe {
        bt.exit_boot_services(handle, key)
            .expect("failed to exit boot services");
    }
    // TODO: check the return and possibly attempt to exit again, perhaps after
    // retrieving an updated memory map again.

    // start the kernel
    entry();

    // entry doesn't return!
    unreachable!();

    // loop {}

    // shutdown the computer
    // let rt = st.runtime;
    // rt.reset(table::runtime::ResetType::Shutdown, Status::Success, None);
}

fn load_kernel() -> (extern "C" fn() -> !) {
    // get the SimpleFileSystem protocol. this is a little weird. afaict, all
    // uefi function calls basically have an implicit parameter - the handle
    // used to call the protocol. protocols are only valid on certain handles.
    // the ones that the simplefilesystem protocol are valid on are device path
    // handles, and that's how it decides which device to act on. the
    // find_protocol function gets all handles for which the simplefilesystem
    // protocol is valid and returns the first one (if there is one).
    //
    // a better approach for this particular use case will probably be to
    // somehow pinpoint the exact device we expect the kernel to be on. one way
    // to do this would be to use the LoadedImageProtocol to get the device that
    // our bootloader is loaded from, using the handle to ourselves that we are
    // passed as a parameter. then, we can explicitly get the SimpleFileSystem
    // protocol for that handle. this approach is complicated by the fact that
    // our underlying uefi library doesn't currently implement the
    // LoadedImageProtocol, so I'm hoping this approach will work instead, at
    // least for now.
    let mut sfs_ptr = find_protocol::<media::SimpleFileSystem>()
        .expect("failed to get SimpleFileSystem protocol: no protocol returned");

    // turn the pointer into a reference.
    let sfs = unsafe { sfs_ptr.as_mut() };

    // open the root directory of the device
    let mut root = sfs.open_volume().expect("failed to open volume");

    // the uefi file protocol allows you to open files on a filesystem using the
    // name, relative to the location of a file you already have open. using
    // open_volume with the simple file system protocol provides us with a file
    // that represents the root directory of the filesystem. use that to open
    // our kernel, which our buildsystem places at `/kernel`.
    let mut kernel_file = root.open("kernel",
                                media::FileMode::READ,
                                media::FileAttribute::NONE)
        .expect("failed to open kernel");

    // find the size of the file by setting the position to the end of the file
    // and getting the position of both sides. then set it back to the beginning
    // of the file so we can read it. the start_pos should always be zero but
    // I'm not confident enough in that assumption to rely on it, so we might as
    // well just do the simple math.
    let start_pos = kernel_file.get_position()
        .expect("failed to get start position in kernel file");
    kernel_file.set_position(0xFFFFFFFFFFFFFFFF)
        .expect("failed to set position to end of kernel file");
    let end_pos = kernel_file.get_position()
        .expect("failed to get end position in kernel file");
    kernel_file.set_position(0)
        .expect("failed to set position to start of kernel file");
    info!("start position in kernel file: {}", start_pos);
    info!("end position in kernel file: {}", end_pos);
    let kernel_size = (end_pos - start_pos) as usize;

    // use the size to allocate a buffer in memory to read the kernel into. we
    // don't /really/ care where the kernel ends up at this point, since it's
    // going to remap itself when it sets up the virtual memory map anyway. we
    // just need to keep track of the address so we can pass it to the kernel so
    // it can actually do that.
    let bt = uefi_services::system_table().boot;
    let buf_size = kernel_size / 4096;
    info!("allocating {} pages for the kernel", buf_size);
    let pages = bt.allocate_pages(
        boot::AllocateType::AnyPages,
        boot::MemoryType::LoaderData,
        buf_size,
    ).expect("failed to allocate memory for the kernel");

    let kernel = unsafe {
        let ptr = mem::transmute::<_, *mut u8>(pages);
        slice::from_raw_parts_mut(ptr, kernel_size)
    };

    // read the kernel from the file into memory
    info!("reading kernel into memory");
    let bytes_read = kernel_file.read(kernel)
        .expect("failed to read kernel into memory");
    if kernel_size != bytes_read {
        panic!("bytes read: {}\nkernel size: {}", bytes_read, kernel_size);
    }

    info!("kernel starts at: {:#010x}", pages);

    // okay next we use goblin to parse the elf headers of our kernel
    let kernel_elf = elf::Elf::parse(kernel)
        .expect("failed to parse kernel elf");

    let e_entry = kernel_elf.header.e_entry;
    info!("kernel elf e_entry: {:#010x}", e_entry);
    let entry_ptr = e_entry + pages as u64;
    info!("entry point: {:#010x}", entry_ptr);

    // now turn this entry point into a callable function
    unsafe {
        core::mem::transmute::<u64, extern "C" fn() -> !>(entry_ptr)
    }
}