//! Provides facilities for working with page tables.

use bitflags::bitflags;

/// Provides a type for physical addresses.
///
/// Reading/writing a physical address isn't generically possible
/// because all memory access goes through paging. Therefore, the
/// physical address needs to be converted to a linear address
/// before it can be read/written, and how that happens depends on
/// the paging structure in place.
///
/// In an identity mapped system, it's simple, the physical addresses
/// and linear addresses are in 1:1 correspondence.
///
/// Another possibility is to map the entire physical address space into
/// part of the linear address space, that way there can be a simple 1:1
/// mapping from physical to linear.
///
/// Other possibilities such as recursive page table mapping also exist.
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    /// Constructs a new physical address from the provided raw physical
    /// address.
    ///
    /// # Safety
    /// This function performs no checks to see if the physical address is
    /// properly formed.
    pub unsafe fn from_raw_unchecked(raw_physical_address: u64) {
        PhysicalAddress(raw_physical_address);
    }
}

bitflags! {
    pub struct PageTableEntryFlags: u16 {

        // The first byte's flags
        const PRESENT = 0b0000_0001;
        const WRITABLE = 0b0000_0010;
        const USER_ACCESSIBLE = 0b0000_0100;
        const WRITE_THROUGH = 0b0000_1000;
        const DISABLE_CACHE = 0b0001_0000;
        const ACCESSED = 0b0010_0000;
        const DIRTY = 0b0100_0000;
        const HUGE_PAGE = 0b1000_0000;

        // The second byte's flags
        const GLOBAL = 0b0001_0000_0000;
    }
}

/// Provides a wrapper around a page table entry.
///
/// A page table entry is 64-bits long, and is layed out
/// as follows:
///
/// | Bits         | Purpose                 |
/// | -------------| ------------------------|
/// |  0           | Present Flag            |
/// |  1           | Writable Flag           |
/// |  2           | User Accessible Flag    |
/// |  3           | Write Through Flag      |
/// |  4           | Disable Cache Flag      |
/// |  5           | Accessed Flag           |
/// |  6           | Dirty Flag              |
/// |  7           | Huge Page Flag          |
/// |  8           | Global Flag             |
/// |  9 - 11      | Unused/Available        |
/// | 12 - 51      | Physical Address        |
/// | 52 - 62      | Unused/Available        |
/// | 63           | No Execute Bit          |
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const FLAGS_MASK: u16 = 0b0000_0001_1111_1111;

    pub fn flags(&self) -> PageTableEntryFlags {
        unsafe { PageTableEntryFlags::from_bits_unchecked(self.0 as u16 & Self::FLAGS_MASK) }
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}
