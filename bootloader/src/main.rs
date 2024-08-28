#![no_std]
#![no_main]

mod compiler_builtins;
mod asm_ffi;
mod mmu;
mod pxe;
mod error;

use core::panic::PanicInfo;
use cpu::x86;
use serial::println;
use parse_pe::Pe;

extern crate alloc;

#[no_mangle]
extern "C" fn entry(_bootloader_start: u32, _bootloader_end: u32, _stack_addr: u32) {
    serial::init();
    mmu::init();

    let kernel = pxe::download(b"pizza.kernel").expect("Failed to download kernel");
    let kernel = Pe::parse(&kernel).expect("Failed to parse kernel PE");

    kernel.access_sections(|base, size, _bytes| {
        println!("Base {:x}, size {:x}", base, size);
        Some(())
    }).expect("Failed to acess sections");

    x86::halt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Print the location where the panic occurred
    if let Some(loc) = info.location() {
        println!("System panic: {}:{}", loc.file(), loc.line());
    } else {
        println!("System panic: unknown location");
    }
    // Print the message for the panic
    println!("{:?}", info.message());
    x86::halt()
}
