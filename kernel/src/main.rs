#![no_std]
#![no_main]

mod compiler_builtins;

use cpu::x86;
use serial::println;
use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn entry() {}

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
