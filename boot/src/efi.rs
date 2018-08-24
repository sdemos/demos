//! the efi module handles all the uefi interactions

use core::{mem, slice};
use uefi::{self, table, table::boot};
use uefi_services;

pub fn alloc_addr<'a>(bytes: usize) -> Result<(&'a mut [u8], usize), uefi::Status> {
    let size = bytes / 4096;
    trace!("allocating {} pages for {} bytes", size, bytes);
    let pages = uefi_services::system_table().boot.allocate_pages(
        boot::AllocateType::AnyPages,
        boot::MemoryType::LoaderData,
        size,
    )?;
    unsafe {
        let ptr = mem::transmute::<_, *mut u8>(pages);
        Ok((slice::from_raw_parts_mut(ptr, bytes), pages))
    }
}

pub fn alloc<'a>(bytes: usize) -> Result<&'a mut [u8], uefi::Status> {
    alloc_addr(bytes).map(|(b, _)| b)
}

pub struct Efi {
    handle: uefi::Handle,
    st: &'static table::SystemTable,
}

impl Efi {
    /// init calls necessary initialization functions to set up our uefi
    /// environment.
    pub fn init(handle: uefi::Handle, st: &'static table::SystemTable) -> Self {
        // initialize uefi_services. this sets up logging and allocation and
        // initializes a globally accessible reference to the system table.
        uefi_services::init(st);

        // get the handle to stdout, and reset it
        let stdout = st.stdout();
        stdout.reset(false)
            .expect("failed to reset stdout");

        // Switch to the maximum supported graphics mode. should be safe to
        // unwrap here because we are getting the last mode, and there should be
        // /any/ modes available.
        let best_mode = stdout.modes().last()
            .expect("failed to get the best mode");
        stdout.set_mode(best_mode)
            .expect("failed to set stdout mode the best mode");

        Efi {handle, st}
    }

    pub fn get_memory_map<'a>(&self) -> (boot::MemoryMapKey, boot::MemoryMapIter<'a>) {
        trace!("getting the memory map");
        let map_size = self.st.boot.memory_map_size();
        // just in case allocating these additional pages increases the size of the
        // memory map, we allocate more space than we need. importantly, this is
        // okay because we aren't using the size of the slice to inform how many
        // entries are in it. the underlying uefi function returns this information
        // explicitly, and the iterator stores it.
        let map_buffer = alloc(map_size + 4096)
            .expect("failed to allocate memory for memory map");
        self.st.boot.memory_map(map_buffer)
            .expect("failed to get memory map")
    }

    pub fn exit_boot_services(self, key: boot::MemoryMapKey) {
        // retrieving an updated memory map again.
        unsafe {
            self.st.boot.exit_boot_services(self.handle, key)
                .expect("failed to exit boot services");
        }
    }
}
