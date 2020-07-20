// Position (32 bit)     Position (64 bit)     Value
// 0-3                   0-3                   Magic number - 0x7F, then 'ELF' in ASCII
// 4                     4                     1 = 32 bit, 2 = 64 bit
// 5                     5                     1 = little endian, 2 = big endian
// 6                     6                     ELF header version
// 7                     7                     OS ABI - usually 0 for System V
// 8-15                  8-15                  Unused/padding
// 16-17                 16-17                 1 = relocatable, 2 = executable, 3 = shared, 4 = core
// 18-19                 18-19                 Instruction set - see table below
// 20-23                 20-23                 ELF Version
//
// 24-27                 24-31                 Program entry position
// 28-31                 32-39                 Program header table position
// 32-35                 40-47                 Section header table position
// 36-39                 48-51                 Flags - architecture dependent; see note below
// 40-41                 52-53                 Header size
// 42-43                 54-55                 Size of an entry in the program header table
// 44-45                 56-57                 Number of entries in the program header table
// 46-47                 58-59                 Size of an entry in the section header table
// 48-49                 60-61                 Number of entries in the section header table
// 50-51                 62-63                 Index in section header table with the section names
#[repr(C)]
pub struct CommonHeader {
    pub magic: [u8; 4],
    pub architecture: u8,
    pub endianness: u8,
    pub header_version: u8,
    pub abi: u8,
    pub unused: [u8; 8],
    pub binary_type: u16,
    pub instruction_set: u16,
    pub elf_version: u32,
}

impl CommonHeader {
    pub fn is_magic_valid(&self) -> bool {
        self.magic.as_ref() == b"\x7FELF"
    }

    pub fn architecture(&self) -> Architecture {
        self.architecture.into()
    }

    pub fn endianness(&self) -> Endianness {
        self.endianness.into()
    }

    pub fn abi(&self) -> ABI {
        self.abi.into()
    }

    pub fn binary_type(&self) -> BinaryType {
        // TODO: endianness
        self.binary_type.into()
    }

    pub fn instruction_set(&self) -> InstructionSet {
        // TODO: endianness
        self.instruction_set.into()
    }
}

impl core::fmt::Debug for CommonHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_magic_valid() {
            f.debug_struct("CommonHeader")
                .field("architecture", &self.architecture())
                .field("endianness", &self.endianness())
                .field("header_version", &self.header_version)
                .field("abi", &self.abi())
                .field("binary_type", &self.binary_type())
                .field("instruction_set", &self.instruction_set())
                .field("elf_version", &self.elf_version)
                .finish()
        } else {
            f.debug_struct("CommonHeader")
                .field("is_magic_valid", &false)
                .finish()
        }
    }
}

#[derive(Debug)]
pub enum Architecture {
    Bits32,
    Bits64,
    Invalid(u8),
}

impl From<u8> for Architecture {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Bits32,
            2 => Self::Bits64,
            n => Self::Invalid(n),
        }
    }
}

#[derive(Debug)]
pub enum Endianness {
    Little,
    Big,
    Invalid(u8),
}

impl From<u8> for Endianness {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Little,
            2 => Self::Big,
            n => Self::Invalid(n),
        }
    }
}

#[derive(Debug)]
pub enum ABI {
    SystemV,
    Invalid(u8),
}

impl From<u8> for ABI {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::SystemV,
            n => Self::Invalid(n),
        }
    }
}

#[derive(Debug)]
pub enum BinaryType {
    Relocatable,
    Executable,
    Shared,
    Core,
    Invalid(u16),
}

impl From<u16> for BinaryType {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::Relocatable,
            2 => Self::Executable,
            3 => Self::Shared,
            4 => Self::Core,
            n => Self::Invalid(n),
        }
    }
}

#[derive(Debug)]
pub enum InstructionSet {
    X86,
    ARM,
    AMD64,
    ARM64,
    RISCV,
    Other(u16),
}

impl From<u16> for InstructionSet {
    fn from(value: u16) -> Self {
        match value {
            0x03 => Self::X86,
            0x28 => Self::ARM,
            0x3E => Self::AMD64,
            0xB7 => Self::ARM64,
            0xF3 => Self::RISCV,
            n => Self::Other(n),
        }
    }
}
