#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;
use cpu::x86::halt;
use serial::Serial;

#[no_mangle]
extern "C" fn entry() {
    Serial::init();
    halt();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
