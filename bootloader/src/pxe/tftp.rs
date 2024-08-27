use crate::pxe::api::{Ip4, BootFile};

/// Structure used to query the server and save and report the size of a given file
#[derive(Default)]
#[repr(packed)]
pub struct GetFileSize {
    pub status: u16,
    pub server_ip: Ip4,
    _gateway_ip: Ip4,
    pub file_name: BootFile,
    pub file_size: u32,
}
