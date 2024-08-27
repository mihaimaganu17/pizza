use core::ops::Range;

#[derive(Debug)]
pub enum PxeError {
    InstallCheck,
    PxeNvPlus,
    Pxe,
    ApiStatus(u16),
    FilenameTooLarge,
    InvalidRange(Range<usize>),
    InvalidBufferAddr(u32),
}
