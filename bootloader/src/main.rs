#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;
use cpu::x86::halt;
use serial::{Serial, print, println};

#[repr(C)]
#[derive(Default, Debug)]
struct RegSelState {
    // All 32-bit registers (protected mode), except esp
    eax: u32,
    ecx: u32,
    edx: u32,
    ebx: u32,
    esp: u32,
    ebp: u32,
    esi: u32,
    edi: u32,
    eflags: u32,
    // All 16-bit selectors, except cs
    ds: u16,
    es: u16,
    ss: u16,
    gs: u16,
    fs: u16,
}


//#[link(name = "build/utils", kind = "static")]
extern "C" {
    //fn add_2_numbers(a: i32, b: i32) -> i32;
    // Call a real mode interrrupt with interrupt number `int_code` and with the given register and
    // selector state from `reg_sel_state`.
    fn real_mode_int(int_code: u8, reg_sel_state: *mut RegSelState);
    // Call a PXE API service given by `pxe_code` id.
    //fn pxe_call(code_seg: u16, seg_offset: u16, data_seg: u16, data_off: u16, pxe_code: u16);
}

#[no_mangle]
extern "C" fn entry() {
    Serial::init();

    unsafe {
#[derive(Default, Debug)]
#[repr(C)]
pub struct AddressRange {
    // Base address for this range
    base_addr_low: u32,
    base_addr_high: u32,
    // Length of this range
    length_low: u32,
    length_high: u32,
    // Address type of this range
    addr_type: u32,
}

        let mut addr_range = AddressRange::default();

        // Get the memory map of the system using the int=15h and ax=e820h interrupt
        let mut reg_sel_state = RegSelState {
            eax: 0xe820,
            ebx: 0,
            edi: &mut addr_range as *mut AddressRange as u32,
            ecx: 20,//core::mem::size_of::<AddressRange>() as u32,
            edx: u32::from_be_bytes(*b"SMAP"),
            ..Default::default()
        };

        real_mode_int(0x15, &mut reg_sel_state);

        print!("AddressRange {:?}", addr_range);
    }
    halt();
}

// TODO: Calling convention from PXE handling, such that we can switch back into real mode from
// stage0.asm

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
