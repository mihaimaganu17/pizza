//! Module that defines and parses a PE Optional header
use parseme::ReadMe;
use read_me::{Reader, ReaderError, Primitive};

/// PE32
pub const PE32_MAGIC: u16 = 0x10b;
/// PE32+
pub const PE32_PLUS_MAGIC: u16 = 0x20b;

#[derive(Debug)]
#[derive(ReadMe)]
pub struct OptionalHeaderType<T: PeArch + Primitive> {
    magic: u16,
    linker_versions: [u8; 2],
    size_of_code: u32,
    size_of_init_data: u32,
    size_of_uninit_data: u32,
    addr_entry_point: u32,
    // Contains bases of code and bases of data
    bases: T::Bases,
    image_base: T,
    section_aligment: u32,
    file_aligment: u32,
    // Various (OS, Image, Subsystem) versions
    versions: [u16; 6],
    win32_version: u32,
    size_of_image: u32,
    size_of_headers: u32,
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

impl<T: PeArch + Primitive> OptionalHeaderType<T> {
    pub fn image_base(&self) -> T {
        self.image_base
    }
}

impl<T: PeArch + Primitive> OptionalHeaderType<T> {
    pub fn number_of_rva_and_sizes(&self) -> u32 {
        self.number_of_rva_and_sizes
    }
}

#[derive(Debug)]
pub enum OptionalHeader {
    PE32(OptionalHeaderType<u32>),
    PE32Plus(OptionalHeaderType<u64>),
}

impl OptionalHeader {
    pub fn number_of_rva_and_sizes(&self) -> u32 {
        match self {
            Self::PE32(opt) => opt.number_of_rva_and_sizes(),
            Self::PE32Plus(opt) => opt.number_of_rva_and_sizes(),
        }
    }

    pub fn image_base(&self) -> u64 {
        match self {
            Self::PE32(opt) => opt.image_base.as_u64(),
            Self::PE32Plus(opt) => opt.image_base.as_u64(),
        }
    }
}

#[derive(Debug)]
#[derive(ReadMe)]
pub struct DataDirectory {
    rva: u32,
    size: u32,
}

pub trait PeArch: Clone + Copy {
    type Bases: Primitive;

    fn as_u64(self) -> u64;
}

impl PeArch for u32 {
    type Bases = [u32; 2];

    fn as_u64(self) -> u64 {
        u64::from(self)
    }
}

impl PeArch for u64 {
    type Bases = [u32; 1];

    fn as_u64(self) -> u64 {
        self
    }
}
