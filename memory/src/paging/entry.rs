//! entry defines entries in the page table

use Frame;
use multiboot2::ElfSection;

bitflags! {
    /// EntryFlags represents a set of flags that can be set for an entry in a
    /// page table.
    pub struct EntryFlags: u64 {
        /// the page is currently in memory.
        const PRESENT         = 1 << 0;
        /// the cpu is allowed to write to this page. note that write protection
        /// is disabled by default when the cpu is running a process in kernel
        /// mode. in order to enable it for ourselves, we must set the
        /// WRITE_PROTECT bit in the CR0 register. we do this at memory
        /// initialization (see memory::enable_write_protect_bit() and
        /// memory::init())
        const WRITABLE        = 1 << 1;
        /// if set, the cpu is allowed to access this page when running
        /// processes that aren't in kernel mode. otherwise, only kernel mode
        /// processes are allowed to access it.
        const USER_ACCESSIBLE = 1 << 2;
        /// writes go directly to memory when using this page
        const WRITE_THROUGH   = 1 << 3;
        /// no cache is used for this page
        const NO_CACHE        = 1 << 4;
        /// the cpu sets this bit when this page is used
        const ACCESSED        = 1 << 5;
        /// the cpu sets this bit when a write to this page occurs
        const DIRTY           = 1 << 6;
        /// huge page changes behavior depending on what table this entry is in.
        /// in level 1 and level 4 page tables, this bit is required to be zero.
        /// in level 2 page tables, it means this entry points to a 2MiB page.
        /// in level 3 page tables, it means this entry points to a 1GiB page.
        const HUGE_PAGE       = 1 << 7;
        /// page isn't flushed from caches on address space switch. this feature
        /// must be enabled before it can be used by setting the PGE bit of the
        /// CR4 register.
        const GLOBAL          = 1 << 8;
        /// forbid executing code on this page. this feature must be enabled by
        /// setting the NXE bit in the EFER register. we do this at memory
        /// initialization (see memory::enable_nxe_bit() and memory::init())
        const NO_EXECUTE      = 1 << 63;
    }
}

impl EntryFlags {
    /// from_elf_section_flags uses the ElfSection struct from the multiboot2
    /// crate to convert elf section flags into their equivalent entry flags.
    /// this is used when identity mapping existing sections.
    pub fn from_elf_section_flags(section: &ElfSection) -> EntryFlags {
        use multiboot2::{ELF_SECTION_ALLOCATED, ELF_SECTION_WRITABLE, ELF_SECTION_EXECUTABLE};

        let mut flags = EntryFlags::empty();

        if section.flags().contains(ELF_SECTION_ALLOCATED) {
            // section is loaded into memory
            flags = flags | EntryFlags::PRESENT;
        }
        if section.flags().contains(ELF_SECTION_WRITABLE) {
            flags = flags | EntryFlags::WRITABLE;
        }
        if !section.flags().contains(ELF_SECTION_EXECUTABLE) {
            // the section is NOT marked with the elf executable flag, so don't
            // allow execution.
            flags = flags | EntryFlags::NO_EXECUTE;
        }

        flags
    }
}

/// Entry represents an entry in a paging table. each table entry is 8 bytes (64
/// bits). the exact meaning of the entry depends on the table it's in, and not
/// all bits in the entry correspond to the address it's pointing at. a chart of
/// the exact breakdown can be found elsewhere
/// (https://os.phil-opp.com/page-tables is a good reference) but in particular,
/// bits 12-51 contain the physical address of the next entity, which is either
/// the actual frame (if this is a level 1 table or huge page is set on a level
/// 2 or 3 table entry) or the next page table. see the table module for more
/// details on how it's actually used.
///
/// another small detail, we understand that an unused entry is entirely zero.
pub struct Entry(u64);

impl Entry {
    /// is_unused checks if a table entry is unused.
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// set_unused zeros out the entire entry.
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// flags gets the EntryFlags for this particular entry.
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// pointed frame returns None if the Present flag is not set, or returns
    /// the frame the entry points to.
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(Frame::containing_address(self.0 as usize & 0x000fffff_fffff000))
        } else {
            None
        }
    }

    /// set sets the entry to point to a particular frame, with a provided set
    /// of flags. it asserts that the frame passed does not point to the zero
    /// address.
    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        assert!(frame.start_address() & !0x000fffff_fffff000 == 0);
        self.0 = (frame.start_address() as u64) | flags.bits();
    }
}
