//! x86 CPU specific routines and instruction wrappers
use core::arch::asm;

const IA32_GS_BASE: u32 = 0xC000_0101;

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
    asm!("invlpg {0}", in(reg) address);
}

#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn invlpg(address: u64) {
    let addr: usize = address as usize;
    asm!("invlpg [{0}]", in(reg) addr);
}

/// Write the contents of value into EDX:EAX (EDX - High 32 bits and EAX - Low 32 bits) into the
/// 64-bit MSR specified in the ECX register.
#[inline]
pub unsafe fn wrmsr(value: u64, msr: u32) {
    let edx = ((value >> 32) & (u32::MAX as u64)) as u32;
    let eax = (value & (u32::MAX as u64)) as u32;
    asm!("wrmsr", in("edx") edx, in("eax") eax, in("ecx") msr);
}

/// This is not the using the instruction `wrgsbase`, but rather write to the IA32_GS_BASE MSR
#[inline]
pub unsafe fn write_gs_base(value: u64) {
    wrmsr(value, IA32_GS_BASE);
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
