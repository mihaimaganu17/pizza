//! Module that defines and parses a PE's Section Header
use parseme::ReadMe;
use read_me::{Reader, ReaderError, Primitive};

#[derive(Debug)]
#[derive(ReadMe)]
pub struct SectionHeader {
    // An 8-byte, null-padded UTF-8 encoded string
    pub name: [u8; 8],
    // The total size of the section when loaded into memory
    virtual_size: u32,
    // The RVA (to the image base) when the section is loaded into memory
    virtual_address: u32,
    // The size of the section or the size of the initialized data on disk
    size_of_raw_data: u32,
    // The file pointer to the first page of the section within the COFF file.
    pointer_to_raw_data: u32,
    // The file pointer to the beginning of relocation entries for the section.
    pointer_to_relocations: u32,
    // The file pointer to the beginning of line_number entries for the section
    point_to_line_numbers: u32,
    // Number of relocation entries for the section
    number_of_relocations: u16,
    // Number of line-number entries for the section.
    number_of_line_numbers: u16,
    // Flags with characteristics of the section
    characteristics: u32,
}
