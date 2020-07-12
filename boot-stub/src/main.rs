#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_efiapi)]
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

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, st: SystemTable<Boot>) -> ! {
    print_formatted_string(&st, "UEFI Boot Stub Entered\r\n");

    // Get the estimated map size
    let map_size = st.boot_services().memory_map_size();

    unsafe {
        uefi::alloc::init(st.boot_services());
    }

    let image_info_cell = st
        .boot_services()
        .handle_protocol::<LoadedImage>(image_handle)
        .unwrap_success();

    let image_info = unsafe { &mut *image_info_cell.get() };

    let sfs_cell = st
        .boot_services()
        .handle_protocol::<SimpleFileSystem>(image_info.device_handle())
        .unwrap_success();

    let sfs = unsafe { &mut *sfs_cell.get() };
    let mut dir = sfs.open_volume().unwrap_success();

    print_formatted_string(&st, "Reading file\r\n");

    let filename = "OSCOS\\KERNEL.BIN";
    let mut file = unsafe {
        RegularFile::new(
            dir.open(filename, FileMode::Read, FileAttribute::empty())
                .unwrap_success(),
        )
    };

    print_formatted_string(&st, "Read file\r\n");

    let mut info_buffer = vec![0u8; 4096];
    let file_info = file
        .get_info::<FileInfo>(info_buffer.as_mut())
        .unwrap_success();
    let file_size = file_info.file_size();

    let mut data = vec![0u8; file_size as usize];
    //file.set_position(file_size - (data.len() as u64));
    file.read(&mut data);

    let it = unsafe { core::str::from_utf8_unchecked(&data) };
    print_formatted_string(&st, &format!("Suffix: {} ", it));

    let image_base = image_info.image_base();
    let image_base_ptr = image_base as *const u8;

    let image_size = image_info.image_size() as usize;

    let image = unsafe { core::slice::from_raw_parts(image_base_ptr, image_size) };
    let mut found = false;

    let string = format!(
        "Memory Map Size: {}\r\nSig: {:#x} {:#x}\r\n",
        map_size, image[0], image[1]
    );

    print_formatted_string(&st, &string);

    loop {}
}

fn print_formatted_string(st: &SystemTable<Boot>, string: &str) {
    let string_bytes = string.as_bytes();
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
