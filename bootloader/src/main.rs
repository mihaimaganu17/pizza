#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> i32 {
    panic!("Freaking out")
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
