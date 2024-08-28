#![no_std]
#![no_main]

use cpu::x86;
use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn entry() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86::halt()
}
