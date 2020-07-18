use super::paging::LinearAddress;

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
