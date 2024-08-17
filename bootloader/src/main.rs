#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;
use cpu::x86::halt;
use serial::{Serial, print, println};

#[repr(C)]
#[derive(Default)]
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
    fn add_2_numbers(a: i32, b: i32) -> i32;
    // Call a real mode interrrupt with interrupt number `int_code` and with the given register and
    // selector state from `reg_sel_state`.
    fn real_mode_int(int_code: u8, reg_sel_state: &mut RegSelState);
    // Call a PXE API service given by `pxe_code` id.
    fn pxe_call(code_seg: u16, seg_offset: u16, data_seg: u16, data_off: u16, pxe_code: u16);
}

#[no_mangle]
extern "C" fn entry() {
    Serial::init();
    print!("MERE\n");
    print!("{}", unsafe { add_2_numbers(23, 10) });
    let mut reg_sel_state = RegSelState { eax: 0x3, ..Default::default() };
    unsafe { real_mode_int(0x10, &mut reg_sel_state); }
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
