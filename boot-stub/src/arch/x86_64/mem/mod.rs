pub mod paging;

#[derive(Copy, Clone)]
pub struct LogicalAddress {
    selector: SegmentSelector,
    offset: u64,
}

impl LogicalAddress {
    pub fn from_selector_and_offset(selector: SegmentSelector, offset: u64) -> Self {
        Self { selector, offset }
    }
}

impl core::fmt::Debug for LogicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("LogicalAddress")
            .field(&self.selector)
            .field(&format_args!("{:#018X}", self.offset))
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    pub fn from_cs_register() -> Self {
        let result;

        unsafe {
            asm!("mov {0:x}, cs", out(reg) result);
        }

        Self::from_raw(result)
    }

    pub fn from_raw(segment_selector: u16) -> Self {
        Self(segment_selector)
    }

    pub fn to_raw(&self) -> u16 {
        self.0
    }

    pub fn index(&self) -> u16 {
        self.0 >> 3
    }

    pub fn rpl(&self) -> u16 {
        self.0 & 0b011
    }

    pub fn indicator(&self) -> TableIndicator {
        let global = (self.0 & 0b100) == 0;

        if global {
            TableIndicator::Global
        } else {
            TableIndicator::Local
        }
    }
}

impl core::fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SegmentSelector")
            .field(&self.indicator())
            .field(&self.rpl())
            .field(&self.index())
            .finish()
    }
}

#[derive(Debug)]
pub enum TableIndicator {
    Global,
    Local,
}

#[derive(Copy, Clone)]
pub struct LinearAddress(u64);

impl LinearAddress {
    pub unsafe fn from_raw_unchecked(raw_linear_address: u64) -> Self {
        Self(raw_linear_address)
    }

    pub fn offset(&self) -> u16 {
        (self.0 & 0b1111_1111_1111) as u16
    }

    pub fn level1(&self) -> u16 {
        ((self.0 >> 12) & 0b1_1111_1111) as u16
    }

    pub fn level2(&self) -> u16 {
        ((self.0 >> 21) & 0b1_1111_1111) as u16
    }

    pub fn level3(&self) -> u16 {
        ((self.0 >> 30) & 0b1_1111_1111) as u16
    }

    pub fn level4(&self) -> u16 {
        ((self.0 >> 39) & 0b1_1111_1111) as u16
    }

    pub fn to_raw(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Debug for LinearAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("LinearAddress")
            .field(&format_args!("{}", self.level4()))
            .field(&format_args!("{}", self.level3()))
            .field(&format_args!("{}", self.level2()))
            .field(&format_args!("{}", self.level1()))
            .field(&format_args!("{:#05X}", self.offset()))
            .finish()
    }
}

/// Provides a type for physical addresses.
///
/// On the x86-64 architecture, physical addresses are fundamentally
/// limited to a maximum of 52 bits.
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
#[derive(Copy, Clone)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    /// Constructs a new physical address from the provided raw physical
    /// address.
    ///
    /// # Safety
    /// This function performs no checks to see if the physical address is
    /// properly formed.
    pub unsafe fn from_raw_unchecked(raw_physical_address: u64) -> Self {
        PhysicalAddress(raw_physical_address)
    }

    /// Gets the underlying raw physical address.
    pub fn to_raw(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PhysicalAddress")
            .field(&format_args!("{:#018X}", self.0))
            .finish()
    }
}
