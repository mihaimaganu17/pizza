#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;
use cpu::x86::halt;
use serial::{Serial, print, println};

#[no_mangle]
extern "C" fn entry() {
    Serial::init();
    print!("Hello world!\n");
    println!("Hello world2!\n");
    let mem = [b'M'; 1000];
    print!("{}\n", mem[..][80+1000]);
    halt();
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
    halt()
}
