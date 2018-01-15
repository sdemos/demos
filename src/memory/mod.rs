//! memory is a memory management abstraction layer

mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::*;

use self::paging::PhysicalAddress;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

impl Frame {
    fn containing_address(addr: usize) -> Frame {
        Frame {
            number: addr / PAGE_SIZE,
        }
    }

    fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    /// clone
    /// this is very importantly a private function. the only valid way to have
    /// a frame is to get one from an allocator. we restrict this function to
    /// only us so we can guarantee that if a frame is passed to us, it has not
    /// yet been used.
    fn clone(&self) -> Frame {
        Frame { number: self.number }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}
