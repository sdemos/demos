//! memory is a memory management abstraction layer

mod area_frame_allocator;
pub mod heap_allocator;
mod paging;
mod stack_allocator;

pub use self::area_frame_allocator::*;
pub use self::paging::remap_the_kernel;
pub use self::stack_allocator::Stack;

use multiboot2::BootInformation;
use self::paging::{PhysicalAddress, Page};
use {HEAP_START, HEAP_SIZE};
use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};
use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

pub const PAGE_SIZE: usize = 4096;

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub fn init(boot_info: &BootInformation) -> MemoryController {
    assert_has_not_been_called!("memory::init must only be called once");

    // enable various cpu features needed for our memory management strategy
    enable_nxe_bit();
    enable_write_protect_bit();

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
    info!("kernel_start: {:#08x}, kernel_end: {:#08x}",
             kernel_start, kernel_end);

    // get the size of the multiboot area
    info!("multiboot_start: {:#08x}, multiboot_end: {:#08x}",
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

    let stack_allocator = {
        let stack_alloc_start = heap_end_page + 1;
        let stack_alloc_end = stack_alloc_start + 100;
        let stack_alloc_range =
            Page::range_inclusive(stack_alloc_start, stack_alloc_end);
        stack_allocator::StackAllocator::new(stack_alloc_range)
    };

    MemoryController {
        active_table: active_table,
        frame_allocator: frame_allocator,
        stack_allocator: stack_allocator,
    }
}

pub struct MemoryController {
    active_table: paging::ActivePageTable,
    frame_allocator: AreaFrameAllocator,
    stack_allocator: stack_allocator::StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController {
            ref mut active_table,
            ref mut frame_allocator,
            ref mut stack_allocator,
        } = self;
        stack_allocator.alloc_stack(
            active_table,
            frame_allocator,
            size_in_pages,
        )
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

/// the EntryFlags::NO_EXECUTE bit is disabled by default on x86_64. this
/// function uses the Extended Feature Enable Register (EFER) to set the NXE
/// bit, which enables using the EntryFlags::NO_EXECUTE bit on page tables.
fn enable_nxe_bit() {
    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

/// by default, the write protection bit is ignored when the cpu is in kernel
/// mode. for security and bug safety, have the cpu respect the bit even in
/// kernel mode by turning on write protection, by setting the WRITE_PROTECT bit
/// in the CR0 register.
fn enable_write_protect_bit() {
    unsafe {
        cr0_write(cr0() | Cr0::WRITE_PROTECT);
    }
}
