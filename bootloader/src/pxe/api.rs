#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Ip4([u8; 4]);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct MacAddr([u8; 16]);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ServerName([u8; 64]);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BootFile(pub [u8; 128]);

impl Default for Ip4 {
    fn default() -> Self {
        Self([0; 4])
    }
}

impl Default for MacAddr {
    fn default() -> Self {
        Self([0; 16])
    }
}

impl Default for ServerName {
    fn default() -> Self {
        Self([0; 64])
    }
}

impl Default for BootFile {
    fn default() -> Self {
        Self([0; 128])
    }
}

pub mod opcode {
    pub const TFTP_OPEN: u16 = 0x0020;
    pub const TFTP_CLOSE: u16 = 0x0021;
    pub const TFTP_READ: u16 = 0x0022;
    pub const TFTP_GET_FILE_SIZE: u16 = 0x0025;
    pub const PREBOOT_GET_CACHED_INFO: u16 = 0x0071;
}
