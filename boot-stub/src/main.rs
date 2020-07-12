#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_efiapi)]
extern crate alloc;
extern crate rlibc;

use self::alloc::format;

use uefi::prelude::*;
use uefi::proto::loaded_image::*;
use uefi::CStr16;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, st: SystemTable<Boot>) -> ! {
    // Get the estimated map size
    let map_size = st.boot_services().memory_map_size();

    unsafe {
        uefi::alloc::init(st.boot_services());
    }

    st.stdout().reset(false).unwrap_success();

    let image_info_cell = st
        .boot_services()
        .handle_protocol::<LoadedImage>(image_handle)
        .unwrap_success();

    let image_info = unsafe { &mut *image_info_cell.get() };

    let image_base = image_info.image_base();
    let image_base_ptr = image_base as *const u8;

    let image_size = image_info.image_size() as usize;

    let image = unsafe { core::slice::from_raw_parts(image_base_ptr, image_size) };
    let mut found = false;

    for search_index in 0..(image.len() - 3) {
        if image[search_index] == b'H' && image[search_index + 1] == b'e' {
            let it = unsafe {
                core::str::from_utf8_unchecked(&image[search_index..(search_index + 10)])
            };
            print_formatted_string(&st, &format!("Found it? {}\r\n", it));
            found = true;
            break;
        }
    }

    if !found {
        print_formatted_string(&st, "Failed\r\n");
    }

    let data_start = 29184usize;
    let data_length = 12usize;

    let data = &image[data_start..(data_start + data_length)];
    let data_str = unsafe { core::str::from_utf8_unchecked(data) };

    let string = format!(
        "Memory Map Size: {}\r\nSig: {:#x} {:#x}\r\nData: {}",
        map_size, image[0], image[1], data_str,
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
