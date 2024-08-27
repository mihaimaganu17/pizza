use crate::asm_ffi::RealModeAddr;

pub const PXENV_PACKET_TYPE_DHCP_ACK: u16 = 2;

#[derive(Debug, Default)]
#[repr(C)]
pub struct GetCachedInfo {
    pub status: u16,
    pub packet_type: u16,
    pub buffer_size: u16,
    pub buffer: RealModeAddr,
    buffer_limit: u16,
}

pub mod opcode {
    pub const GET_CACHED_INFO: u16 = 0x0071;
}
