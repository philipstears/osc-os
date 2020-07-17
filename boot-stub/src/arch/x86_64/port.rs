//! Provides access to the IO address space of an x86-64 system.

/// Represents an address in the IO address space.
#[derive(Debug, Copy, Clone)]
pub struct PortAddress(u16);

impl PortAddress {
    /// Constructs a new port number from its raw 16-bit address.
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// Gets the raw 16-bit address.
    pub fn as_raw(&self) -> u16 {
        self.0
    }
}

/// Represents the width of a port (8-bit, 16-bit, or 32-bit). This
/// is a sealed trait with a fixed set of implementations for
/// u8, u16, and u32.
pub trait PortWidth: private::PortWidthInternal {}

/// Marks u8 as a valid port width.
impl PortWidth for u8 {}

/// Marks u16 as a valid port width.
impl PortWidth for u16 {}

/// Marks u32 as a valid port width.
impl PortWidth for u32 {}

/// Provides a type-safe wrapper around a port.
#[repr(transparent)]
#[derive(Debug)]
pub struct Port<T: PortWidth> {
    port_address: PortAddress,
    _phantom: core::marker::PhantomData<T>,
}

impl<T: PortWidth> Port<T> {
    /// Constructs a new port.
    ///
    /// # Safety
    /// This is unsafe because it allows arbitrary access to the IO address
    /// space.
    pub unsafe fn new(port_address: PortAddress) -> Self {
        Self {
            port_address,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Reads a value from the port.
    pub fn read(&self) -> T {
        T::read(self.port_address)
    }

    /// Writes the specified value to the port.
    pub fn write(&self, value: T) {
        T::write(self.port_address, value)
    }
}

mod private {
    use super::*;

    pub trait PortWidthInternal {
        fn read(port_address: PortAddress) -> Self;
        fn write(port_address: PortAddress, value: Self);
    }

    impl PortWidthInternal for u8 {
        fn read(port_address: PortAddress) -> Self {
            let result;

            unsafe {
                asm!(
                    "in al, dx",
                    in("dx") port_address.as_raw(),
                    out("al") result,
                )
            }

            result
        }

        fn write(port_address: PortAddress, value: Self) {
            unsafe {
                asm!(
                    "out dx, al",
                    in("dx") port_address.as_raw(),
                    in("al") value,
                );
            }
        }
    }

    impl PortWidthInternal for u16 {
        fn read(port_address: PortAddress) -> Self {
            let result;

            unsafe {
                asm!(
                    "in ax, dx",
                    in("dx") port_address.as_raw(),
                    out("ax") result,
                )
            }

            result
        }

        fn write(port_address: PortAddress, value: Self) {
            unsafe {
                asm!(
                    "out dx, ax",
                    in("dx") port_address.as_raw(),
                    in("ax") value,
                );
            }
        }
    }

    impl PortWidthInternal for u32 {
        fn read(port_address: PortAddress) -> Self {
            let result;

            unsafe {
                asm!(
                    "in eax, dx",
                    in("dx") port_address.as_raw(),
                    out("eax") result,
                )
            }

            result
        }

        fn write(port_address: PortAddress, value: Self) {
            unsafe {
                asm!(
                    "out dx, eax",
                    in("dx") port_address.as_raw(),
                    in("eax") value,
                );
            }
        }
    }
}
