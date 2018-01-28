//! mapper is the abstraction of a virtual to physical address map

use core::ptr::Unique;
use memory::{PAGE_SIZE, Frame, FrameAllocator};
use super::{VirtualAddress, PhysicalAddress, Page, ENTRY_COUNT};
use super::entry::*;
use super::table::{self, Table, Level4, Level1};

/// Mapper represents a set of page tables able to map virtual addresses to
/// physical ones. it provides the ability to translate virtual addresses, as
/// well as map virtual addresses to physical addresses.
#[derive(Debug)]
pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Self {
        Mapper {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    /// translate takes a virtual address and traverses the active page tables
    /// to translate it into it's mapped physical address.
    pub fn translate(&self, virtual_addr: VirtualAddress) ->
        Option<PhysicalAddress>
    {
        let offset = virtual_addr % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_addr))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    /// translate_page translates a virtual Page to a physical Frame. there is a
    /// cursory implementation of translating huge pages because our initial
    /// page tables set up in memory use them, but for the most part it just
    /// uses the table indexes in the Page number to traverse the page tables
    /// and return the mapped physical frame. if at any point it doesn't find a
    /// corresponding entry, it returns None.
    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                // 1GiB page?
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(Frame {
                            number: start_frame.number + page.p2_index() *
                                ENTRY_COUNT + page.p1_index(),
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page?
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                            // address must be 2MiB aligned
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number: start_frame.number + page.p1_index(),
                            });
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(huge_page)
    }

    /// map_to takes a virtual Page and maps it to a physical Frame in our
    /// hierarchy of page tables. along the way, it creates any page tables that
    /// don't already exist. it makes sure the Present flag is set in the page
    /// table entry. it contains an assertion that the page is currently unused.
    pub fn map_to<A>(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    )
        where A: FrameAllocator
    {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    /// map takes a virtual Page and maps it to the next available spot in
    /// memory, as provided by the provided allocator.
    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    /// identity_map takes a Frame and maps it's physical address to an
    /// identical virtual address.
    pub fn identity_map<A>(
        &mut self,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    )
        where A: FrameAllocator
    {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    /// temporary_map temporarily maps a page in the kernel's temporary virtual
    /// memory range to the provided frame. the virtual address of the start of
    /// the page is passed to the provided function when it is called, then
    /// unmapped.
    ///
    /// this function is not particularly efficient, it is O(n) on the number
    /// temporary maps currently happening. it's also definitely not thread safe
    /// at all (not that that matters this instant, but in the future, I should
    /// clean this up) TODO << fix those issues
    pub fn temporary_map<A, F>(
        &mut self,
        frame: Frame,
        allocator: &mut A,
        f: F,
    )
        where F: FnOnce(VirtualAddress),
              A: FrameAllocator
    {
        // find the next available page in the temporary virtual memory space
        let mut page = Page::containing_address(::KERNEL_TEMP_OFFSET);
        let end = Page::containing_address(::KERNEL_TEMP_OFFSET + ::PML4_SIZE - 1);
        while self.translate_page(page).is_some() {
            assert!(page < end, "ran out of temp page space? how??");
            page = page + 1;
        }

        self.map_to(page, frame, EntryFlags::WRITABLE, allocator);

        f(page.start_address());

        self.unmap(page, allocator);
    }

    /// temporary_table_map temporarily maps a page in the kernel's temporary
    /// virtual memory range to the provided frame. the page is passed to the
    /// provided function when it is called as a page table, then unmapped.
    ///
    /// TODO: there is another issue that I didn't anticipate - because we now
    /// need the allocator passed to us, callers can't use it in the closure.
    /// this is particularly painful for the with function, which would use this
    /// functionality. this function will only be useful once I fix that
    /// problem, but for now I'm going to leave this half-implemented.
    pub fn temporary_table_map<A, F>(
        &mut self,
        frame: Frame,
        allocator: &mut A,
        f: F
    )
        where F: FnOnce(&mut Table<Level1>),
              A: FrameAllocator
    {
        self.temporary_map(frame, allocator, |addr| {
            let t = unsafe {
                &mut *(addr as *mut Table<Level1>)
            };
            f(t)
        })
    }

    /// unmap sets the entry defined by the provided virtual Page to be unused.
    /// it asserts that it is currently mapped. it panics if it fails to get the
    /// next table and doesn't currently support huge pages. once it sets the
    /// entry as unused it flushes the tlb and deallocates the frame.
    ///
    /// currently it doesn't actually deallocate the frame, since our current
    /// main memory allocator doesn't implement deallocation and instead just
    /// leaks the memory.
    pub fn unmap<A>(&mut self, page: Page, _allocator: &mut A)
        where A: FrameAllocator
    {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("mapping code does not support huge pages");
        let _frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;
        tlb::flush(VirtualAddress(page.start_address()));

        // TODO free p(1,2,3) table if empty
        // TODO implement deallocate_frame
        // allocator.deallocate_frame(frame);
    }
}
