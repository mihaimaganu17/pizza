//! Module containing PXE and iPXE utility functions
use crate::asm_ffi::{real_mode_int, RegSelState, RealModeAddr};
use crate::println;

#[derive(Debug)]
#[repr(packed)]
pub struct PxeNvPlus {
    // "PXENV+" signature
    signature: [u8; 6],
    // API Version number. If the API version number is 0x0201 or higher, use the !PXE structure.
    // If the API version number is less than 0x0201, then use the PXENV+ structure
    version: u16,
    // Lenght of the structure in bytes. Used when computing the checksum
    length: u8,
    // Used to make 8-bit checksum of this structure equal to zero.
    checksum: u8,
    // Far pointer to real-mode PXE/UNDI API entry point. May be CS:0000h
    rm_entry: RealModeAddr,
    // 32-bit offset to protected-mode PXE/UNDI API entry point. Not to be used! For protected-mode
    // API services, use the !PXE structure
    _pm_offset: u32,
    // Protected-mode selector of protected-mode PXE/UNDI API entry point. Not to be used! For
    // protected-mode API services, use the !PXE structure.
    _pm_selector: u16,
    // Stack segment address. Must be set to 0 when removed from memory.
    _stack_seg: u16,
    // Stack segment size in bytes.
    _stack_size: u16,
    // BC code semgent address. Must be set to 0 when removed from memory.
    _bc_code_seg: u16,
    // BC code semgent size. Must be set to 0 when removed from memory.
    _bc_code_size: u16,
    // BC data semgent address. Must be set to 0 when removed from memory.
    _bc_data_seg: u16,
    // BC data semgent size. Must be set to 0 when removed from memory.
    _bc_data_size: u16,
    // UNDI data semgent address. Must be set to 0 when removed from memory.
    _undi_data_seg: u16,
    // UNDI data semgent size. Must be set to 0 when removed from memory.
    _undi_data_size: u16,
    // UNDI code semgent address. Must be set to 0 when removed from memory.
    _undi_code_seg: u16,
    // UNDI code semgent size. Must be set to 0 when removed from memory.
    _undi_code_size: u16,
    // Real mode segment offset pointer to !PXE structure. This field is only present if the API
    // version number is 2.1 or greater.
    pxe_ptr: RealModeAddr,
}

impl PxeNvPlus {
    // Reads the PXENV+ structure, in real mode, from the address given as `real_mode_addr`
    pub fn from_real_mode(real_mode_addr: RealModeAddr) -> Option<Self> {
        // Convert the address into a pointer
        let pxenv_ptr = real_mode_addr.linear() as *const PxeNvPlus;

        // Check is not null
        if pxenv_ptr.is_null() {
            return None;
        }

        // Read the structure, without moving it
        let pxenv = unsafe { pxenv_ptr.read() };

        // Check signature
        if &pxenv.signature != b"PXENV+" { return None; }

        // Compute the checksum
        let length = pxenv.length;

        // Add all the bytes creating the structure
        let checksum = unsafe {
            (0..length).fold(0u8, |acc, idx| {
                acc.wrapping_add((pxenv_ptr as *const u8).add(idx as usize).read())
            })
        };

        // If it is not zero, the checksum is invalid
        if checksum != 0 {
            return None;
        }

        Some(pxenv)
    }
}

/// Checks wheter PXE is installed through the real mode interrupt 0x1A, function 0x5650
pub fn install_check() -> Option<RealModeAddr> {
    let mut reg_state = RegSelState::default();
    // Set eax to the function code
    reg_state.eax = 0x5650;
    // Call real mode interrupt
    unsafe { real_mode_int(0x1A, &mut reg_state); };

    if reg_state.eax == 0x564E && reg_state.eflags & 1 == 0 {
        Some(RealModeAddr::new(reg_state.es, reg_state.ebx as u16))
    } else {
        None
    }
}

pub fn build() -> Option<()> {
    let pxenv_addr = install_check()?;

    let pxenv = PxeNvPlus::from_real_mode(pxenv_addr)?;
    println!("{:#?}", pxenv);
    Some(())
}
