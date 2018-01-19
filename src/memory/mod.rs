//! memory is a memory management abstraction layer

mod area_frame_allocator;
pub mod heap_allocator;
mod paging;

pub use self::area_frame_allocator::*;
pub use self::paging::remap_the_kernel;

use multiboot2::BootInformation;
use self::paging::{PhysicalAddress, Page};
use {HEAP_START, HEAP_SIZE};

pub const PAGE_SIZE: usize = 4096;

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub fn init(boot_info: &BootInformation) {
    assert_has_not_been_called!("memory::init must only be called once");

    let memory_map_tag = boot_info.memory_map_tag()
        .expect("memory map tag required");
    // get some info about the kernel elf sections
    let elf_sections_tag = boot_info.elf_sections_tag()
        .expect("elf-sections tag required");

    // get the size of the kernel
    let kernel_start = elf_sections_tag.sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr)
        .min()
        .unwrap();
    let kernel_end = elf_sections_tag.sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr + s.size)
        .max()
        .unwrap();
    println!("kernel_start: {:#08x}, kernel_end: {:#08x}",
             kernel_start, kernel_end);

    // get the size of the multiboot area
    println!("multiboot_start: {:#08x}, multiboot_end: {:#08x}",
             boot_info.start_address(),
             boot_info.end_address());

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize,
        boot_info.start_address(), boot_info.end_address(),
        memory_map_tag.memory_areas()
    );

    let mut active_table =
        paging::remap_the_kernel(&mut frame_allocator, boot_info);

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE-1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page,
                         paging::EntryFlags::WRITABLE,
                         &mut frame_allocator);
    }
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
