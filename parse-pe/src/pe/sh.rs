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

#[derive(Debug)]
pub struct SectionHeadersIterator<'data> {
    bytes: &'data [u8],
    offset: usize,
    number_of_sections: usize,
}

impl<'data> SectionHeadersIterator<'data> {
    pub fn from(bytes: &'data [u8], offset: usize, number_of_sections: usize) -> Self {
        Self {
            bytes,
            offset,
            number_of_sections,
        }
    }
}

impl<'data> Iterator for SectionHeadersIterator<'data> {
    type Item = SectionHeader;

    fn next(&mut self) -> Option<Self::Item> {
        // If we already consumed all the bytes or we parsed all the section headers, return `None`
        if self.offset >= self.bytes.len() || self.number_of_sections == 0 {
            return None;
        }

        // Wrap the bytes into a reader, for easier use
        let mut reader = Reader::from(self.bytes);
        // Skip the already parsed bytes
        reader.skip(self.offset);

        if let Ok(section) = reader.read::<SectionHeader>() {
            // First update the offset
            self.offset = reader.offset();
            // Also decrement the number of sections we still have to parse
            self.number_of_sections = self.number_of_sections.saturating_sub(1);
            // Return the section header
            Some(section)
        } else {
            None
        }
    }
}
