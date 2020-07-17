use super::paging::PhysicalAddress;

/// Provides support for inspecting/manipulating the
/// contents of the third control register.
#[repr(transparent)]
pub struct CR3Value(u64);

impl CR3Value {
    const FLAGS_OR_PCID_MASK: u64 = 0x0000_0000_0000_0FFF;
    const PML4_MASK: u64 = 0xFFFF_FFFF_FFFF_F000;

    /// Reads the current value of CR3.
    pub fn read() -> Self {
        let result: u64;

        unsafe {
            asm!(
            "mov {0}, cr3",
            out(reg) result,
            );
        }

        Self(result)
    }

    /// Gets the physical address of the root page table
    /// (Page Map Level 4).
    pub fn pml4_address(&self) -> PhysicalAddress {
        let pml4 = self.0 & Self::PML4_MASK;
        unsafe { PhysicalAddress::from_raw_unchecked(pml4) }
    }

    /// Gets the flags (if CR4.PCID is 0), or the
    /// PCID (if CR4.PCID is 1).
    pub fn flags_or_pcid(&self) -> u16 {
        (self.0 & Self::FLAGS_OR_PCID_MASK) as u16
    }
}
