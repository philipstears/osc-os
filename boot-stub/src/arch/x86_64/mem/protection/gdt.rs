use super::SegmentDescriptorTableRef;

/// Provides access to the GDTR (Global Descriptor Table Register).
///
/// For more details about the structure of the GDT see Intel 3A - 3.4.5
pub enum GDTR {}

impl GDTR {
    /// Reads the current value of the GDT register.
    pub fn read() -> SegmentDescriptorTableRef {
        unsafe {
            let mut result: SegmentDescriptorTableRef =
                core::mem::MaybeUninit::uninit().assume_init();

            let result_ptr = &mut result as *mut SegmentDescriptorTableRef;

            asm!("sgdt [{0}]", in(reg) result_ptr);

            result
        }
    }
}
