#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;
use cpu::x86::halt;
use serial::{Serial, print, println};

//#[link(name = "build/utils", kind = "static")]
extern "C" {
    fn add_2_numbers(a: i32, b: i32) -> i32;
    fn switch_to_real_mode();
}

#[no_mangle]
extern "C" fn entry() {
    Serial::init();
    print!("MERE\n");
    print!("{}", unsafe { add_2_numbers(23, 10) });
    unsafe { switch_to_real_mode(); }
    halt();
}

// TODO: Calling convention from PXE handling, such that we can switch back into real mode from
// stage0.asm

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
