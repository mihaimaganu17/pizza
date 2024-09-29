//! Memory manager module for the kernel
use crate::BOOT_STATE;
use core::alloc::{Layout, GlobalAlloc};
use core::ops::RangeInclusive;

// Structure used by the memory manager to allocate memory. This implements `GlobalAlloc` crate in
// order to be used by Rust.
struct GlobalAllocator;

#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut phys_mem_lock = BOOT_STATE.unwrap().mmu.lock();

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
        let mut phys_mem_lock = BOOT_STATE.unwrap().mmu.lock();
        let ptr = ptr as u64;
        let end = ptr.saturating_add(layout.size() as u64).saturating_sub(1);
        // Compute the range to be deallocated
        let range = RangeInclusive::new(ptr, end);

        phys_mem_lock.as_mut()
            .and_then(|mmu| mmu.deallocate(range)).expect("Cannot free memory");
    }
}
