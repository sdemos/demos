//! stack allocator allocates new stacks

use memory::paging::{self, Page, PageIter, ActivePageTable};
use memory::{PAGE_SIZE, FrameAllocator};

#[derive(Debug)]
pub struct StackAllocator {
    range: PageIter,
}

impl StackAllocator {
    pub fn new(page_range: PageIter) -> StackAllocator {
        StackAllocator { range: page_range }
    }

    pub fn alloc_stack<A>(
        &mut self,
        active_table: &mut ActivePageTable,
        frame_allocator: &mut A,
        size_in_pages: usize,
    ) -> Option<Stack>
        where A: FrameAllocator
    {
        if size_in_pages == 0 {
            // it doesn't make any snese to allocate a zero-sized stack
            return None;
        }

        // clone the range, since we only want to change it on success
        let mut range = self.range.clone();

        // try to allocate the stack pages and a guard page
        let guard_page = range.next();
        let stack_start = range.next();
        let stack_end = if size_in_pages == 1 {
            stack_start
        } else {
            // choose the (size_in_pages-2)th element, since index starts at 0
            // and we already allocated the start page.
            range.nth(size_in_pages - 2)
        };

        match (guard_page, stack_start, stack_end) {
            (Some(_), Some(start), Some(end)) => {
                // success! write back updated range
                self.range = range;

                // map stack pages to physical frames
                for page in Page::range_inclusive(start, end) {
                    active_table.map(page, paging::EntryFlags::WRITABLE,
                                     frame_allocator);
                }

                // create a new stack
                let top_of_stack = end.start_address() + PAGE_SIZE;
                Some(Stack::new(top_of_stack, start.start_address()))
            }
            // not enough pages
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Stack {
    top: usize,
    bottom: usize,
}

impl Stack {
    fn new(top: usize, bottom: usize) -> Stack {
        assert!(top > bottom);
        Stack {
            top: top,
            bottom: bottom,
        }
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn bottom(&self) -> usize {
        self.bottom
    }
}
