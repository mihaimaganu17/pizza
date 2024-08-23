#![no_std]
#![no_main]

mod compiler_builtins;

use core::{
    panic::PanicInfo,
    ops::RangeInclusive,
    alloc::{GlobalAlloc, Layout},
};
use cpu::x86::halt;
use serial::{Serial, println};
use ops::RangeSet;

extern crate alloc;

struct GlobalAllocator;

#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        panic!("No alloc");
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("No dealloc");
    }
}

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

extern "C" {
    // Call a real mode interrrupt with interrupt number `int_code` and with the given register and
    // selector state from `reg_sel_state`.
    fn real_mode_int(int_code: u8, reg_sel_state: *mut RegSelState);
    // Call a PXE API service given by `pxe_code` id.
    fn pxe_call(code_seg: u16, seg_offset: u16, data_seg: u16, data_off: u16, pxe_code: u16);
}

#[derive(Default, Debug)]
#[repr(C)]
pub struct AddressRange {
    // Base address for this range
    base_low: u32,
    base_high: u32,
    // Length of this range
    length_low: u32,
    length_high: u32,
    // Address type of this range
    addr_type: u32,
}

#[no_mangle]
extern "C" fn entry(bootloader_start: u32, bootloader_end: u32, stack_addr: u32) {
    Serial::init();

    unsafe {
        // Type given to available RAM usable by the operating system
        const RANGE_MEMORY: u32 = 1;
        const RANGE_RESERVED: u32 = 2;
        let mut addr_range = AddressRange::default();
        let mut reg_sel_state = RegSelState::default();

        // Set up the register state for a int 15h, ax e820h call
        // ebx contains the continuation value, which must start as 0 and is updated after each
        // interrupt call to e820. `ebx` becomes 0 again at the last returned descriptor
        reg_sel_state.ebx = 0;
        // Does not change between calls
        reg_sel_state.ecx = core::mem::size_of::<AddressRange>() as u32;
        // Does not change between calls
        reg_sel_state.edi = &mut addr_range as *mut AddressRange as u32;

        // Create a new set of memory ranges
        let mut set = RangeSet::new();

        loop {
            // EAX and EDX register values differ between input and output
            reg_sel_state.eax = 0xe820;
            reg_sel_state.edx = u32::from_be_bytes(*b"SMAP");
            real_mode_int(0x15, &mut reg_sel_state);

            // If the range is memory we can use
            if addr_range.addr_type == RANGE_MEMORY {
                // Compute the start and end for the set entry
                let start = ((addr_range.base_high as u64) << 32) | addr_range.base_low as u64;
                let length = ((addr_range.length_high as u64) << 32) | addr_range.length_low as u64;
                // We are substracting 1 here because we use `RangeInclusive`
                let end = start.saturating_add(length.saturating_sub(1));
                // Create a new range
                let entry = RangeInclusive::new(start, end);

                set.insert(entry);
            }

            // If either carry flag is set (error), or the continuation value (ebx) is zero after
            // this call, there are no other descriptors left to be read.
            // Last address range in AMD systems can be explained in qemu/hw/i386/pc.c:782
            if reg_sel_state.eflags & 1 == 1 || reg_sel_state.ebx == 0 { break; }
        }
        // Remove the code and data used by the bootloader
        let bootloader_range = RangeInclusive::new(
            u64::from(bootloader_start),
            u64::from(bootloader_end.saturating_sub(1)),
        );
        set.consume(&bootloader_range).expect("Failed to remove booloader range");

        // Remove some space for the stack
        let stack_size = 0x400;
        let stack_range = RangeInclusive::new(
            u64::from(stack_addr.saturating_sub(stack_size)),
            u64::from(stack_addr.saturating_sub(1)),
        );
        set.consume(&stack_range).expect("Failed to remove stack range");

        // Remove everything up to the 64 KB boundary (0xff_ffff)
        let bios_needs = RangeInclusive::new(
            0,
            0xffffff,
        );
        set.discard(&bios_needs).expect("Failed to remove bios needs");

        println!("{:#x?}", set.ranges());

        println!("Available memory: {:x?}", set.sum());

        let mut test_vec = alloc::vec::Vec::new();
        test_vec.push(5);
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
