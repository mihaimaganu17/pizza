//! Module defining and implementing the memory manager
use ops::RangeSet;
use core::{
    alloc::{GlobalAlloc, Layout},
    ops::RangeInclusive,
};
use crate::asm_ffi::{RegSelState, real_mode_int};
use mmu::Mmu;
use crate::BOOT_STATE;

// Structure used by the memory manager to allocate memory. This implements `GlobalAlloc` crate in
// order to be used by Rust.
struct GlobalAllocator;

#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut phys_mem_lock = BOOT_STATE.mmu.lock();

        let ptr = phys_mem_lock.as_mut()
            .and_then(|mmu| mmu.allocate(layout.size() as u64, layout.align() as u64))
            .unwrap_or(0) as *mut u8;
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // We do not have anything to free for zero sized types.
        if layout.size() == 0 {
            return;
        }
        let mut phys_mem_lock = BOOT_STATE.mmu.lock();
        let ptr = ptr as u64;
        let end = ptr.saturating_add(layout.size() as u64).saturating_sub(1);
        // Compute the range to be deallocated
        let range = RangeInclusive::new(ptr, end);

        phys_mem_lock.as_mut()
            .and_then(|mmu| mmu.deallocate(range)).expect("Cannot free memory");
    }
}


#[derive(Default)]
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

// Initialize the current MMU by querring and gathering all the available physical memory on the
// system, using the e820 call from real mode. We additionally substract the first 64kB of memory
// in order to not overwrite any BIOS needed functions.
pub fn init() -> Option<()> {
    let set = unsafe {
        // Type given to available RAM usable by the operating system
        const RANGE_MEMORY: u32 = 1;
        const _RANGE_RESERVED: u32 = 2;
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
            // EAX and EDX register values differ between input and output.
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
        // Remove everything up to the 64 KB boundary (0xff_ffff)
        let bios_needs = RangeInclusive::new(
            0,
            1024 * 1024 - 1,
        );
        set.discard(&bios_needs)?;

        set
    };

    // Acquire a lock for the `RangeSet`
    let mut phys_mem_lock = BOOT_STATE.mmu.lock();
    // If we previously allocated, panic
    assert!(phys_mem_lock.is_none(), "Already allocated");
    // Insert the set
    *phys_mem_lock = Some(Mmu::new(set));

    Some(())
}
