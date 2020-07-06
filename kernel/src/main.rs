#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_efiapi)]
extern crate alloc;
extern crate rlibc;

use self::alloc::format;

use uefi::prelude::*;
use uefi::CStr16;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "efiapi" fn efi_main(_image: Handle, st: SystemTable<Boot>) -> ! {
    // Get the estimated map size
    let map_size = st.boot_services().memory_map_size();

    unsafe {
        uefi::alloc::init(st.boot_services());
    }

    let string = format!("Memory Map Size: {}", map_size);
    let string_bytes = string.as_bytes();
    let mut buf = [0u16; 128];

    for i in 0..string_bytes.len() {
        buf[i] = string_bytes[i] as u16;
    }

    st.stdout().reset(false).unwrap_success();

    st.stdout()
        .output_string(unsafe { &CStr16::from_u16_with_nul_unchecked(&buf) })
        .unwrap_success();

    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    loop {}
}
