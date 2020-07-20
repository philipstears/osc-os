//! Provides facilities for working with page tables.
use super::PhysicalAddress;
use bitflags::bitflags;
use core::ops::{Index, IndexMut};

bitflags! {
    pub struct PageTableEntryFlags: u64 {

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
        const GLOBAL = 0b0000_0001 << 8;

        // The eighth byte's flags
        const NO_EXECUTE = 0b1000_0000 << 56;
    }
}

/// Provides a wrapper around a page table entry.
///
/// A page table entry is 64-bits long, and is laid out
/// as follows:
///
/// | Bits         | Length | Purpose                 |
/// | -------------| -------| ------------------------|
/// |  0           |  1     | Present Flag            |
/// |  1           |  1     | Writable Flag           |
/// |  2           |  1     | User Accessible Flag    |
/// |  3           |  1     | Write Through Flag      |
/// |  4           |  1     | Disable Cache Flag      |
/// |  5           |  1     | Accessed Flag           |
/// |  6           |  1     | Dirty Flag              |
/// |  7           |  1     | Huge Page Flag          |
/// |  8           |  1     | Global Flag             |
/// |  9 - 11      |  3     | Unused/Available        |
/// | 12 - 51      | 40     | Physical Address        |
/// | 52 - 62      | 11     | Unused/Available        |
/// | 63           |  1     | No Execute Bit          |
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    // NOTE: we only present the bits that are defined
    const FLAGS_MASK_B1: u64 = 0b1111_1111;
    const FLAGS_MASK_B2: u64 = 0b0000_0001;
    const FLAGS_MASK_B3: u64 = 0b0000_0000;
    const FLAGS_MASK_B4: u64 = 0b0000_0000;
    const FLAGS_MASK_B5: u64 = 0b0000_0000;
    const FLAGS_MASK_B6: u64 = 0b0000_0000;
    const FLAGS_MASK_B7: u64 = 0b0000_0000;
    const FLAGS_MASK_B8: u64 = 0b1000_0000;

    const FLAGS_MASK: u64 = Self::FLAGS_MASK_B8 << 56
        | Self::FLAGS_MASK_B7 << 48
        | Self::FLAGS_MASK_B6 << 40
        | Self::FLAGS_MASK_B5 << 32
        | Self::FLAGS_MASK_B4 << 24
        | Self::FLAGS_MASK_B3 << 16
        | Self::FLAGS_MASK_B2 << 8
        | Self::FLAGS_MASK_B1;

    const PHYSICAL_ADDRESS_MASK: u64 = 0x000F_FFFF_FFFF_F000;

    pub fn flags(&self) -> PageTableEntryFlags {
        let flags = self.0 & Self::FLAGS_MASK;

        // NOTE: This is safe because we already mask off
        // any flags that aren't defined.
        unsafe { PageTableEntryFlags::from_bits_unchecked(flags) }
    }

    pub fn physical_address(&self) -> PhysicalAddress {
        // NOTE: The physical address already has the lower 12-bits
        // set to zero due to the way the page table entry is laid out.
        let physical_address = self.0 & Self::PHYSICAL_ADDRESS_MASK;
        unsafe { PhysicalAddress::from_raw_unchecked(physical_address) }
    }
}

impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("flags", &self.flags())
            .field("physical_address", &self.physical_address())
            .finish()
    }
}

/// Provides support for inspecting and modifying a page table.
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}
