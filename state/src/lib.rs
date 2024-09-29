#![no_std]

//! Pacakge that contains states and contexts to be passed between the different stages of booting

use mmu::Mmu;
use serial::Serial;
use sync::lockcell::LockCell;

/// Contains the bidirectional state to be passed between the bootloader and the kernel
#[repr(C)]
pub struct BootState {
    pub mmu: LockCell<Option<Mmu>>,
    pub serial: LockCell<Option<Serial>>,
}
