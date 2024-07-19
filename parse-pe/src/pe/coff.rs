use read_me::{Reader, ReaderError};
use parseme::ReadMe;

#[derive(Debug)]
#[derive(ReadMe)]
pub struct CoffHeader {
    // The number that identifies the type of target machine
    machine: u16,
    // Number of section table immediately following the header
    number_of_sections: u16,
    // The low 32-bits of the number of seconds since epoch, indicatin when the file was created
    time_data_stamp: u32,
    // File offset of the COFF symbol table, or zero if no COFF symbol table is present.
    pointer_to_symbol_table: u32,
    // Number of entries in the symbol table. Can also be used to locate the string table, which
    // immediately follows the symbol table.
    number_of_symbols: u32,
    // Size of the OptionalHeader, required for executable, but not for object files.
    size_of_optional_header: u32,
    // Attributes of the file
    characteristics: u16,
}

#[derive(Debug)]
// TODO: Not there yet
//#[derive(ReadMe)]
//#[from = "u16"]
//#[handler = "try_from"]
pub enum Machine {
    // x64
    I386,
    // Intel 386 or later processors and compatible processors
    AMD64,
}

impl TryFrom<u16> for Machine {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            IMAGE_FILE_MACHINE_I386 => Ok(Self::I386),
            IMAGE_FILE_MACHINE_AMD64 => Ok(Self::AMD64),
            _ => Err(Error::UnsupportedMachine(value))
        }
    }
}

#[derive(Debug)]
pub enum Error {
    UnsupportedMachine(u16),
}

/// x86
pub const IMAGE_FILE_MACHINE_I386: u16 = 0x14c;
/// x64
pub const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;


