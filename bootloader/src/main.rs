#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;
use core::arch::asm;


#[no_mangle]
extern "C" fn entry() {
    unsafe {
        // https://wiki.osdev.org/Printing_To_Screen
        let com_ptr: *const u16 = 0x0400 as *const u16;
        for i in 0..4 {
            let port: u16 = unsafe { com_ptr.offset(i).read() };
            if port == 0 {
                continue;
            }
            init_serial(port);
        }
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
