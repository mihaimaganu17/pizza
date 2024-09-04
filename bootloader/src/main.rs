#![no_std]
#![no_main]

mod compiler_builtins;
mod asm_ffi;
mod memory;
mod pxe;
mod error;

use core::panic::PanicInfo;
use cpu::x86;
use serial::println;
use parse_pe::Pe;
use mmu::{PML4, VirtualAddress, PageSize};

extern crate alloc;

#[no_mangle]
extern "C" fn entry(_bootloader_start: u32, _bootloader_end: u32, _stack_addr: u32) {
    serial::init();
    memory::init();

    let kernel = pxe::download(b"pizza.kernel").expect("Failed to download kernel");
    let kernel = Pe::parse(&kernel).expect("Failed to parse kernel PE");

    kernel.access_sections(|base, size, _bytes| {
        println!("Base {:x}, size {:x}", base, size);
        Some(())
    }).expect("Failed to acess sections");

    // Create a page table and jump in IA-32e mode
    {
        // Get access to phyisical memory
        let mut phys_mem = memory::PHYSICAL_MEMORY.lock();
        let phys_mem = phys_mem.as_mut().expect("Physical memory not initialised");

        unsafe {
            // Create a new PML4 table
            let mut pml4 = PML4::new(phys_mem).expect("Cannot create PML4 table");

            pml4.map(VirtualAddress(0x1337_0000_0000), Some(0), PageSize::Page4Kb)
                .expect("Failed to map memory");
        }
    }

    println!("We made it!\n");

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
