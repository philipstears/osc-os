use super::paging::PhysicalAddress;

#[repr(packed)]
#[derive(Debug)]
pub struct IDTRValue {
    limit: u16,
    address: PhysicalAddress,
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

    /// Gets the physical address.
    pub fn address(&self) -> PhysicalAddress {
        self.address
    }

    /// Gets the limit.
    pub fn limit(&self) -> u16 {
        self.limit
    }
}
