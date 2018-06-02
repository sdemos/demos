/// consts holds a bunch of constants for general use across the kernel.
/// primarily it is for holding various virtual memory location constants, for
/// things like where the kernel and the kernel heap and temporary memory go.
///
/// in general our memory map mimics redox's memory map. the lower 256 entries,
/// which equates to half of the level 4 page table, is reserved for userspace.
/// the top entry (the 511th) is used to recursively map the level 4 page table
/// to itself, which is used for page table modification. the second from the
/// top entry (510th) is used for the kernel itself. we also use the 509th entry
/// for the kernel heap.

/// PML4_SIZE is the size in virtual memory space of a single entry in the level
/// 4 page table. this is equivalent to the total range of a level 3 page table,
/// which makes sense, since each entry in the level 4 page table points at a
/// level 3 one.
pub const PML4_SIZE: usize = 0x0000_0080_0000_0000;
/// PML4_MASK can be used to convert a normal virtual address into one which
/// only has it's level 4 page table index
pub const PML4_MASK: usize = 0x0000_ff80_0000_0000;

/// RECURSIVE_PAGE_OFFSET is the offset in the level 4 page table that contains
/// the location of the recursive mapping. this is the way that redox defined
/// it. I'm not sure if I'm a fan of the weird casting. it basically takes
/// advantage of how negative numbers are represented in two's compliment.
pub const RECURSIVE_PAGE_OFFSET: usize = (-(PML4_SIZE as isize)) as usize;
/// RECURSIVE_PAGE_PML4 is the entry we use for the recursive mapping. this
/// should be 511 with our current page scheme.
pub const RECURSIVE_PAGE_PML4_INDEX: usize = (RECURSIVE_PAGE_OFFSET & PML4_MASK) / PML4_SIZE;

/// KERNEL_OFFSET is the offset used to refer to the kernel.
pub const KERNEL_OFFSET: usize = RECURSIVE_PAGE_OFFSET - PML4_SIZE;
pub const KERNEL_PML4_INDEX: usize = (KERNEL_OFFSET & PML4_MASK) / PML4_SIZE;

/// offset to the kernel heap
pub const KERNEL_HEAP_OFFSET: usize = KERNEL_OFFSET - PML4_SIZE;
pub const KERNEL_HEAP_PML4_INDEX: usize = (KERNEL_HEAP_OFFSET & PML4_MASK) / PML4_SIZE;
/// size of the kernel heap
pub const KERNEL_HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// offset to temporary pages for temporary things
pub const KERNEL_TEMP_OFFSET: usize = KERNEL_HEAP_OFFSET - PML4_SIZE;
pub const KERNEL_TEMP_PML4_INDEX: usize = (KERNEL_TEMP_OFFSET & PML4_MASK) / PML4_SIZE;

/// offset to userspace. I will probably have to revisit this when I actually
/// have a userspace.
pub const USER_OFFSET: usize = 0;
pub const USER_PML4_INDEX: usize = (USER_OFFSET & PML4_MASK) / PML4_SIZE;
