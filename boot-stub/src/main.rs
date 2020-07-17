#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_efiapi)]
#![feature(never_type)]
#![feature(asm)]
#![allow(dead_code)]
extern crate alloc;
extern crate rlibc;

use uefi::prelude::*;

use core::fmt::Write;
use core::panic::PanicInfo;

mod ansi;
mod arch;

mod loader;
use loader::*;

use arch::x86_64::interrupts::*;
use arch::x86_64::paging::*;
use arch::x86_64::serial;

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> ! {
    let mut com1 = unsafe { serial::SerialPort::new(serial::SerialPortDescriptor::StandardCom1) };

    writeln!(
        com1,
        "{}Hello from osc-os!{}",
        ansi::Color::from_fg_and_bg(ansi::StandardColor::Black, ansi::StandardColor::White),
        ansi::Reset
    )
    .unwrap();

    let cr3_value = arch::x86_64::registers::CR3Value::read();

    writeln!(
        com1,
        "PML4 Location according to CR3 (with flags {:#X}): {:?}",
        cr3_value.flags_or_pcid(),
        cr3_value.pml4_address()
    )
    .unwrap();

    let pt_ptr = cr3_value.pml4_address().to_raw() as *const PageTable;
    let pt_ref = unsafe { &*pt_ptr };

    for index in 0..16 {
        let entry = &pt_ref[index];
        writeln!(com1, "PT entry {} is {:?}", index, entry).unwrap();
    }

    let idtr_value = arch::x86_64::interrupts::IDTRValue::read();

    writeln!(com1, "IDTR: {:?}", idtr_value).unwrap();

    let idtr_ptr = idtr_value.address().to_raw() as *const IDTEntry;
    let idtr_ref = unsafe { &*idtr_ptr };
    let entry_count = (usize::from(idtr_value.limit()) + 1) / core::mem::size_of::<IDTEntry>();

    writeln!(com1, "IDT entry is {:?}", idtr_ref).unwrap();

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
