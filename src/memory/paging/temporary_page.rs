//! temporary_page contains the structures for temporarily mapping frames in the
//! page table. it is essentially a thin wrapper around normal page table
//! mapping. I'm not entirely convinced that having this buys us anything, so
//! far it has only caused me confusion when working with stuff. I think I would
//! prefer to have a temporary mapping function that executes a closure with the
//! frame mapped at a particular address, then unmaps it immediately. I'm not
//! sure if that will actually provide all the functionality I need yet though,
//! so we will see.
//!
//! until then I'm not going to bother really documenting anything in this
//! module.

use super::{Page, ActivePageTable, VirtualAddress};
use super::table::{Table, Level1};
use memory::{Frame, FrameAllocator};

#[derive(Debug)]
struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
    fn new<A>(allocator: &mut A) -> TinyAllocator
        where A: FrameAllocator
    {
        let mut f = || allocator.allocate_frame();
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }
}

impl FrameAllocator for TinyAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("tiny allocator can only hold 3 frames.");
    }
}

#[derive(Debug)]
pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    pub fn new<A>(page: Page, allocator: &mut A) -> TemporaryPage
        where A: FrameAllocator
    {
        TemporaryPage {
            page: page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    /// maps the temporary page to the given frame in the active page table.
    /// returns the start address of the temporary page.
    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable)
               -> VirtualAddress
    {
        use super::entry::EntryFlags;

        assert!(active_table.translate_page(self.page).is_none(),
                "temporary page is already mapped");
        active_table.map_to(self.page, frame, EntryFlags::WRITABLE,
                            &mut self.allocator);
        self.page.start_address()
    }

    /// unmaps the temporary page in the active table
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator)
    }

    /// maps the temporary page to the given page table frame in the active
    /// table. returns a reference to the now mapped table.
    pub fn map_table_frame(
        &mut self,
        frame: Frame,
        active_table: &mut ActivePageTable
    ) -> &mut Table<Level1>
    {
        unsafe {
            &mut *(self.map(frame, active_table) as *mut Table<Level1>)
        }
    }
}
