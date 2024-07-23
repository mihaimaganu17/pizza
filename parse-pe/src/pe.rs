mod coff;
mod opt;
mod sh;

use coff::CoffHeader;
use opt::{OptionalHeader, OptionalHeaderType, DataDirectory};
use sh::SectionHeader;
use read_me::{Reader, ReaderError};
use parseme::ReadMe;

pub const MZ_MAGIC: &[u8; 2] = b"MZ";
pub const PE_MAGIC: &[u8; 4] = b"PE\0\0";

pub const OPT_PE32_MAGIC: u16 = 0x10b;
pub const OPT_PE32_PLUS_MAGIC: u16 = 0x20b;

#[derive(Debug)]
pub struct Pe;

impl Pe {
    pub fn parse(reader: &mut Reader) -> Result<Pe, PeError> {
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

        // Read a section
        let section = reader.read::<SectionHeader>()?;


        return Err(PeError::Bad(section.name));
        Ok(Self)
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
