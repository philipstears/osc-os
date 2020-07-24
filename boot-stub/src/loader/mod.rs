use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::proto::loaded_image::*;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::*;
use uefi::table::boot::*;
use uefi::CStr16;

use crate::elf::SegmentType;
use crate::elf::ELF64;

const KERNEL_LOCATION: &'static str = "OSCOS\\KERNEL.BIN";

enum BootError {
    RetrieveImageInfoFailed(Status),
    RetrieveSimpleFileSystemFailed(Status),
    RetrieveVolumeFailed(Status),
    OpenKernelFailed(Status),
    StatKernelFailed(Status),
    ReadKernelFailed(Status),
}

struct PreparedKernel {
    loaded_image: Vec<u8>,
}

pub struct Prepare;

pub struct Ready {
    kernel: PreparedKernel,
}

pub struct Loader<Phase> {
    image_handle: Handle,
    system_table: SystemTable<Boot>,
    phase_data: Phase,
}

impl Loader<Ready> {
    fn transfer_to_kernel(self) -> ! {
        let kernel_elf = ELF64::open(self.phase_data.kernel.loaded_image.as_ref());

        print_string(&self.system_table, format!("Kernel: {:?}\r\n", kernel_elf));

        let mut load_entries =
            kernel_elf.program_entries().iter().filter(|pe| pe.segment_type() == SegmentType::Load);

        print_string(
            &self.system_table,
            format!(
                "{:10}\t{:16}\t{:16}\t{:10}\t{:10}\t{:<10}\t{:16}\r\n",
                "Idx",
                "VMA",
                "VMA Aligned",
                "VMA Offset",
                "Page Count",
                "Zero Bytes",
                "Physical Address"
            ),
        );

        let bs = self.system_table.boot_services();
        let mut required_mappings = Vec::with_capacity(4);

        for (index, entry) in load_entries.enumerate() {
            let vma = entry.vma as usize;
            let vma_offset = vma & 0xFFF;
            let vma_page_aligned = vma & !0xFFF;
            let vma_page_count = (entry.size_in_memory as usize + vma_offset + 0xFFF) >> 12;
            let zero_bytes = (entry.size_in_memory - entry.size_in_file) as usize;

            let target_raw = bs
                .allocate_pages(
                    AllocateType::AnyPages,
                    MemoryType::LOADER_DATA,
                    vma_page_count as usize,
                )
                .unwrap_success();

            print_string(
                &self.system_table,
                format!(
                    "{:<10}\t{:<016x}\t{:<016x}\t{:<10}\t{:<10}\t{:<10}\t{:<016x}\r\n",
                    index,
                    vma,
                    vma_page_aligned,
                    vma_offset,
                    vma_page_count,
                    zero_bytes,
                    target_raw
                ),
            );

            let target = unsafe {
                core::slice::from_raw_parts_mut(target_raw as *mut u8, vma_page_count << 12)
            };

            let data = kernel_elf.program_entry_data(entry);

            &mut target[vma_offset..(vma_offset + entry.size_in_file as usize)]
                .copy_from_slice(data);

            let first_zero_byte = vma_offset + entry.size_in_file as usize;
            let last_zero_byte_excl = first_zero_byte + zero_bytes;

            for index in first_zero_byte..last_zero_byte_excl {
                target[index] = 0;
            }

            required_mappings.push((vma, target_raw));
        }

        // let first = load_entries.next().unwrap();
        // let first_vma = first.vma;
        // let last_vma = first_vma + first.size_in_memory;

        for (index, entry) in kernel_elf.program_entries().iter().enumerate() {
            print_string(&self.system_table, format!("PE {}: {:?}\r\n", index, entry));
        }

        // Get the estimated map size
        let map_info = self.system_table.boot_services().memory_map_params();
        let map_entry_count = map_info.map_size / map_info.entry_size;

        print_string(&self.system_table, format!("Mem Map Info: {:?}\r\n", map_info));
        print_string(&self.system_table, format!("Mem Map Entry Count: {}\r\n", map_entry_count));

        let mut map_dest = vec![0u8; map_info.map_size << 2];

        let (_mem_map_key, mem_map_iter) =
            self.system_table.boot_services().memory_map(map_dest.as_mut()).unwrap_success();

        // print_string(&self.system_table, format!("Mem Map Iter: {:?}\r\n", mem_map_iter));

        // We're ready to switch to the kernel
        let mut map_dest = vec![0u8; map_info.map_size << 2];

        let (runtime_table, mem_map_iter) = self
            .system_table
            .exit_boot_services(self.image_handle, map_dest.as_mut())
            .unwrap_success();

        loop {}
    }
}

impl Loader<Prepare> {
    pub fn new(image_handle: Handle, system_table: SystemTable<Boot>) -> Self {
        Self {
            image_handle,
            system_table,
            phase_data: Prepare,
        }
    }

    pub fn run(self) -> ! {
        self.print_string("UEFI boot stub entered.\r\n");

        // Make uefi-rs's built-in allocator atop UEFI allocation
        // available for us to use
        unsafe {
            uefi::alloc::init(self.system_table.boot_services());
        }

        match self.prepare() {
            Ok(kernel) => {
                self.print_string("Preparation succeeded, transferring to kernel.\r\n");

                let ready = Loader {
                    image_handle: self.image_handle,
                    system_table: self.system_table,
                    phase_data: Ready {
                        kernel,
                    },
                };

                ready.transfer_to_kernel();
            }

            Err(error) => {
                match error {
                    BootError::RetrieveImageInfoFailed(Status(status_code)) => self.print_string(
                        format!("Failed to get boot image information ({:x})\r\n", status_code),
                    ),

                    BootError::RetrieveSimpleFileSystemFailed(Status(status_code)) => self
                        .print_string(format!(
                            "Failed to get access to boot file system ({:x})\r\n",
                            status_code
                        )),

                    BootError::RetrieveVolumeFailed(Status(status_code)) => self.print_string(
                        format!("Failed to get access to boot volume ({:x})\r\n", status_code),
                    ),

                    BootError::OpenKernelFailed(Status(status_code)) => self.print_string(format!(
                        "Failed to get open the kernel file for reading ({:x})\r\n",
                        status_code
                    )),

                    BootError::StatKernelFailed(Status(status_code)) => self.print_string(format!(
                        "Failed to get read information about the kernel file ({:x})\r\n",
                        status_code
                    )),

                    BootError::ReadKernelFailed(Status(status_code)) => self.print_string(format!(
                        "Failed to read the kernel file into memory ({:x})\r\n",
                        status_code
                    )),
                }

                self.exit();
            }
        }
    }

    fn prepare(&self) -> Result<PreparedKernel, BootError> {
        let image_info_cell = self
            .system_table
            .boot_services()
            .handle_protocol::<LoadedImage>(self.image_handle)
            .warning_as_error()
            .map_err(|err| BootError::RetrieveImageInfoFailed(err.status()))?;

        let image_info = unsafe { &mut *image_info_cell.get() };

        let sfs_cell = self
            .system_table
            .boot_services()
            .handle_protocol::<SimpleFileSystem>(image_info.device_handle())
            .warning_as_error()
            .map_err(|err| BootError::RetrieveSimpleFileSystemFailed(err.status()))?;

        let sfs = unsafe { &mut *sfs_cell.get() };

        let mut volume = sfs
            .open_volume()
            .warning_as_error()
            .map_err(|err| BootError::RetrieveVolumeFailed(err.status()))?;

        let file = volume
            .open(KERNEL_LOCATION, FileMode::Read, FileAttribute::empty())
            .warning_as_error()
            .map_err(|err| BootError::OpenKernelFailed(err.status()))?;

        let mut file = unsafe { RegularFile::new(file) };

        let mut info_buffer = vec![0u8; 4096];

        let file_info = file
            .get_info::<FileInfo>(info_buffer.as_mut())
            .warning_as_error()
            .map_err(|err| BootError::StatKernelFailed(err.status()))?;

        let mut data = {
            let file_size = file_info.file_size();
            vec![0u8; file_size as usize]
        };

        file.read(&mut data)
            .warning_as_error()
            .map_err(|err| BootError::ReadKernelFailed(err.status()))?;

        let kernel = PreparedKernel {
            loaded_image: data,
        };

        Ok(kernel)
    }

    fn exit(self) -> ! {
        self.print_string("UEFI boot stub should exit now...\r\n");
        loop {}
    }

    fn print_string(&self, string: impl AsRef<str>) {
        print_string(&self.system_table, string);
    }
}

fn print_string(st: &SystemTable<Boot>, string: impl AsRef<str>) {
    let string_bytes = string.as_ref().as_bytes();
    let mut buf = [0u16; 64];

    for chunk in string_bytes.chunks(buf.len() - 1) {
        for i in 0..chunk.len() {
            buf[i] = chunk[i] as u16;
        }

        buf[chunk.len()] = 0;

        st.stdout()
            .output_string(unsafe { &CStr16::from_u16_with_nul_unchecked(&buf[..chunk.len()]) })
            .unwrap_success();
    }
}
