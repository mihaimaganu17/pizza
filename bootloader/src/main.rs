#![no_std]
#![no_main]

mod compiler_builtins;
mod asm_ffi;
mod memory;
mod pxe;
mod error;

use core::panic::PanicInfo;
use cpu::x86;
use parse_pe::Pe;
use mmu::{PML4, VirtualAddress, PageSize, RWX};
use state::BootState;
use sync::LockCell;

pub static BOOT_STATE: BootState = BootState {
    mmu: LockCell::new(None),
    serial: LockCell::new(None),
};

extern crate alloc;

#[no_mangle]
extern "C" fn entry(_bootloader_start: u32, _bootloader_end: u32, _stack_addr: u32) {
    {
        let mut serial_lock = BOOT_STATE.serial.lock();
        if serial_lock.is_none() {
            *serial_lock = Some(serial::Serial::init());
        }
    }
    // Initialize memory
    memory::init();

    // Download the kernel
    let kernel = pxe::download(b"pizza.kernel").expect("Kernel download");
    // Parse the kernel's PE
    let kernel = Pe::parse(&kernel).expect("Kernel parsing");

    // Create a page table and jump in IA-32e mode
    let (cr3, stack, entry_point): (u32, u64, u64) =  unsafe {
        // Get access to phyisical memory
        let mut phys_mem_lock = BOOT_STATE.mmu.lock();
        let (cr3, stack, entry_point) = {
        let phys_mem = phys_mem_lock.as_mut().expect("Physical memory not initialised");

        // Create a new PML4 table
        let mut pml4 = PML4::new(phys_mem).expect("Cannot create PML4 table");

        // Create an identity map of the current memory
        for p in (0..(4 * 1024 * 1024 * 1024)).step_by(4096) {
            pml4.map_page(
                VirtualAddress(p),
                p | 3,
                PageSize::Page4Kb,
            ).expect("Failed to map PE");
        }

        // Map the section of the kernel in memory
        kernel.access_sections(|base, _size, bytes| {
            println!("Base1 {:x?}, bytes len {:x?} size {:x?}", base, bytes.len(), _size);
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
            VirtualAddress(0xb00_0000_0000),
            core::alloc::Layout::from_size_align(8192, 4096).expect("Failed to create layout"),
            PageSize::Page4Kb,
            RWX { read: true, write: true, execute: false },
        ).expect("Failed to map a stack");
            (pml4.cr3().0 as u32, 0xb00_0000_0000 + 8192, kernel.entry_point())
        };

        (cr3, stack, entry_point)
    };

    unsafe {
        extern {
            fn enter_ia32e(entry_point: u64, stack: u64, param: u64, cr3: u32) -> !;
        }
        println!("Boot state {:#x?}", &BOOT_STATE as *const BootState as u64);
        enter_ia32e(entry_point, stack, &BOOT_STATE as *const BootState as u64, cr3);
    }
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

/// Writer for serial
#[repr(C)]
pub struct SerialWriter;

impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(serial) = BOOT_STATE.serial.lock().as_mut() {
            serial.write_str(s);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(&mut $crate::SerialWriter, core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(&mut $crate::SerialWriter,
            core::format_args!("{}\n", core::format_args!($($arg)*)));
    };
}
