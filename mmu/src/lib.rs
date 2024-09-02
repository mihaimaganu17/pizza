use core::alloc::Layout;

/// Implementors of this trait are capable of taking advantange of Intels x86 4-Level Paging
/// linear address translation capability
pub trait AddressTranslate {
    /// Allocates memory with the specified layout and returns a pointer to that memory.
    fn alloc(&mut self, layout: Layout) -> *mut u8;
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

// Each table entry is referenced by 9 bits, at different locations in the linear address, which
// means each table contains 512 entries. Each entry, is a u64 -> 8 bytes, meaning that an entire
// page table is 512 * 8 = 4096 bytes
const PAGE_TABLE_SIZE: usize = 4096;
// Marks that the page is present
const PAGE_PRESENT: u64 = 1 << 0;
// Marks that the page is writable
const PAGE_WRITE: u64 = 1 << 1;
// Marks that the page is USER accessible (other option is supervisor)
const PAGE_USER: u64 = 1 << 2;

// A x86_64 page table
pub enum PageTable {
    // A 4-level paging page table
    PML4(PML4),
}

pub struct PML4 {
    // The value in cr3 that points to the 4 level page table.
    cr3_root: PhysicalAddress,
}

#[derive(Debug)]
pub enum PageSize {
    Page4Kb,
    Page2Mb,
    Page1Gb,
}

impl PageSize {
    fn size(&self) -> u64 {
        match self {
            PageSize::Page4Kb => 4096,
            PageSize::Page2Mb => 2 * 1024 * 1024,
            PageSize::Page1Gb => 1 * 1024 * 1024 * 1024,
        }
    }
}


impl PML4 {
    /// Instantiate a new PML4 table from a given address. This address must:
    /// - already point to an allocated 4Kb Page Table.
    /// - have the page table either 0 or initilized
    pub unsafe fn from_addr(addr: PhysicalAddress) -> Self {
        PML4 { cr3_root: addr }
    }

    // Map a virtual address using the 4-level paging translation
    pub fn map<A: AddressTranslate>(
        &self,
        virtual_address: VirtualAddress,
        // Raw value to map at the address
        raw: u64,
        page_size: PageSize,
        allocator: &mut A,
    ) -> Result<(), MapError> {
        // Check that the address is aligned according to the size
        let align_mask = page_size.size() - 1;

        // Make sure the virtual address is aligned to the desired size
        if virtual_address.0 & align_mask != 0 {
            return Err(MapError::AddressUnaligned((virtual_address, page_size.size())));
        }

        // For each translation step, the address of the next page table or the address of the
        // page frame (the actual physical page) is computed as follows
        // Bits 51:12 are bits 51:12 of the current table entry
        // Bits 11:3 are the follwing bits from the actual linear address (virtual address):
        // - PML4 -> bits 47:39 of the linear address
        // - PDPTE (page directory pointer table entry) -> bits 38:30 of the linear address
        // - PDE (page directory entry) -> bits 29:20 of the linear address
        // - PTE (page table entry) -> bits 20:12 of the linear address
        // - PageFrame (page table entry) -> bits 11:0 of the linear address
        let page_table_ptrs = match page_size {
            PageSize::Page4Kb => {
                [
                    Some((virtual_address.0 >> 38) & 0x1ff),
                    Some((virtual_address.0 >> 30) & 0x1ff),
                    Some((virtual_address.0 >> 21) & 0x1ff),
                    Some((virtual_address.0 >> 12) & 0x1ff),
                ]
            }
            PageSize::Page2Mb => {
                [
                    Some((virtual_address.0 >> 38) & 0x1ff),
                    Some((virtual_address.0 >> 30) & 0x1ff),
                    Some((virtual_address.0 >> 21) & 0x1ff),
                    None,
                ]
            }
            PageSize::Page1Gb => {
                [
                    Some((virtual_address.0 >> 38) & 0x1ff),
                    Some((virtual_address.0 >> 30) & 0x1ff),
                    None,
                    None,
                ]
            }
        };

        // We start at the cr3 root
        let mut next_table = self.cr3_root.0;

        for (depth, maybe_page_table_ptr) in page_table_ptrs.iter().enumerate() {
            // At this point, the table does exist
            if let Some(page_table_ptr) = maybe_page_table_ptr {
                // First we go to the table
                next_table = next_table & 0xffffffffff000;
                // Cast the address into a pointer, because this is what it essentially is
                let table_ptr = next_table as *mut u64;
                // If the table does not exist yet, as in our pointer references a zero entry,
                // we allocate it
                unsafe {
                    // If the we are not at a page frame and the page is not present
                    if table_ptr.is_null() || (*table_ptr) & PAGE_PRESENT == 0 {
                        let mut temp_table_ptr = allocator
                            .alloc(Layout::from_size_align(PAGE_TABLE_SIZE, PAGE_TABLE_SIZE)?);
                        // Now that we allocated it, we want to zero it out.
                        core::slice::from_raw_parts_mut(temp_table_ptr, PAGE_TABLE_SIZE)
                            .into_iter()
                            .map(|entry| *entry = 0);
                        // We asign the new address to our pointer
                        *table_ptr = temp_table_ptr as *mut u64 as u64;
                        // Mark the new table as PRESENT, WRITE and USER.
                        *table_ptr = *table_ptr | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
                        // Update the local pointer to the next table
                        next_table = *table_ptr;
                    }
                }
                // Now we go to the entry in the table, which is the follow-up table or the page
                // frame
                next_table = next_table | (page_table_ptr << 3);
            } else {
                // The biggest page frame we can have is 1Gb, which needs 2 indirect pointers to be
                // addressed. As such, if we are at a lower depth than 3, this means something is
                // not ok. Technically, this is solely dependant on how we mapped the table
                // pointers, so it could not fail currently.
                if depth < 2 {
                    return Err(MapError::PagePointerZero(depth));
                }
            }
        }

        // At this point, we need to map the actual page frame
        let (page, offset_shift) = match page_size {
            PageSize::Page4Kb => {
                let page = next_table & (((1 << 40) - 1) << 12);
                let offset_shift = 12;
                (page, offset_shift)
            }
            PageSize::Page2Mb => {
                let page = next_table & (((1 << 31) - 1) << 21);
                let offset_shift = 21;
                (page, offset_shift)
            }
            PageSize::Page1Gb => {
                let page = next_table & (((1 << 22) - 1) << 30);
                let offset_shift = 30;
                (page, offset_shift)
            }
        };

        unsafe {
            // Check if the page frame exists.
            if *(page as *mut u64) & PAGE_PRESENT == 0 {
                let size = page_size.size() as usize;
                // If not, we allocate it
                println!("{:?}", page_size);
                let mut temp_table_ptr = allocator
                    .alloc(Layout::from_size_align(size, size)?);
                // And we assign the entry
                *(page as *mut u64) = temp_table_ptr as u64;
                // Mark the new table as PRESENT, WRITE and USER.
                *(page as *mut u64) = *(page as *mut u64) | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
            }

            println!("page ptr {:x?}, page contents {:x?}", page as *mut u64, *(page as *mut u64));
            // At this point, the frame exist, so we just need to assign it the value
            let page_addr = Self::page_frame_addr(virtual_address, page, page_size);
            println!("page addr {:x?} {:x?}", page_addr, virtual_address.0);
            *(page_addr as *mut u64) = raw;
        }

        Ok(())
    }

    // Extracts the page frame address given a page table entry, virtual address and a page size
    fn page_frame_addr(virt_addr: VirtualAddress, page_entry: u64, page_size: PageSize) -> u64 {
        match page_size {
            PageSize::Page4Kb => {
                let page = page_entry & (((1 << 40) - 1) << 12);
                let offset = virt_addr.0 & ((1 << 12) - 1);
                page | offset
            }
            PageSize::Page2Mb => {
                let page = page_entry & (((1 << 31) - 1) << 21);
                let offset = virt_addr.0 & ((1 << 21) - 1);
                page | offset
            }
            PageSize::Page1Gb => {
                let page = page_entry & (((1 << 22) - 1) << 30);
                let offset = virt_addr.0 & ((1 << 30) - 1);
                page | offset
            }
        }
    }
}

#[derive(Debug)]
pub enum MapError {
    AddressUnaligned((VirtualAddress, u64)),
    PagePointerZero(usize),
    LayoutError(core::alloc::LayoutError),
}

impl From<core::alloc::LayoutError> for MapError {
    fn from(err: core::alloc::LayoutError) -> Self {
        Self::LayoutError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        alloc::Layout,
        cell::LazyCell,
    };

    extern crate alloc;

    #[derive(Debug)]
    pub struct Allocator;

    impl AddressTranslate for Allocator {
        fn alloc(&mut self, layout: Layout) -> *mut u8 {
            alloc::vec![0u8; layout.size()].as_mut_ptr()
        }
    }

    #[test]
    fn test_4kb_page_ok() {
        /// Create a new memory unit of 4MB
        let physical_memory = alloc::vec![0u8; 4096];
        let cr3 = physical_memory.as_ptr() as u64;
        let virt_addr = VirtualAddress(0x0123_8000);
        let pml4 = unsafe { PML4::from_addr(PhysicalAddress(cr3)) };
        let mut allocator = Allocator;
        let raw: u64 = 0x1337_b00b;

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page4Kb, &mut allocator);

        assert!(mapped_page.is_ok());
    }

    #[test]
    fn test_4kb_page_err() {
        let physical_memory = alloc::vec![0u8; 4096];
        let cr3 = physical_memory.as_ptr() as u64;
        let virt_addr = VirtualAddress(0x0123_8100);
        let pml4 = unsafe { PML4::from_addr(PhysicalAddress(cr3)) };
        let mut allocator = Allocator;
        let raw: u64 = 0x1337_b00b;

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page4Kb, &mut allocator);

        assert!(mapped_page.is_err());
    }

    #[test]
    fn test_2mb_page_ok() {
        let virt_addr = VirtualAddress(0x0123 << 21);
        let physical_memory = alloc::vec![0u8; 4096];
        let cr3 = physical_memory.as_ptr() as u64;
        let pml4 = unsafe { PML4::from_addr(PhysicalAddress(cr3)) };
        let mut allocator = Allocator;
        let raw: u64 = 0x1337_b00b;

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page2Mb, &mut allocator);

        assert!(mapped_page.is_ok());
    }

    #[test]
    fn test_2mb_page_err() {
        let virt_addr = VirtualAddress(0x0123 << 20);
        let physical_memory = alloc::vec![0u8; 4096];
        let cr3 = physical_memory.as_ptr() as u64;
        let pml4 = unsafe { PML4::from_addr(PhysicalAddress(cr3)) };
        let mut allocator = Allocator;
        let raw: u64 = 0x1337_b00b;

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page2Mb, &mut allocator);

        assert!(mapped_page.is_err());
    }

    //#[test]
    fn test_1gb_page_ok() {
        let virt_addr = VirtualAddress(0x1234_1234_1234_1234);
        let physical_memory = alloc::vec![0u8; 4096];
        let cr3 = physical_memory.as_ptr() as u64;
        let pml4 = unsafe { PML4::from_addr(PhysicalAddress(cr3)) };
        let mut allocator = Allocator;
        let raw: u64 = 0x1337_b00b;

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page1Gb, &mut allocator);

        assert!(mapped_page.is_ok());
    }

    //#[test]
    fn test_1gb_page_err() {
        let virt_addr = VirtualAddress(0x0123 << 29);
        let physical_memory = alloc::vec![0u8; 4096];
        let cr3 = physical_memory.as_ptr() as u64;
        let pml4 = unsafe { PML4::from_addr(PhysicalAddress(cr3)) };
        let mut allocator = Allocator;
        let raw: u64 = 0x1337_b00b;

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page1Gb, &mut allocator);

        assert!(mapped_page.is_err());
    }
}
