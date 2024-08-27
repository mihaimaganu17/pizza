use crate::{
    asm_ffi::RealModeAddr,
    pxe::api::{Ip4, BootFile},
};

/// Structure used to query the server and save and report the size of a given file
#[derive(Default)]
#[repr(C, packed)]
pub struct GetFileSize {
    pub status: u16,
    pub server_ip: Ip4,
    _gateway_ip: Ip4,
    pub file_name: BootFile,
    pub file_size: u32,
}

/// Structure used to open a TFTP connection for reading/writing. At any one time there can be only
/// one connection. The connection must be close before another can be opened.
#[derive(Default)]
#[repr(C)]
pub struct TftpOpen {
    pub status: u16,
    // TFTP server IP address in network order
    pub server_ip: Ip4,
    // Relay agent IP address in network order (Big endian)
    _gateway_ip: Ip4,
    // Name of the file to be downloaded.
    pub file_name: BootFile,
    // UDP port TFTP servers is listening to requests on.
    pub port: u16,
    // Requested size of TFTP packet, in byts; with a minimum of 512 bytes. After the call is made
    // this contains the negotiated size of TFTP packet, in bytes; less than or equal to the
    // requested size
    pub packet_size: u16,
}

#[derive(Default)]
#[repr(C)]
pub struct TftpRead {
    // Status of the call
    pub status: u16,
    // Packet number (1-65535) sent from the TFTP server.
    pub packet_number: u16,
    // Number of bytes written to the packet buffer. Last packet is this is less than the size
    // negotiated in the Tftp Open call. Zero is valid.
    pub buffer_size: u16,
    // Address to the buffer that will store the bytes we read
    pub buffer: RealModeAddr,
}
