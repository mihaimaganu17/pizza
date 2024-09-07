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
use mmu::{PML4, VirtualAddress, PageSize, RWX};

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

        let pml4 = unsafe {
            // Create a new PML4 table
            let mut pml4 = PML4::new(phys_mem).expect("Cannot create PML4 table");

            kernel.access_sections(|base, _size, bytes| {
                pml4.map_slice(
                    VirtualAddress(base),
                    bytes,
                    PageSize::Page4Kb,
                    RWX { read: true, write: true, execute: true },
                ).expect("Failed to map PE");
                Some(())
            });

            // Allocate and map a stack
            pml4.map_zero(
                VirtualAddress(0x00_0010_0000),
                core::alloc::Layout::from_size_align(8192, 4096).expect("Failed to create layout"),
                PageSize::Page4Kb,
                RWX { read: true, write: true, execute: true },
            ).expect("Failed to map a stack");
            pml4
        };
        extern {
            fn enter_ia32e(entry_point: u64, stack: u64, cr3: u32) -> !;
        }

        unsafe {
            println!("Entry point {:x?}", kernel.entry_point());

            //x86::halt();
            enter_ia32e(kernel.entry_point(), 0x00_0010_0000 + 8192, pml4.cr3().0 as u32);
        }
    }
    println!("We made it!\n");

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
