#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_efiapi)]
#![feature(never_type)]
#![feature(asm)]
extern crate alloc;
extern crate rlibc;

use uefi::prelude::*;

use core::panic::PanicInfo;

mod arch;

mod loader;
use loader::*;

use arch::x86_64::serial;

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> ! {
    let com1 = unsafe { serial::SerialPort::new(serial::SerialPortDescriptor::StandardCom1) };
    com1.write(b"Hello from osc-os!\r\n");
    Loader::new(image_handle, system_table).run();
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    loop {}
}
