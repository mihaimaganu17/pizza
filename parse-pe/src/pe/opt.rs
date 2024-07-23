//! Module that defines and parses a PE Optional header
use parseme::ReadMe;
use read_me::{Reader, ReaderError, Primitive};

/// PE32
pub const PE32_MAGIC: u16 = 0x10b;
/// PE32+
pub const PE32_PLUS_MAGIC: u16 = 0x20b;

#[derive(Debug)]
#[derive(ReadMe)]
pub struct OptionalHeader<T: PeArch + Primitive> {
    magic: u16,
    linker_versions: [u8; 2],
    size_of_code: u32,
    size_of_init_data: u32,
    size_of_uninit_data: u32,
    addr_entry_point: u32,
    // Contains bases of code and bases of data
    bases: T::Bases,
    image_base: T,
    section_aligments: u32,
    file_aligment: u32,
    // Various (OS, Image, Subsystem) versions
    versions: [u16; 6],
    win32_version: u32,
    size_of_image: u32,
    checksum: u32,
    subsystem: u16,
    dll_characteristics: u16,
    size_of_stack_reserve: T,
    size_of_stack_commit: T,
    size_of_heap_reserve: T,
    size_of_heap_commit: T,
    loader_flags: u32,
    number_of_rva_and_sizes: u32,
}

#[derive(ReadMe)]
pub struct Temp<T: Primitive + PeArch> {
    pub link_version: [u8; 2],
    pub size: [u8; 10],
    pub bases: T::Bases,
    pub addr: T,
}

pub trait PeArch {
    type Bases: Primitive;
}

impl PeArch for u32 {
    type Bases = [u32; 2];
}

impl PeArch for u64 {
    type Bases = [u32; 1];
}
