use super::mem::LinearAddress;

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct GDTRValue {
    limit: u16,
    address: LinearAddress,
}

impl GDTRValue {
    /// Reads the current value of the GDT register.
    pub fn read() -> Self {
        unsafe {
            let mut result: Self = core::mem::MaybeUninit::uninit().assume_init();
            let result_ptr = &mut result as *mut Self;

            asm!("sgdt [{0}]", in(reg) result_ptr);

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

/// The type of the GDT entry.
///
/// In amd64, call gates, IDT gates, LDT and TSS descriptors
/// are expanded to 16-bytes.
///
/// 3.5 in Intel 3A.
#[derive(Debug)]
pub enum GDTEntryType {
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

#[derive(Debug)]
pub enum ExpandDirection {
    Down,
    Up,
}

/// Provides access to the data in an entry in a global descriptor
/// table.
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
/// For more details about the structure of the GDT see Intel 3A - 3.4.5
///
/// Note that in x86-64, only interrupt gates and trap gates are
/// supported (task gates are deprecated).
#[repr(packed)]
pub struct GDTEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    type_and_attributes: u8,
    limit_high_and_attributes: u8,
    base_high: u8,
}

impl GDTEntry {
    const PRESENT_MASK: u8 = 0b1000_0000;
    const DPL_MASK: u8 = 0b0110_0000;
    const DPL_SHIFT: usize = 5;
    const S_MASK: u8 = 0b0001_0000;
    const EXEC_MASK: u8 = 0b0000_1000;
    const DC_MASK: u8 = 0b0000_0100;
    const RW_MASK: u8 = 0b0000_0010;
    const ACCESSED_MASK: u8 = 0b0000_0001;
    const TYPE_MASK: u8 = 0b0000_1111;

    pub fn is_present(&self) -> bool {
        self.type_and_attributes & Self::PRESENT_MASK != 0
    }

    pub fn dpl(&self) -> u8 {
        (self.type_and_attributes & Self::DPL_MASK) >> Self::DPL_SHIFT
    }

    pub fn entry_type(&self) -> GDTEntryType {
        let attrs = self.type_and_attributes;
        let s_is_set = attrs & Self::S_MASK != 0;

        if s_is_set {
            let dc = attrs & Self::DC_MASK != 0;
            let rw = attrs & Self::RW_MASK != 0;
            let accessed = attrs & Self::ACCESSED_MASK != 0;

            if attrs & Self::EXEC_MASK != 0 {
                GDTEntryType::Code {
                    accessed,
                    read_only: rw,
                    conforming: dc,
                }
            } else {
                GDTEntryType::Data {
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
                0b0000 => GDTEntryType::Upper,
                0b0010 => GDTEntryType::LDT,
                0b1001 => GDTEntryType::AvailableTSS,
                0b1011 => GDTEntryType::BusyTSS,
                0b1100 => GDTEntryType::CallGate,
                other => GDTEntryType::InvalidSystem(other),
            }
        }
    }
}

impl core::fmt::Debug for GDTEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_present() {
            f.debug_struct("GDTEntry")
                .field("entry_type", &self.entry_type())
                .field("present", &true)
                .field("dpl", &self.dpl())
                .finish()
        } else {
            f.debug_struct("GDTEntry").field("present", &false).finish()
        }
    }
}
