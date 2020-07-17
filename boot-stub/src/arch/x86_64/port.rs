/// The IO address space on x86 is 16-bits wide.
type PortNumber = u16;

/// Represents the width of a port (8-bit, 16-bit, or 32-bit). This
/// is a sealed trait with a fixed set of implementations for
/// u8, u16, and u32.
pub trait PortWidth: private::PortWidthInternal {}

/// Marks u8 as a valid port width.
impl PortWidth for u8 {}

/// Provides a type-safe wrapper around a port.
#[repr(transparent)]
pub struct Port<T: PortWidth> {
    port_number: PortNumber,
    _phantom: core::marker::PhantomData<T>,
}

impl<T: PortWidth> Port<T> {
    /// Constructs a new port. This is unsafe because it
    /// allows arbitrary access to the IO address space.
    pub unsafe fn new(port_number: PortNumber) -> Self {
        Self {
            port_number,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Reads a value from the port.
    pub fn read(&self) -> T {
        T::read(self.port_number)
    }

    /// Writes the specified value to the port.
    pub fn write(&self, value: T) {
        T::write(self.port_number, value)
    }
}

mod private {
    use super::*;

    pub trait PortWidthInternal {
        fn read(port_number: PortNumber) -> Self;
        fn write(port_number: PortNumber, value: Self);
    }

    impl PortWidthInternal for u8 {
        fn read(port_number: PortNumber) -> Self {
            let result;

            unsafe {
                asm!(
                    "in al, dx",
                    in("dx") port_number,
                    out("al") result,
                )
            }

            result
        }

        fn write(port_number: PortNumber, value: Self) {
            unsafe {
                asm!(
                    "out dx, al",
                    in("dx") port_number,
                    in("al") value,
                );
            }
        }
    }
}
