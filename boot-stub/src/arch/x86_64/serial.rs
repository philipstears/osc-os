//! Provides serial port capabilities.

use super::port::{Port, PortAddress};

/// The standard base IO address of the first COM port. This is
/// generally reliable.
pub const COM1_BASE_ADDRESS: PortAddress = PortAddress::from_raw(0x3F8);

/// The standard base IO address of the second COM port. This is
/// generally reliable.
pub const COM2_BASE_ADDRESS: PortAddress = PortAddress::from_raw(0x2F8);

/// The standard base IO address of the third COM port. Note that this
/// is less reliably the case than for the first two COM ports.
pub const COM3_BASE_ADDRESS: PortAddress = PortAddress::from_raw(0x3E8);

/// The standard base IO address of the fourth COM port. Note that this
/// is less reliably the case than for the first two COM ports.
pub const COM4_BASE_ADDRESS: PortAddress = PortAddress::from_raw(0x2E8);

/// Indicates the serial port to be opened.
#[derive(Debug)]
pub enum SerialPortDescriptor {
    /// The first COM port in its standard location in the IO address space. This is
    /// generally reliable.
    StandardCom1,

    /// The second COM port in its standard location in the IO address space. This is
    /// generally reliable.
    StandardCom2,

    /// The third COM port in its standard location in the IO address space. Note that
    /// COM3 is less reliably in the standard location than the first two ports.
    StandardCom3,

    /// The fourth COM port in its standard location in the IO address space. Note that
    /// COM4 is less reliably in the standard location than the first two ports.
    StandardCom4,

    /// A COM port at a specific base address
    Custom { base_address: PortAddress },
}

impl SerialPortDescriptor {
    fn to_base_address(&self) -> PortAddress {
        match self {
            Self::StandardCom1 => COM1_BASE_ADDRESS,
            Self::StandardCom2 => COM2_BASE_ADDRESS,
            Self::StandardCom3 => COM3_BASE_ADDRESS,
            Self::StandardCom4 => COM4_BASE_ADDRESS,
            Self::Custom { base_address } => *base_address,
        }
    }
}

/// Provides access to a serial port.
pub struct SerialPort {
    data_port: Port<u8>,
}

impl SerialPort {
    /// Constructs a new serial port.
    ///
    /// # Safety
    /// This is unsafe because it can construct a serial port from
    /// an arbitrary IO address.
    pub unsafe fn new(descriptor: SerialPortDescriptor) -> Self {
        Self {
            data_port: Port::<u8>::new(descriptor.to_base_address()),
        }
    }

    /// Writes the given slice of bytes to the serial port.
    pub fn write(&self, bytes: &[u8]) {
        for b in bytes {
            self.data_port.write(*b);
        }
    }

    /// Writes the given byte to the serial port.
    pub fn write_byte(&self, byte: u8) {
        self.data_port.write(byte);
    }
}
