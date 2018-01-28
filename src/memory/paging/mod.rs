//! paging keeps track of the virtual page table

mod entry;
mod mapper;
mod table;
mod temporary_page;

pub use self::entry::*;
pub use self::mapper::Mapper;

use core::ops::{Add, Deref, DerefMut};
use memory::{PAGE_SIZE, Frame, FrameAllocator};
use multiboot2::BootInformation;
use self::temporary_page::TemporaryPage;

/// ENTRY_COUNT defines the number of entries in every page table.
const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

/// init initializes the paging that will actually be used by the kernel during
/// normal runtime. the assembly code that runs on startup sets up an extremely
/// simple set of page tables that point at huge pages (which we don't currently
/// fully support) and isn't very fleshed out. we also rely on identity mapping
/// for the initial kernel and stack execution, since it is the easiest way to
/// boot up, but during normal execution I would like to have the kernel mapped
/// in the higher half of memory.
pub fn init<A>(
    allocator: &mut A,
    boot_info: &BootInformation,
) -> ActivePageTable
    where A: FrameAllocator
{
    assert_has_not_been_called!("paging::init must only be called once");

    let mut temporary_page = TemporaryPage::new(
        Page::containing_address(::KERNEL_TEMP_OFFSET),
        allocator,
    );

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("memory map tag required");

        for section in elf_sections_tag.sections() {
            use self::entry::EntryFlags;
            if !section.is_allocated() {
                // section is not loaded into memory
                continue;
            }

            assert!(section.start_address() % PAGE_SIZE == 0,
                    "sections need to be page aligned");

            info!("mapping section at addr: {:#x}, size: {:#x}",
                     section.addr, section.size);

            let flags = EntryFlags::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.start_address());
            let end_frame = Frame::containing_address(section.end_address() - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        // identity map the VGA text buffer
        let vga_buffer_frame = Frame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, allocator);

        // identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }
    });

    let old_table = active_table.switch(new_table);

    // turn the old p4 page into a guard page for some basic stack overflow
    // protections. a guard page is just an unallocated (and unallocatable) page
    // that gets stomped on if something overruns the stack, which triggers a
    // page fault.
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);

    active_table
}

#[derive(Debug)]
pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    /// new returns a new ActivePageTable struct. this function is unsafe and
    /// not public because there should only ever be one ActivePageTable, since
    /// it uses a special virtual memory address to reference itself.
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    /// with provides a mechanism for modifying the entries in page tables that
    /// aren't the current active one. it takes a function to execute on a
    /// Mapper struct. it then executes this function after overwriting the
    /// recursive mapping in the in the active table to point at the inactive
    /// table. it requires a temporary page so we can keep a reference to the
    /// original p4 table, since the normal mechanism of modifying page tables
    /// is being hijacked by the inactive table.
    ///
    /// the mechanism takes advantage of the recursive mapping used to modify
    /// page tables. normally, in order to modify a page table in the current
    /// active page table hierarchy, you use a virtual address that makes the
    /// paging functionality look in the last entry of the top level page table.
    /// this entry is mapped to itself. it then looks up the next table as if it
    /// is looking in the level 3 page table. using this, we can refer to page
    /// tables in level 1 by looping once, level 2 by looping twice, level 3 by
    /// looping 3 times, and level 4 by looping four times.
    ///
    /// since with is not changing the active page table, almost all virtual
    /// addresses are still the same. the only ones that aren't are the magic
    /// ones we use to modify the page tables. this lets us modify inactive page
    /// tables using the same mechanism we do to modify the active one.
    pub fn with<F>(
        &mut self,
        table: &mut InactivePageTable,
        temporary_page: &mut temporary_page::TemporaryPage,
        f: F,
    )
        where F: FnOnce(&mut Mapper)
    {
        use x86_64::instructions::tlb;
        use x86_64::registers::control_regs;

        {
            // backup the p4 frame
            let backup = Frame::containing_address(control_regs::cr3().0 as usize);

            // map temporary_page to current p4 table
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(),
                                   EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[511].set(backup,
                              EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();
        }

        temporary_page.unmap(self);
    }

    /// switch swaps the current active page table that the cpu uses to map
    /// virtual addresses to physical ones. in the current x86_64 implementation
    /// of the kernel, it writes the address of the beginning of the frame
    /// containing the page table into the CR3 register. it returns the previous
    /// page table as an InactivePageTable.
    ///
    /// this actually has an interesting implementation, because you will notice
    /// that we never seem to update the active table struct, which seems like
    /// it would break our mapping functions. that's because the pointer to the
    /// active table is a virtual memory address that specifies our recursive
    /// mapping, specifically 0xffffffff_fffff000. we recursively point at the
    /// p4 table 4 times, and that allows us to modify the page table.
    ///
    /// this is the same mechanism used in the mapping and table creation
    /// functions to modify existing page tables using virtual addresses. it's a
    /// pretty clever approach!
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86_64::PhysicalAddress;
        use x86_64::registers::control_regs;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(control_regs::cr3().0 as usize),
        };

        unsafe {
            control_regs::cr3_write(PhysicalAddress(
                new_table.p4_frame.start_address() as u64));
        }

        old_table
    }
}

/// InactivePageTable keeps track of a set of page tables, starting with a level
/// 4 page table. Inactive page tables refer directly to a frame instead of
/// using Mapper because the Mapper struct takes advantage of the recursive
/// mapping we set up for page tables to always point at the active table.
/// unfortunately, that means that we can't modify the inactive page tables
/// directly using the same mechanism we do otherwise. instead, we can use the
/// ActivePageTable::with function to
#[derive(Debug)]
pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    /// new returns a newly allocated InactivePageTable. the table is entirely
    /// empty, with every entry set to zero, except the last one, which is the
    /// recursive mapping.
    pub fn new(
        frame: Frame,
        active_table: &mut ActivePageTable,
        temporary_page: &mut TemporaryPage,
    ) -> InactivePageTable {
        {
            let table = temporary_page
                .map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(),
                           EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}

/// Page represents a PAGE_SIZE chunk of virtual address space.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Page {
    // the number stored by the page is derived from the virtual addresses
    // contained within it. specifically, this number is the virtual address
    // divided by PAGE_SIZE truncated, which removes the 12-bit offset into the
    // page that is bits 0-11 of a virtual address. See the `start_address`
    // function and it's inverse `containing_address` for more details on Page
    // construction.
    number: usize,
}

impl Page {
    /// p4_index returns the 9-bit index into the level 4 paging table. this
    /// still works because even though the representation of the page number is
    /// the virtual address / PAGE_SIZE, that just means (at the binary level)
    /// that all the bits are shifted over by 12.
    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    /// p3_index returns the 9-bit index into the level 3 paging table.
    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    /// p2_index returns the 9-bit index into the level 2 paging table.
    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    /// p1_index returns the 9-bit index into the level 1 paging table.
    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

    /// start_address returns the virtual address of the start of the page.
    pub fn start_address(&self) -> VirtualAddress {
        self.number * PAGE_SIZE
    }

    /// containing_address takes a virtual address and returns the page in which
    /// that address resides. it asserts that the provided virtual address is
    /// sign-extended in the top 16 bits (48-63). it does no validation that the
    /// page being returned is actually mapped in the current active page table.
    pub fn containing_address(addr: VirtualAddress) -> Page {
        assert!(addr < 0x0000_8000_0000_0000 ||
                addr >= 0xffff_8000_0000_0000,
                "invalid address: 0x{:x}", addr);
        Page {
            number: addr / PAGE_SIZE,
        }
    }

    /// range_inclusive returns an iterator from the start Page to the end Page.
    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        Page {
            number: self.number + rhs,
        }
    }
}

/// PageIter is the iterator for pages. There is a potential problem with this,
/// but I've yet to confirm it's actually a problem or think of any possible
/// solutions. Basically, because of the sign-extension present in the high bits
/// of a virtual address, and the way that the "page number" is constructed,
/// which is what this iterates over, it is conceivable that one could construct
/// an iterator starting at one valid page containing address
/// 0x0000_7fff_ffff_ffff and ending at another valid page containing address
/// 0xffff_8000_0000_0000. each page is okay on it's own, but none of the pages
/// between those addresses exist, because the sign extension requires those to
/// be invalid. the containing_address function properly asserts this fact, but
/// the iterator might still manually create a page, since it gets the next page
/// by simply adding one to the current page number. Anyway, I don't know if
/// it's an actual problem or if I will literally ever run into a manifestation
/// of it, but it seems like it would be possible to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}
