use crate::{
    asm_ffi::RealModeAddr,
    pxe::api::{Ip4, MacAddr, ServerName, BootFile},
};

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


#[derive(Debug, Default)]
#[repr(C)]
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
    pub next_server_ip: Ip4,
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
