use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::proto::loaded_image::*;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::*;
use uefi::CStr16;

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
        // Get the estimated map size
        let map_size = self.system_table.boot_services().memory_map_size();
        let _map_dest = vec![0u8; map_size << 2];

        print_string(
            &self.system_table,
            format!("Mem Map Size: {}\r\n", map_size),
        );

        let kernel_elf = ELF64::open(self.phase_data.kernel.loaded_image.as_ref());

        print_string(&self.system_table, format!("ELF64: {:?}\r\n", kernel_elf));

        for (index, entry) in kernel_elf.program_entries().iter().enumerate() {
            print_string(&self.system_table, format!("PE {}: {:?}\r\n", index, entry));
        }

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
                    phase_data: Ready { kernel },
                };

                ready.transfer_to_kernel();
            }

            Err(error) => {
                match error {
                    BootError::RetrieveImageInfoFailed(Status(status_code)) => {
                        self.print_string(format!(
                            "Failed to get boot image information ({:#x})\r\n",
                            status_code
                        ))
                    }

                    BootError::RetrieveSimpleFileSystemFailed(Status(status_code)) => self
                        .print_string(format!(
                            "Failed to get access to boot file system ({:#x})\r\n",
                            status_code
                        )),

                    BootError::RetrieveVolumeFailed(Status(status_code)) => {
                        self.print_string(format!(
                            "Failed to get access to boot volume ({:#x})\r\n",
                            status_code
                        ))
                    }

                    BootError::OpenKernelFailed(Status(status_code)) => self.print_string(format!(
                        "Failed to get open the kernel file for reading ({:#x})\r\n",
                        status_code
                    )),

                    BootError::StatKernelFailed(Status(status_code)) => self.print_string(format!(
                        "Failed to get read information about the kernel file ({:#x})\r\n",
                        status_code
                    )),

                    BootError::ReadKernelFailed(Status(status_code)) => self.print_string(format!(
                        "Failed to read the kernel file into memory ({:#x})\r\n",
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

        let kernel = PreparedKernel { loaded_image: data };

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
