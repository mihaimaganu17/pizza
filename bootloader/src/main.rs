#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[link(name="c")]
extern "C" {
}

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> i32 {
    panic!("Freaking out")
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
