#![no_std]
#![no_main]

mod compiler_builtins;

use cpu::x86;
use core::panic::PanicInfo;
use state::BootState;
use core::arch::asm;

#[no_mangle]
extern "C" fn entry(boot_state: &'static BootState) {
    serial::init();
    let screen = unsafe {
        core::slice::from_raw_parts_mut(0xb8000 as *mut u16, 80 * 25)
    };
    {
        serial::print!("Suck it");
        unsafe {
        serial::print!("Boot state in kernel {:#x?}", boot_state as *const BootState as u64);
        }
    }
    screen.iter_mut().for_each(|x| *x = 0x0f75);

    {
        let mut phys_mem = boot_state.mmu.lock();
        //let _phys_mem = phys_mem.as_mut().expect("Physical memory not initialised");
    }
    serial::println!("{:#?}", "TOO MANY BALLS");
    x86::halt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86::halt()
}
