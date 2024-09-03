//! x86 CPU specific routines and instruction wrappers
use core::arch::asm;

/// Write or output a `u8` value to the `I/O` port at `address`
#[inline]
pub fn out_u8(address: u16, value: u8) {
    unsafe { asm!("out dx, al", in("dx") address, in("al") value); }
}

/// Read and return a `u8` value from the `I/O` port at `address`
#[inline]
pub fn in_u8(address: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") address, out("al") value);
    }
    value
}

/// Invalidate TBL entries for page containing m.
#[inline]
#[cfg(target_arch = "x86_64")]
pub unsafe fn invlpg(address: u64) {
    asm!("invlpg rax", in("rax") address);
}

#[cfg(target_arch = "x86")]
pub unsafe fn invlpg(address: u64) {
    let address = address as u32;
    asm!("invlpg eax", in("eax") address);
}

/// Disable interrupts and halt forever
pub fn halt() -> ! {
    loop {
        unsafe {
            asm!(
                "cli",
                "hlt",
            );
        }
    }
}
