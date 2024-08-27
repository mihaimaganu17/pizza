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

extern crate alloc;

#[no_mangle]
extern "C" fn entry(_bootloader_start: u32, _bootloader_end: u32, _stack_addr: u32) {
    serial::init();
    mmu::init();

    pxe::build();

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
