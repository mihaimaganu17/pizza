#![no_std]
#![no_main]

mod compiler_builtins;
mod asm_ffi;
mod mmu;

use core::panic::PanicInfo;
use cpu::x86::halt;
use serial::println;

extern crate alloc;

#[no_mangle]
extern "C" fn entry(_bootloader_start: u32, _bootloader_end: u32, _stack_addr: u32) {
    serial::init();
    mmu::init();

    let phys_mem_lock = crate::mmu::PHYSICAL_MEMORY.lock();

    if let Some(phys_mem) = &*phys_mem_lock {
        println!("{:#x?}", phys_mem.ranges());
        println!("Available memory: {:x?}", phys_mem.sum());
    }
    let mut test_vec = alloc::vec::Vec::new();
    test_vec.push(5);

    halt();
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
    halt()
}
