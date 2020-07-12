#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_efiapi)]
#![feature(never_type)]
extern crate alloc;
extern crate rlibc;

use self::alloc::format;
use self::alloc::vec;

use uefi::prelude::*;
use uefi::proto::loaded_image::*;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::*;
use uefi::CStr16;

use core::panic::PanicInfo;

const KERNEL_LOCATION: &'static str = "OSCOS\\KERNEL.BIN";

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> ! {
    Loader::new(image_handle, system_table).run();
}

enum BootError {
    RetrieveImageInfoFailed(Status),
}

struct Loader {
    image_handle: Handle,
    system_table: SystemTable<Boot>,
}

impl Loader {
    fn new(image_handle: Handle, system_table: SystemTable<Boot>) -> Self {
        Self {
            image_handle,
            system_table,
        }
    }

    fn run(self) -> ! {
        self.print_string("UEFI boot stub entered.\r\n");

        // Make uefi-rs's built-in allocator atop UEFI allocation
        // available for us to use
        unsafe {
            uefi::alloc::init(self.system_table.boot_services());
        }

        self.boot();
    }

    fn boot(self) -> ! {
        let image_info_cell = match self
            .system_table
            .boot_services()
            .handle_protocol::<LoadedImage>(self.image_handle)
            .warning_as_error()
        {
            Ok(result) => result,
            Err(err) => self.exit_with_error(BootError::RetrieveImageInfoFailed(err.status())),
        };

        let image_info = unsafe { &mut *image_info_cell.get() };

        let sfs_cell = self
            .system_table
            .boot_services()
            .handle_protocol::<SimpleFileSystem>(image_info.device_handle())
            .unwrap_success();

        let sfs = unsafe { &mut *sfs_cell.get() };
        let mut dir = sfs.open_volume().unwrap_success();

        self.print_string("Reading file\r\n");

        let filename = KERNEL_LOCATION;
        let mut file = unsafe {
            RegularFile::new(
                dir.open(filename, FileMode::Read, FileAttribute::empty())
                    .unwrap_success(),
            )
        };

        self.print_string("Read file\r\n");

        let mut info_buffer = vec![0u8; 4096];
        let file_info = file
            .get_info::<FileInfo>(info_buffer.as_mut())
            .unwrap_success();
        let file_size = file_info.file_size();

        let mut data = vec![0u8; file_size as usize];
        //file.set_position(file_size - (data.len() as u64));
        file.read(&mut data);

        let it = unsafe { core::str::from_utf8_unchecked(&data) };
        self.print_string(format!("Suffix: {} ", it));

        let image_base = image_info.image_base();
        let image_base_ptr = image_base as *const u8;

        let image_size = image_info.image_size() as usize;

        let image = unsafe { core::slice::from_raw_parts(image_base_ptr, image_size) };
        let mut found = false;

        // Get the estimated map size
        let map_size = self.system_table.boot_services().memory_map_size();

        let string = format!(
            "Memory Map Size: {}\r\nSig: {:#x} {:#x}\r\n",
            map_size, image[0], image[1]
        );

        self.print_string(string);

        // We should never get here, the kernel should never return, there's no
        // safe way to handle it if it does
        loop {}
    }

    fn exit_with_error(self, error: BootError) -> ! {
        match error {
            BootError::RetrieveImageInfoFailed(Status(status_code)) => self.print_string(format!(
                "Failed to get boot image information ({:#x})",
                status_code
            )),
        }

        self.exit();
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
    let mut buf = [0u16; 128];

    for i in 0..string_bytes.len() {
        buf[i] = string_bytes[i] as u16;
    }

    st.stdout()
        .output_string(unsafe { &CStr16::from_u16_with_nul_unchecked(&buf) })
        .unwrap_success();
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    loop {}
}
