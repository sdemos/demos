//! the kernel module deals with loading, remapping, and entering the kernel,
//! and keeping track of all the important kernel-related details.

use core::mem;
use efi;
use goblin::elf;
use uefi::{proto::media};
use uefi_utils::proto::find_protocol;

type EntryFunc = extern "C" fn() -> !;

pub struct Kernel {
    entry: u64,
    addr: usize,
}

impl Kernel {
    pub fn load() -> Self {
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
        let kernel_size = (end_pos - start_pos) as usize;

        // use the size to allocate a buffer in memory to read the kernel into. we
        // don't /really/ care where the kernel ends up at this point, since it's
        // going to remap itself when it sets up the virtual memory map anyway. we
        // just need to keep track of the address so we can pass it to the kernel so
        // it can actually do that.
        let (kernel, kernel_addr) = efi::alloc_addr(kernel_size)
            .expect("failed to allocate memory for the kernel");

        // read the kernel from the file into memory
        let bytes_read = kernel_file.read(kernel)
            .expect("failed to read kernel into memory");
        // sanity check: make sure everything has the right number of bytes
        if kernel_size != bytes_read {
            panic!("bytes read: {}\nkernel size: {}", bytes_read, kernel_size);
        }

        // okay next we use goblin to parse the elf headers of our kernel
        let kernel_elf = elf::Elf::parse(kernel)
            .expect("failed to parse kernel elf");

        let entry_ptr = kernel_elf.header.e_entry;

        Kernel {
            entry: entry_ptr,
            addr: kernel_addr,
        }
    }

    pub fn remap(&self) {
        // uefi dumps us into long mode, so paging is already enabled. it identity
        // maps everything we care about, so the table we have right now isn't
        // particularly useful, but it's a start.
        //
        // the function we created when we loaded the kernel uses the e_entry field
        // of the elf headers. this field, when the object file is created by the
        // linker, is filled with a value under the assumption that the binary will
        // be loaded in a known location. we can (and do) control that location
        // through the linker script.
        //
        // importantly, this known address is _not_ a valid physical address. it's
        // much to large for that. instead, we tell the linker that our kernel
        // expects to be loaded into a really high page in memory. we use the second
        // to last entry in the pml4 table for our kernel.
        //
        // what this means for the bootloader is that we need to massage the page
        // tables to reflect this without mapping ourselves to a different place (so
        // we can continue to execute). we take advantage of the natural break
        // between the bootloader and the kernel to do this without much fuss. we
        // keep ourselves (and all the other stuff the firmware gave us with
        // get_memory_map) identity mapped, but map the kernel's physical location
        // to that place in high memory where we want it, swap in our new page
        // tables, and then call the entry function. once the kernel is running, we
        // will clean up memory there.
        //
        // some temporary things about this implementation - we are currently only
        // interested in getting the kernel called, so right now the kernel isn't
        // getting the information it needs. eventually, I will need to figure out
        // how to send it the information it craves, such as where it is in physical
        // memory, it's size, and the whole uefi memory map, so it can make informed
        // decisions.

        // to use our regular paging functionality, we need an allocator.
    }

    pub fn enter(self) -> ! {
        // now turn this entry point into a callable function
        unsafe {
            mem::transmute::<u64, EntryFunc>(self.entry)()
        };
    }
}
