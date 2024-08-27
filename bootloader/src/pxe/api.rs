use crate::asm_ffi::RealModeAddr;

pub const PXENV_PACKET_TYPE_DHCP_ACK: u16 = 2;
// Maximum length of DHCP options: https://dox.ipxe.org/pxe__api_8h_source.html
const _BOOTP_DHCPVEND: u16 = 1024;

#[derive(Debug, Default)]
#[repr(C)]
pub struct GetCachedInfo {
    pub status: u16,
    pub packet_type: u16,
    pub buffer_size: u16,
    pub buffer: RealModeAddr,
    buffer_limit: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct Ip4([u8; 4]);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct MacAddr([u8; 16]);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct ServerName([u8; 64]);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct BootFile([u8; 128]);

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

#[derive(Debug, Default)]
#[repr(packed)]
pub struct BootCachedPacket {
    // Message opcode (1 is for Request and 2 is for Reply)
    _opcode: u8,
    // Hardware type.
    _hardware: u8,
    // Hardware address length
    _hardware_len: u8,
    // Client sets to zero. Optionally used by relay agent when booting via a relay agent
    _gate_ops: u8,
    // Transaction ID. Random number used by the client.
    _ident: u32,
    // Filled in by client. Seconds elapsed since client began address acquisition/renewal process.
    _seconds: u16,
    // BOOTP/DHCP broadcat flags.
    _flags: u16,
    // Client IPv4 address
    _client_ip: Ip4,
    // IP address of the current machine
    _your_ip: Ip4,
    // IP address of next server in boot process
    _next_server_ip: Ip4,
    // Relay agent IP address
    _relay_ip: Ip4,
    // Client hardware address
    _client_mac_addr: MacAddr,
    // Optional server host name. Null terminated string.
    _server_name: ServerName,
    // Boot file name. Null terminated string
    _bootfile: BootFile,
    // Following, we could have a field containin DHCP options. However, that should be 1024 and we
    // will not need it.
}

pub mod opcode {
    pub const GET_CACHED_INFO: u16 = 0x0071;
}
