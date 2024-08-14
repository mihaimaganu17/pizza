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
    panic!("An empty line above");
    halt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        println!("System panic: {}:{}", loc.file(), loc.line());
    } else {
        println!("System panic: unknown location");
    }
    if let Some(p) = info.payload().downcast_ref::<&str>() {
        println!("System panic: {p:?}");
    } else {
        println!("System panic: unknown reason");
    }
    halt()
}
