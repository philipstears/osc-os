use bitflags::bitflags;

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
            let first_entry = self.base.add(self.header_64.program_header_table_position as usize)
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
#[repr(C)]
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

    pub fn elf_version(&self) -> ElfVersion {
        // TODO: endianness
        self.elf_version.into()
    }
}

impl core::fmt::Debug for FileHeaderCommon {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_magic_valid() {
            f.debug_struct("FileHeaderCommon")
                .field("architecture", &self.architecture())
                .field("endianness", &self.endianness())
                .field("header_version", &self.header_version)
                .field("abi", &self.abi())
                .field("binary_type", &self.binary_type())
                .field("instruction_set", &self.instruction_set())
                .field("elf_version", &self.elf_version())
                .finish()
        } else {
            f.debug_struct("FileHeaderCommon").field("is_magic_valid", &false).finish()
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
    None,
    Relocatable,
    Executable,
    Shared,
    Core,
    ProcessorSpecific(u16),
    Invalid(u16),
}

impl From<u16> for BinaryType {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Relocatable,
            2 => Self::Executable,
            3 => Self::Shared,
            4 => Self::Core,
            n @ 0xff00..=0xffff => Self::ProcessorSpecific(n),
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

#[derive(Debug)]
pub enum ElfVersion {
    Original,
    Other(u32),
}

impl From<u32> for ElfVersion {
    fn from(value: u32) -> Self {
        match value {
            0x01 => Self::Original,
            n => Self::Other(n),
        }
    }
}

#[repr(C)]
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
        f.debug_struct("Header64")
            .field("program_entry", &format_args!("{:016X}", self.program_entry))
            .field(
                "program_header_table_position",
                &format_args!("{:016X}", self.program_header_table_position),
            )
            .field(
                "section_header_table_position",
                &format_args!("{:016X}", self.section_header_table_position),
            )
            .field("flags", &self.flags)
            .field("header_size", &self.header_size)
            .field("program_header_entry_size", &self.program_header_entry_size)
            .field("program_header_entry_count", &self.program_header_entry_count)
            .field("section_header_entry_size", &self.section_header_entry_size)
            .field("section_header_entry_count", &self.section_header_entry_count)
            .field("section_name_entry_index", &self.section_name_entry_index)
            .finish()
    }
}

#[repr(C)]
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

impl ProgramHeader64 {
    const STANDARD_FLAGS_MASK: u32 = 0b0111;

    pub fn segment_type(&self) -> SegmentType {
        self.segment_type.into()
    }

    pub fn standard_flags(&self) -> StandardSegmentFlags {
        unsafe { StandardSegmentFlags::from_bits_unchecked(self.flags & Self::STANDARD_FLAGS_MASK) }
    }

    pub fn other_flags(&self) -> u32 {
        self.flags & !Self::STANDARD_FLAGS_MASK
    }
}

impl core::fmt::Debug for ProgramHeader64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProgramHeader64")
            .field("segment_type", &self.segment_type())
            .field("standard_flags", &self.standard_flags())
            .field("other_flags", &self.other_flags())
            .field("offset", &format_args!("{:016X}", self.offset))
            .field("vma", &format_args!("{:016X}", self.vma))
            .field("lma", &format_args!("{:016X}", self.lma))
            .field("size_in_file", &self.size_in_file)
            .field("size_in_memory", &self.size_in_memory)
            .field("align", &self.align)
            .finish()
    }
}

/// Value 	Name 	Meaning
/// 0x00000000 	PT_NULL 	Program header table entry unused
/// 0x00000001 	PT_LOAD 	Loadable segment
/// 0x00000002 	PT_DYNAMIC 	Dynamic linking information
/// 0x00000003 	PT_INTERP 	Interpreter information
/// 0x00000004 	PT_NOTE 	Auxiliary information
/// 0x00000005 	PT_SHLIB 	reserved
/// 0x00000006 	PT_PHDR 	segment containing program header table itself
/// 0x00000007 	PT_TLS 	    Thread-Local Storage template
#[derive(Debug)]
pub enum SegmentType {
    Null,
    Load,
    Dynamic,
    Interpreter,
    Note,
    Shlib,
    ProgramHeaderTable,
    ThreadLocalStorage,
    GNUEHFrame,
    GNUStack,
    OperatingSystemSpecific(u32),
    ProcessorSpecific(u32),
    Other(u32),
}

impl From<u32> for SegmentType {
    fn from(value: u32) -> Self {
        match value {
            0x00 => Self::Null,
            0x01 => Self::Load,
            0x02 => Self::Dynamic,
            0x03 => Self::Interpreter,
            0x04 => Self::Note,
            0x05 => Self::Shlib,
            0x06 => Self::ProgramHeaderTable,
            0x6474e550 => Self::GNUEHFrame,
            0x6474e551 => Self::GNUStack,
            n @ 0x60000000..=0x6FFFFFFF => Self::OperatingSystemSpecific(n),
            n @ 0x70000000..=0x7FFFFFFF => Self::ProcessorSpecific(n),
            n => Self::Other(n),
        }
    }
}

bitflags! {
    pub struct StandardSegmentFlags: u32 {
        const EXECUTE = 0b0001;
        const WRITE = 0b0010;
        const READ = 0b0100;
    }
}
