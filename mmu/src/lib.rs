#![no_std]

/// Implementors of this trait are capable of taking advantange of Intels x86 4-Level Paging
/// linear address translation capability
pub trait LinearAddressTranslate {}

pub struct PhysicalAddress(u64);
pub struct VirtualAddress(u64);

// A x86_64 page table
pub enum PageTable {
    // A 4-level paging page table
    PML4(PML4),
}

pub struct PML4 {
    // The value in cr3 that points to the 4 level page table.
    cr3_root: PhysicalAddress,
}

impl PML4 {
    // Map a virtual address using the 4-level paging translation
    pub fn map(&self) -> Result<(), MapError> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum MapError {}

#[cfg(test)]
mod tests {}
