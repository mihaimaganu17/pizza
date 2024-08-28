use core::ops::Range;

#[allow(dead_code)]
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
