#![no_std]
#![no_main]

mod compiler_builtins;

use cpu::x86;
use serial::println;
use core::panic::PanicInfo;
use state::BootState;

#[no_mangle]
extern "C" fn entry(boot_state: &BootState) {
    //serial::init();
    {
        //serial::println!("Suck it");
    }
    let screen = unsafe {
        core::slice::from_raw_parts_mut(0xb8000 as *mut u16, 80 * 25)
    };
    screen.iter_mut().for_each(|x| *x = 0x0f75);

    {
        let mut phys_mem = boot_state.mmu.lock();
        let _phys_mem = phys_mem.as_mut().expect("Physical memory not initialised");
    }
    x86::halt();
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
    x86::halt()
}
