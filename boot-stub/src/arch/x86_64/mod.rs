//! Provides access to 64-bit x86 specific
//! functionality.

pub mod paging;
pub mod port;
pub mod serial;

pub mod cr4 {
    use super::paging::PhysicalAddress;

    const FLAGS_OR_PCID_MASK: u64 = 0x0000_0000_0000_0FFF;
    const PML4_MASK: u64 = 0xFFFF_FFFF_FFFF_F000;

    pub fn read() -> (u16, PhysicalAddress) {
        let result: u64;

        unsafe {
            asm!(
            "mov {0}, cr3",
            out(reg) result,
            );
        }

        let flags_or_pcid = (result & FLAGS_OR_PCID_MASK) as u16;

        let pml4 = result & PML4_MASK;
        let pml4_addr = unsafe { PhysicalAddress::from_raw_unchecked(pml4) };

        (flags_or_pcid, pml4_addr)
    }
}
