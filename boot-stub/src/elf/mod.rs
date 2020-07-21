#[derive(Debug)]
pub struct ELF64<'a> {
    base: *const u8,
    len: usize,

    header_common: &'a FileHeaderCommon,
    header_64: &'a FileHeader64,
}

impl<'a> ELF64<'a> {
    // TODO: fallibility
    pub fn open(file: &'a [u8]) -> Self {
        unsafe {
            //
            // TODO: validation
            //
            let base = file.as_ptr();
            let len = file.len();

            let header_common = base as *const FileHeaderCommon;
            let header_64 =
                base.add(core::mem::size_of::<FileHeaderCommon>()) as *const FileHeader64;

            Self {
                base,
                len,
                header_common: &*header_common,
                header_64: &*header_64,
            }
        }
    }

    pub fn program_entries(&self) -> &[ProgramHeader64] {
        unsafe {
            let first_entry = self
                .base
                .add(self.header_64.program_header_table_position as usize)
                as *const ProgramHeader64;

            core::slice::from_raw_parts(
                first_entry,
                self.header_64.program_header_entry_count as usize,
            )
        }
    }
}

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
#[repr(packed)]
pub struct FileHeaderCommon {
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

impl FileHeaderCommon {
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

impl core::fmt::Debug for FileHeaderCommon {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_magic_valid() {
            // We copy values on to the stack because the formatter wants
            // borrowed values, but our fields are packed, which in theory
            // means when the references are dereferenced, it could trigger
            // an unaligned access exception
            let header_version = self.header_version;
            let elf_version = self.elf_version;

            f.debug_struct("FileHeaderCommon")
                .field("architecture", &self.architecture())
                .field("endianness", &self.endianness())
                .field("header_version", &header_version)
                .field("abi", &self.abi())
                .field("binary_type", &self.binary_type())
                .field("instruction_set", &self.instruction_set())
                .field("elf_version", &elf_version)
                .finish()
        } else {
            f.debug_struct("FileHeaderCommon")
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

#[repr(packed)]
pub struct FileHeader64 {
    pub program_entry: u64,
    pub program_header_table_position: u64,
    pub section_header_table_position: u64,
    pub flags: u32,
    pub header_size: u16,
    pub program_header_entry_size: u16,
    pub program_header_entry_count: u16,
    pub section_header_entry_size: u16,
    pub section_header_entry_count: u16,
    pub section_name_entry_index: u16,
}

impl core::fmt::Debug for FileHeader64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // We copy values on to the stack because the formatter wants
        // borrowed values, but our fields are packed, which in theory
        // means when the references are dereferenced, it could trigger
        // an unaligned access exception
        let program_entry = self.program_entry;
        let program_header_table_position = self.program_header_table_position;
        let section_header_table_position = self.section_header_table_position;
        let flags = self.flags;
        let header_size = self.header_size;
        let program_header_entry_size = self.program_header_entry_size;
        let program_header_entry_count = self.program_header_entry_count;
        let section_header_entry_size = self.section_header_entry_size;
        let section_header_entry_count = self.section_header_entry_count;
        let section_name_entry_index = self.section_name_entry_index;

        f.debug_struct("Header64")
            .field("program_entry", &format_args!("{:016X}", program_entry))
            .field(
                "program_header_table_position",
                &format_args!("{:016X}", program_header_table_position),
            )
            .field(
                "section_header_table_position",
                &format_args!("{:016X}", section_header_table_position),
            )
            .field("flags", &flags)
            .field("header_size", &header_size)
            .field("program_header_entry_size", &program_header_entry_size)
            .field("program_header_entry_count", &program_header_entry_count)
            .field("section_header_entry_size", &section_header_entry_size)
            .field("section_header_entry_count", &section_header_entry_count)
            .field("section_name_entry_index", &section_name_entry_index)
            .finish()
    }
}

#[repr(packed)]
pub struct ProgramHeader64 {
    pub segment_type: u32,
    pub flags: u32,
    pub offset: u64,
    pub vma: u64,
    pub lma: u64,
    pub size_in_file: u64,
    pub size_in_memory: u64,
    pub align: u64,
}

impl core::fmt::Debug for ProgramHeader64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // We copy values on to the stack because the formatter wants
        // borrowed values, but our fields are packed, which in theory
        // means when the references are dereferenced, it could trigger
        // an unaligned access exception
        let segment_type = self.segment_type;
        let flags = self.flags;
        let offset = self.offset;
        let vma = self.vma;
        let lma = self.lma;
        let size_in_file = self.size_in_file;
        let size_in_memory = self.size_in_memory;
        let align = self.align;

        f.debug_struct("ProgramHeader64")
            .field("segment_type", &segment_type)
            .field("flags", &flags)
            .field("offset", &format_args!("{:016X}", offset))
            .field("vma", &format_args!("{:016X}", vma))
            .field("lma", &format_args!("{:016X}", lma))
            .field("size_in_file", &size_in_file)
            .field("size_in_memory", &size_in_memory)
            .field("align", &align)
            .finish()
    }
}
