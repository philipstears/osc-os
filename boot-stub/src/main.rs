#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_efiapi)]
#![feature(never_type)]
#![feature(asm)]
extern crate alloc;
extern crate rlibc;

use uefi::prelude::*;

use core::fmt::Write;
use core::panic::PanicInfo;

mod ansi;
mod arch;

mod loader;
use loader::*;

use arch::x86_64::serial;

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> ! {
    let mut com1 = unsafe { serial::SerialPort::new(serial::SerialPortDescriptor::StandardCom1) };
    writeln!(
        com1,
        "{}Hello from osc-os!{}",
        ansi::Color::from_fg_and_bg(ansi::StandardColor::Black, ansi::StandardColor::White),
        ansi::Reset
    );
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
