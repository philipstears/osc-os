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
#[derive(Debug)]
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
