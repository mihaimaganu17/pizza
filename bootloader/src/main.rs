#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;
use core::arch::asm;

#[no_mangle]
fn entry() {
    unsafe {
        // https://wiki.osdev.org/Printing_To_Screen
        core::ptr::write(0xB8000 as *mut u16, 0x0f4d);
        asm!(
            "cli",
            "hlt",
        );
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
