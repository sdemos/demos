//! memory is a memory management abstraction layer

mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::*;
pub use self::paging::remap_the_kernel;

use self::paging::PhysicalAddress;

pub const PAGE_SIZE: usize = 4096;

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

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

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }
}

struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}
