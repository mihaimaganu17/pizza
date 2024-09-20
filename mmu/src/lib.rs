#![no_std]
use core::alloc::Layout;
use cpu::x86;

/// Implementors of this trait are capable of taking advantange of Intels x86 4-Level Paging
/// linear address translation capability
pub trait AddressTranslate {
    /// Allocates memory with the specified layout and returns a pointer to that memory.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    /// Translates a physical memory address into a virtual memory one, checking whether or not
    /// the block is available for `size` bytes
    unsafe fn translate(&self, physical_address: PhysicalAddress, size: usize) -> Option<*mut u8>;
    /// Allocates memory and fills it with 0
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> *mut u8 {
        let ptr = self.alloc(layout);
        // Now that we allocated it, we want to zero it out.
        core::slice::from_raw_parts_mut(ptr, layout.size())
            .into_iter()
            .for_each(|entry| *entry = 0);
        ptr
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PhysicalAddress(pub u64);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VirtualAddress(pub u64);

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
// Execute disable. If 1 and the MSR IA32_EFER.NXE bit is 1, instruction fecthes are not allowed
// from the region controlled by this page
const PAGE_NXE: u64 = 1 << 63;

// A x86_64 page table
pub enum PageTable {
    // A 4-level paging page table
    PML4,
}

// Read write execute flags
#[derive(Debug, Clone, Copy)]
pub struct RWX {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

pub struct PML4<'mem, A: AddressTranslate> {
    // The value in cr3 that points to the 4 level page table.
    cr3_root: PhysicalAddress,
    // Keep a reference to the phyiscal memory translator, such that we know we have the lock
    mem: &'mem mut A,
}

#[derive(Debug, Clone, Copy)]
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


impl<'mem, A: AddressTranslate> PML4<'mem, A> {
    /// Instantiate a new PML4 table from a given address. This address must:
    /// - already point to an allocated 4Kb Page Table.
    /// - have the page table either 0 or initilized
    pub unsafe fn from_addr(mem: &'mem mut A, addr: PhysicalAddress) -> Option<Self> {
        Some(PML4 { cr3_root: addr, mem })
    }

    /// Instantiate and allocate a new PML4 table from a given address.
    pub unsafe fn new(mem: &'mem mut A) -> Option<Self> {
        let pml4_table = mem
            .alloc_zeroed(Layout::from_size_align(PAGE_TABLE_SIZE, PAGE_TABLE_SIZE).ok()?);
        Self::from_addr(mem, PhysicalAddress(pml4_table as *mut u64 as u64))
    }

    pub fn cr3(&self) -> PhysicalAddress {
        self.cr3_root
    }

    pub unsafe fn map_zero(
        &mut self,
        virtual_address: VirtualAddress,
        layout: Layout,
        page_size: PageSize,
        rwx: RWX,
    ) -> Result<(), MapError> {
        let region = core::slice::from_raw_parts(self.mem.alloc_zeroed(layout), layout.size());
        self.map_slice(virtual_address, region, page_size, rwx)
    }

    pub fn map_slice(
        &mut self,
        virtual_address: VirtualAddress,
        // Raw value to map at the address
        slice: &[u8],
        page_size: PageSize,
        rwx: RWX,
    ) -> Result<(), MapError> {
        let start = virtual_address.0;
        let end = start.saturating_add(slice.len() as u64);
        for page_frame_addr in (start..end).step_by(page_size.size() as usize) {
            let slice_start = page_frame_addr.saturating_sub(virtual_address.0) as usize;
            serial::println!("Page {:#x?}, Slice start {:#x?}, slice len {:#x?}, end {:#x?}",
                page_frame_addr,
                slice_start,
                slice.len(),
                end,
            );
            // If we are at the last slice, we just read until the end
            let temp_slice = if page_frame_addr.saturating_add(page_size.size()) > end {
                slice.get(slice_start..).ok_or(MapError::RangeOverflow)?
            } else {
                slice
                    .get(slice_start..slice_start.saturating_add(page_size.size() as usize))
                    .ok_or(MapError::RangeOverflow)?
            };
            let page = unsafe {
                // Allocate the page
                let page = self.mem.alloc_zeroed(
                    Layout::from_size_align(page_size.size() as usize, page_size.size() as usize)?
                );
                page.copy_from(temp_slice.as_ptr(), temp_slice.len());
                page as *mut u64 as u64
            };
            self.map_page(VirtualAddress(page_frame_addr), Some(page | 3), page_size, rwx)?;
        }
        Ok(())
    }

    // Map a virtual address using the 4-level paging translation, with page frames of size
    // `page_size` with the desired `rwx` read, write, execute permissions and fill it with the
    // potential value stored in `maybe_raw`
    pub fn map_page(
        &mut self,
        virtual_address: VirtualAddress,
        // Raw value to map at the address
        maybe_raw: Option<u64>,
        page_size: PageSize,
        rwx: RWX,
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
                    Some((virtual_address.0 >> 39) & 0x1ff),
                    Some((virtual_address.0 >> 30) & 0x1ff),
                    Some((virtual_address.0 >> 21) & 0x1ff),
                    Some((virtual_address.0 >> 12) & 0x1ff),
                ]
            }
            PageSize::Page2Mb => {
                [
                    Some((virtual_address.0 >> 39) & 0x1ff),
                    Some((virtual_address.0 >> 30) & 0x1ff),
                    Some((virtual_address.0 >> 21) & 0x1ff),
                    None,
                ]
            }
            PageSize::Page1Gb => {
                [
                    Some((virtual_address.0 >> 39) & 0x1ff),
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
                let mut table_ptr = next_table as *mut u64;
                unsafe {
                    table_ptr = table_ptr.add((*page_table_ptr) as usize);

                    if depth == page_table_ptrs.len() - 1 && maybe_raw.is_some() {
                        *table_ptr = maybe_raw.unwrap();
                        if (*table_ptr & PAGE_PRESENT) != 0
                            && core::mem::size_of::<usize>() != core::mem::size_of::<u64>(){
                            // We caused an update, we need to invalidate the TLB
                            x86::invlpg(*table_ptr);
                        }
                        return Ok(());
                    }

                    // If the table does not exist yet, as in our pointer references a zero entry,
                    // we allocate it
                    if (*table_ptr & PAGE_PRESENT) == 0 {
                        let temp_table_ptr = if depth < page_table_ptrs.len() - 1 {
                            // This should not use the `alloc` crate allocation methods since we want
                            // to keep the pointer valid beyond this scope.
                            self.mem.alloc_zeroed(
                                Layout::from_size_align(PAGE_TABLE_SIZE, PAGE_TABLE_SIZE)?
                            )
                        } else {
                            self.mem.alloc_zeroed(
                                Layout::from_size_align(
                                    page_size.size() as usize,
                                    page_size.size() as usize,
                                )?
                            )
                        };
                        // We asign the new address to our pointer
                        *table_ptr = temp_table_ptr as *mut u64 as u64 ;
                        *table_ptr = *table_ptr | PAGE_PRESENT | PAGE_USER | PAGE_WRITE;
                    }
                    next_table= *table_ptr;

                }
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

        /*
        let page_frame_addr = Self::page_frame_addr(VirtualAddress(next_table), page_size);
        if let Some(raw) = maybe_raw {
            // If the size of the slice is to big to be mapped, we return an error
            if raw.len() > page_size.size() as usize {
                return Err(MapError::DataOverflow(raw.len()));
            }
            unsafe {
                (page_frame_addr as *mut u8).copy_from(raw.as_ptr(), raw.len());
            }
        }*/

        Ok(())
    }

    // Extracts the page frame address given a page table entry, virtual address and a page size
    fn page_frame_offset(virt_addr: VirtualAddress, page_size: PageSize) -> u64 {
        match page_size {
            PageSize::Page4Kb => virt_addr.0 & ((1 << 12) - 1),
            PageSize::Page2Mb => virt_addr.0 & ((1 << 21) - 1),
            PageSize::Page1Gb => virt_addr.0 & ((1 << 30) - 1),
        }
    }

    // Extracts the page frame address given a page table entry, virtual address and a page size
    fn page_frame_addr(virt_addr: VirtualAddress, page_size: PageSize) -> u64 {
        // At this point, we need to map the actual page frame
        match page_size {
            PageSize::Page4Kb => virt_addr.0 & (((1 << 40) - 1) << 12),
            PageSize::Page2Mb => virt_addr.0 & (((1 << 31) - 1) << 21),
            PageSize::Page1Gb => virt_addr.0 & (((1 << 22) - 1) << 30),
        }
    }
}

#[derive(Debug)]
pub enum MapError {
    AddressUnaligned((VirtualAddress, u64)),
    PagePointerZero(usize),
    LayoutError(core::alloc::LayoutError),
    OverflowingIdx(usize),
    RangeOverflow,
    DataOverflow(usize),
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
        let raw: Option<u64> = Some(0x1337_b00b);

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
        let raw: Option<u64> = Some(0x1337_b00b);

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
        let raw: Option<u64> = Some(0x1337_b00b);

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
        let raw: Option<u64> = Some(0x1337_b00b);

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page2Mb, &mut allocator);

        assert!(mapped_page.is_err());
    }

    #[test]
    fn test_1gb_page_ok() {
        let virt_addr = VirtualAddress(0x1234_1234_8000_0000);
        let physical_memory = alloc::vec![0u8; 4096];
        let cr3 = physical_memory.as_ptr() as u64;
        let pml4 = unsafe { PML4::from_addr(PhysicalAddress(cr3)) };
        let mut allocator = Allocator;
        let raw: Option<u64> = Some(0x1337_b00b);

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page1Gb, &mut allocator).unwrap();

        assert!(mapped_page.is_ok());
    }

    #[test]
    fn test_1gb_page_err() {
        let virt_addr = VirtualAddress(0x0123 << 29);
        let physical_memory = alloc::vec![0u8; 4096];
        let cr3 = physical_memory.as_ptr() as u64;
        let pml4 = unsafe { PML4::from_addr(PhysicalAddress(cr3)) };
        let mut allocator = Allocator;
        let raw: Option<u64> = Some(0x1337_b00b);

        let mapped_page = pml4.map(virt_addr, raw, PageSize::Page1Gb, &mut allocator);

        assert!(mapped_page.is_err());
    }
}
