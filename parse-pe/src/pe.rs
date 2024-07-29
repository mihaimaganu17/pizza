mod coff;
mod opt;
mod sh;

use coff::CoffHeader;
use opt::{OptionalHeader, OptionalHeaderType, DataDirectory};
use sh::{SectionHeader, SectionHeadersIterator};
use read_me::{Reader, ReaderError};
use parseme::ReadMe;

pub const MZ_MAGIC: &[u8; 2] = b"MZ";
pub const PE_MAGIC: &[u8; 4] = b"PE\0\0";

pub const OPT_PE32_MAGIC: u16 = 0x10b;
pub const OPT_PE32_PLUS_MAGIC: u16 = 0x20b;

#[derive(Debug)]
pub struct Pe<'data> {
    bytes: &'data [u8],
    coff_header: CoffHeader,
    opt_header: OptionalHeader,
    section_headers_offset: usize,
}

impl<'data> Pe<'data> {
    pub fn parse(bytes: &'data [u8]) -> Result<Pe, PeError> {
        let mut reader = Reader::from(bytes);
        // Check MZ
        let mz = reader.read_bytes(MZ_MAGIC.len())?;

        if mz != MZ_MAGIC {
            return Err(PeError::MZMagic);
        }

        // Go to the PE offset.
        // Get the offset from the 0x3c location
        reader.seek(0x3c);
        let pe_offset = reader.read::<u32>()?;
        // Move to that offset
        reader.seek(usize::try_from(pe_offset)?);
        // Read the PE magic
        let pe = reader.read_bytes(PE_MAGIC.len())?;

        // Check we have the PE magic
        if pe != PE_MAGIC {
            return Err(PeError::PEMagic);
        }

        let coff_header = reader.read::<CoffHeader>()?;

        // Peek into the `OptionalHeader` magic to get the architecture (x86 or x64)
        let opt_magic = reader.peek::<u16>()?;

        let opt_header = if opt_magic == OPT_PE32_MAGIC {
            OptionalHeader::PE32(reader.read::<OptionalHeaderType<u32>>()?)
        } else if opt_magic == OPT_PE32_PLUS_MAGIC {
            OptionalHeader::PE32Plus(reader.read::<OptionalHeaderType<u64>>()?)
        } else {
            return Err(PeError::UnsupportedOptionalMagic(opt_magic));
        };

        for _ in 0..opt_header.number_of_rva_and_sizes() {
            let data_dir = reader.read::<DataDirectory>();
        }

        // Save the offset of the section headers
        let offset = reader.offset();

        Ok(Self {
            bytes,
            coff_header,
            opt_header,
            section_headers_offset: offset,
        })
    }

    /// Returns the absolute virtual address of this PE's entry point
    pub fn entry_point(&self) -> u64 {
        self.opt_header.image_base().saturating_add(u64::from(self.opt_header.addr_entry_point()))
    }

    /// Returns an iterator over the section headers of this PE
    pub fn section_headers(&self) -> SectionHeadersIterator {
        // Offset of the sections
        SectionHeadersIterator::from(
            self.bytes,
            self.section_headers_offset,
            usize::from(self.coff_header.number_of_sections()),
        )
    }

    pub fn access_sections<F: Fn(usize, usize, &[u8]) -> Option<()>>(&self, f: F) -> Option<()> {
        for section in self.section_headers() {
            let section_start = usize::try_from(section.pointer_to_raw_data()).ok()?;
            // Get the smallest size representation of the section in order to reduce memory
            // footprint
            let section_size = core::cmp::min(section.size_of_raw_data(), section.virtual_size());
            let section_end = usize::try_from(section.pointer_to_raw_data()
                .saturating_add(section_size)).ok()?;
            let bytes = self.bytes.get(section_start..section_end)?;

            // Compute the absolute Virtual Address of this section
            let section_base = usize::try_from(self.opt_header.image_base()
                .saturating_add(u64::from(section.virtual_address())))
                .ok()?;
            let section_size = usize::try_from(section_size).ok()?;

            f(section_base, section_size, bytes)?;
        }

        Some(())
    }
}

#[derive(Debug)]
pub enum PeError {
    ReaderError(ReaderError),
    MZMagic,
    PEMagic,
    UnsupportedOptionalMagic(u16),
    Bad([u8; 8]),
    TryFromIntError(core::num::TryFromIntError),
}

impl From<ReaderError> for PeError {
    fn from(err: ReaderError) -> Self {
        Self::ReaderError(err)
    }
}

impl From<core::num::TryFromIntError> for PeError {
    fn from(err: core::num::TryFromIntError) -> Self {
        Self::TryFromIntError(err)
    }
}
