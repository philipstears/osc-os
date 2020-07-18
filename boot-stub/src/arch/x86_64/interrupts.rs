use super::paging::LinearAddress;
use super::paging::LogicalAddress;
use super::paging::SegmentSelector;

#[repr(packed)]
#[derive(Debug)]
pub struct IDTRValue {
    limit: u16,
    address: LinearAddress,
}

impl IDTRValue {
    /// Reads the current value of the IDT register.
    pub fn read() -> Self {
        unsafe {
            let mut result: Self = core::mem::MaybeUninit::uninit().assume_init();
            let result_ptr = &mut result as *mut Self;

            asm!("sidt [{0}]", in(reg) result_ptr);

            result
        }
    }

    /// Gets the linear address.
    pub fn address(&self) -> LinearAddress {
        self.address
    }

    /// Gets the limit.
    pub fn limit(&self) -> u16 {
        self.limit
    }
}

/// The type of the IDT entry - either an interrupt gate, or a
/// trap gate.
#[derive(Debug)]
pub enum IDTEntryType {
    InterruptGate,
    TrapGate,
    Invalid(u8),
}

/// Provides access to the data in an entry in an interrupt descriptor
/// table.
///
/// | Bytes    | Length | Purpose                                  |
/// | ---------| -------| -----------------------------------------|
/// |  0 - 1   | 2      | Offset low bits (0..15)                  |
/// |  2 - 3   | 2      | Segment selector                         |
/// |  4       | 1      | Zero and reserved                        |
/// |  5       | 1      | Type and attributes                      |
/// |  6 - 7   | 2      | Offset middle bits (16..31)              |
/// |  8 - 11  | 4      | Offset high bits (32..63)                |
/// | 12 - 16  | 4      | Reserved                                 |
///
/// For more details about the structure of the IDR see Intel 3A - 6.14.1.
///
/// Note that in x86-64, only interrupt gates and trap gates are
/// supported (task gates are deprecated).
#[repr(packed)]
pub struct IDTEntry {
    // These fields are the same as ia32
    offset_lower: u16,
    selector: u16,
    zero_and_reserved: u8,
    type_and_attributes: u8,
    offset_middle: u16,

    // x86-64 doubles the entry size to 16-bytes
    offset_high: u32,
    extended_reserved: u32,
}

impl IDTEntry {
    const PRESENT_MASK: u8 = 0b1000_0000;
    const DPL_MASK: u8 = 0b0110_0000;
    const DPL_SHIFT: usize = 5;
    const S_MASK: u8 = 0b0001_0000;
    const TYPE_MASK: u8 = 0b0000_1111;

    pub fn is_present(&self) -> bool {
        self.type_and_attributes & Self::PRESENT_MASK != 0
    }

    pub fn dpl(&self) -> u8 {
        (self.type_and_attributes & Self::DPL_MASK) >> Self::DPL_SHIFT
    }

    pub fn entry_type(&self) -> IDTEntryType {
        let gate_type = self.type_and_attributes & Self::TYPE_MASK;

        // Table 3-2 in Intel 3A
        match gate_type {
            0b1110 => IDTEntryType::InterruptGate,
            0b1111 => IDTEntryType::TrapGate,
            other => IDTEntryType::Invalid(other),
        }
    }

    pub fn logical_address(&self) -> LogicalAddress {
        let selector = SegmentSelector::from_raw(self.selector);

        let offset = u64::from(self.offset_high) << 32
            | u64::from(self.offset_middle) << 16
            | u64::from(self.offset_lower);

        LogicalAddress::from_selector_and_offset(selector, offset)
    }
}

impl core::fmt::Debug for IDTEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IDTEntry")
            .field("entry_type", &self.entry_type())
            .field("present", &self.is_present())
            .field("dpl", &self.dpl())
            .field("logical_address", &self.logical_address())
            .finish()
    }
}
