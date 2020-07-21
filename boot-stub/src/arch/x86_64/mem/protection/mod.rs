//! Provides facilities for working with the x86-64 architecture's
//! protection facilities, specifically the global and local
//! descriptor tables.
use super::LinearAddress;

mod gdt;
pub use gdt::*;

/// The type of the segment descriptor.
///
/// In x86-64, call gates, IDT gates, LDT and TSS descriptors
/// are expanded to 16-bytes.
///
/// See 3.5 in Intel 3A.
#[derive(Debug)]
pub enum SegmentDescriptorType {
    Code {
        accessed: bool,
        read_only: bool,
        conforming: bool,
    },

    Data {
        accessed: bool,
        write_enabled: bool,
        expansion_direction: ExpandDirection,
    },

    LDT,
    AvailableTSS,
    BusyTSS,

    CallGate,

    /// The upper eight bytes of a 16-byte descriptor.
    Upper,

    /// An unknown system descriptor.
    InvalidSystem(u8),
}

/// The expansion direction of a data segment.
#[derive(Debug)]
pub enum ExpandDirection {
    Down,
    Up,
}

/// Provides access to the data in segment descriptor.
///
/// | Bytes    | Length | Purpose                                  |
/// | ---------| -------| -----------------------------------------|
/// | 0 - 1    | 2      | Limit (0..15)                            |
/// | 2 - 3    | 2      | Base (0..15)                             |
/// | 4        | 1      | Base (16..23)                            |
/// | 5        | 1      | Type and attributes                      |
/// | 6        | 1      | Limit (16..19) and attributes            |
/// | 7        | 1      | Base (24..31)                            |
///
/// Note that in x86-64, only interrupt gates and trap gates are
/// supported (task gates are deprecated).
#[repr(packed)]
pub struct SegmentDescriptor {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    type_and_attributes: u8,
    limit_high_and_attributes: u8,
    base_high: u8,
}

impl SegmentDescriptor {
    const PRESENT_MASK: u8 = 0b1000_0000;

    const DPL_MASK: u8 = 0b0110_0000;
    const DPL_SHIFT: usize = 5;

    // The interpretation of types depends on the
    // so-called S-Field
    const S_MASK: u8 = 0b0001_0000;

    // If it's clear, the type is a set of flags
    // for either a code or a data segment
    const EXEC_MASK: u8 = 0b0000_1000; // Whether it's a code segment
    const DC_MASK: u8 = 0b0000_0100; // Direction for Data, Conforming for Code
    const RW_MASK: u8 = 0b0000_0010; // Readable for Data, Writable for Code
    const ACCESSED_MASK: u8 = 0b0000_0001; // Accessed for both Data and Code

    // Otherwise it's value indicating the type
    // of the system segment
    const TYPE_MASK: u8 = 0b0000_1111;

    pub fn is_present(&self) -> bool {
        self.type_and_attributes & Self::PRESENT_MASK != 0
    }

    pub fn dpl(&self) -> u8 {
        (self.type_and_attributes & Self::DPL_MASK) >> Self::DPL_SHIFT
    }

    pub fn entry_type(&self) -> SegmentDescriptorType {
        let attrs = self.type_and_attributes;
        let s_is_set = attrs & Self::S_MASK != 0;

        if s_is_set {
            let dc = attrs & Self::DC_MASK != 0;
            let rw = attrs & Self::RW_MASK != 0;
            let accessed = attrs & Self::ACCESSED_MASK != 0;

            if attrs & Self::EXEC_MASK != 0 {
                SegmentDescriptorType::Code {
                    accessed,
                    read_only: rw,
                    conforming: dc,
                }
            } else {
                SegmentDescriptorType::Data {
                    accessed,
                    write_enabled: rw,
                    expansion_direction: match dc {
                        true => ExpandDirection::Down,
                        false => ExpandDirection::Up,
                    },
                }
            }
        } else {
            let gate_type = self.type_and_attributes & Self::TYPE_MASK;

            // Table 3-2 in Intel 3A
            match gate_type {
                0b0000 => SegmentDescriptorType::Upper,
                0b0010 => SegmentDescriptorType::LDT,
                0b1001 => SegmentDescriptorType::AvailableTSS,
                0b1011 => SegmentDescriptorType::BusyTSS,
                0b1100 => SegmentDescriptorType::CallGate,
                other => SegmentDescriptorType::InvalidSystem(other),
            }
        }
    }
}

impl core::fmt::Debug for SegmentDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_present() {
            f.debug_struct("SegmentDescriptor")
                .field("entry_type", &self.entry_type())
                .field("present", &true)
                .field("dpl", &self.dpl())
                .finish()
        } else {
            f.debug_struct("SegmentDescriptor")
                .field("present", &false)
                .finish()
        }
    }
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct SegmentDescriptorTableRef {
    limit: u16,
    address: LinearAddress,
}

impl SegmentDescriptorTableRef {
    /// Gets the linear address.
    pub fn address(&self) -> LinearAddress {
        self.address
    }

    /// Gets the limit.
    pub fn limit(&self) -> u16 {
        self.limit
    }

    /// Gets the count of entries.
    pub fn count(&self) -> usize {
        (self.limit as usize + 1) / core::mem::size_of::<SegmentDescriptor>()
    }

    pub unsafe fn entries(&self) -> &[SegmentDescriptor] {
        let first_ptr = self.address.to_raw() as *const SegmentDescriptor;
        let count = self.count();
        core::slice::from_raw_parts(first_ptr, count)
    }

    pub unsafe fn entries_mut(&self) -> &mut [SegmentDescriptor] {
        let first_ptr = self.address.to_raw() as *mut SegmentDescriptor;
        let count = self.count();
        core::slice::from_raw_parts_mut(first_ptr, count)
    }
}
