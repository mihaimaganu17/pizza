//! Module containing PXE and iPXE utility functions
mod preboot;
mod tftp;
mod api;

use crate::{
    asm_ffi::{real_mode_int, pxe_call, RegSelState, RealModeAddr},
    println,
    error::PxeError,
};
use sync::LockCell;
use alloc::vec::Vec;
use preboot::*;
use api::*;

static PXE_LOCK: LockCell<()> = LockCell::new(());

#[derive(Debug)]
#[repr(packed)]
pub struct PxeNvPlus {
    // "PXENV+" signature
    signature: [u8; 6],
    // API Version number. If the API version number is 0x0201 or higher, use the !PXE structure.
    // If the API version number is less than 0x0201, then use the PXENV+ structure
    version: u16,
    // Lenght of the structure in bytes. Used when computing the checksum
    length: u8,
    // Used to make 8-bit checksum of this structure equal to zero.
    _checksum: u8,
    // Far pointer to real-mode PXE/UNDI API entry point. May be CS:0000h
    _rm_entry: RealModeAddr,
    // 32-bit offset to protected-mode PXE/UNDI API entry point. Not to be used! For protected-mode
    // API services, use the !PXE structure
    _pm_offset: u32,
    // Protected-mode selector of protected-mode PXE/UNDI API entry point. Not to be used! For
    // protected-mode API services, use the !PXE structure.
    _pm_selector: u16,
    // Stack segment address. Must be set to 0 when removed from memory.
    _stack_seg: u16,
    // Stack segment size in bytes.
    _stack_size: u16,
    // BC code semgent address. Must be set to 0 when removed from memory. BC stands for base code
    _bc_code_seg: u16,
    // BC code semgent size. Must be set to 0 when removed from memory.
    _bc_code_size: u16,
    // BC data semgent address. Must be set to 0 when removed from memory.
    _bc_data_seg: u16,
    // BC data semgent size. Must be set to 0 when removed from memory.
    _bc_data_size: u16,
    // UNDI data semgent address. Must be set to 0 when removed from memory.
    _undi_data_seg: u16,
    // UNDI data semgent size. Must be set to 0 when removed from memory.
    _undi_data_size: u16,
    // UNDI code semgent address. Must be set to 0 when removed from memory.
    _undi_code_seg: u16,
    // UNDI code semgent size. Must be set to 0 when removed from memory.
    _undi_code_size: u16,
    // Real mode segment offset pointer to !PXE structure. This field is only present if the API
    // version number is 2.1 or greater.
    pxe_ptr: RealModeAddr,
}

impl PxeNvPlus {
    // Reads the PXENV+ structure, in real mode, from the address given as `real_mode_addr`
    pub fn from_real_mode(real_mode_addr: RealModeAddr) -> Option<Self> {
        // Convert the address into a pointer
        let pxenv_ptr = real_mode_addr.linear() as *const PxeNvPlus;

        // Check is not null
        if pxenv_ptr.is_null() {
            return None;
        }

        // Read the structure, without moving it
        let pxenv = unsafe { pxenv_ptr.read() };

        // Check signature
        if &pxenv.signature != b"PXENV+" { return None; }

        // Compute the checksum
        let length = pxenv.length;

        // Add all the bytes creating the structure
        let checksum = unsafe {
            (0..length).fold(0u8, |acc, idx| {
                acc.wrapping_add((pxenv_ptr as *const u8).add(idx as usize).read())
            })
        };

        // If it is not zero, the checksum is invalid
        if checksum != 0 {
            return None;
        }

        Some(pxenv)
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn pxe_ptr(&self) -> RealModeAddr {
        self.pxe_ptr
    }
}

#[derive(Debug)]
// Protected mode segment descriptor
#[repr(packed(2))]
pub struct SegDesc {
    // The real mode segment or protected mode selector.
    _segment_address: u16,
    // The physical address of the segment
    _physical_address: u32,
    // Size of the segment.
    _seg_size: u16,
}

#[derive(Debug)]
#[repr(C)]
pub struct Pxe {
    // Signature `!PXE` of this structure
    signature: [u8; 4],
    // Length of this structure in bytes. Must be used when computing checksum
    length: u8,
    // 2's complement used to make structure byte checksum equal zero.
    checksum: u8,
    // Revision of this structure is zero.
    _revision: u8,
    // Must be zero.
    _reserved1: u8,
    // Real mode segment::offset of UNDI ROM ID structure. Check this structure if you need to know
    // the UNDI API revision level. Filled in by UNDI loader module.
    _undi_rom_id: RealModeAddr,
    // Real mode segment::offset of BC(base code) ROM ID structure. Must be set to zero if BC is
    // removed from memory. Check this structure if you need to know the BC API revision level.
    // Filled in by base-code loader module.
    _bc_rom_id: RealModeAddr,
    // PXE API entry point for 16-bit stack segment. This API entry point is in the UNDI code
    // segment and must not be CS:0000h. Filled in by UNDI loader module.
    entry_point_sp: RealModeAddr,
    // PXE API entry point for 32-bit stack segment. May be zero. This API entry point is in the
    // UNDI code segment and must not be CS:0000h. Filled in by UNDI loader module.
    entry_point_esp: RealModeAddr,
    // Far pointer to DHCP/TFTP status call-out procedure.
    // If -1(0xfffff), DHCP/TFTP will not make status calls.
    // If 0, DHCP/TFTP will use the internal status call-out procedure.
    // This field defaults to 0.
    // Note: The interanal status call-out procedure uses BIOS I/O interrupts and will only work in
    // real mode. This field must be updated before making any base-code API calls in protected
    // mode.
    _status_callout: RealModeAddr,
    // Must be zero.
    _reserved2: u8,
    // Number of segment descriptors needed in protected mode and defined in this table.
    // UNDI requires 4
    // UNDI + BC requires 7.
    _seg_desc_count: u8,
    // First protected mode selector assigned to PXE. Protected mode selectors assigned to PXE must
    // be consecutive. Not used in real mode. Filled in by application before switching to
    // protected mode.
    first_selector: u16,
    // Some implementations may need more selectors. The first seven are required to be implemented
    // in this order.
    // Note: These descriptors always contain the physical addresses of the segments and the
    // protected mode driver must not overwrite them with the virtual addresses. Filled in by UNDI
    // and base-code loader modules before any API calls are made.
    _stack: SegDesc,
    _undi_data: SegDesc,
    _undi_code: SegDesc,
    _undi_code_write: SegDesc,
    _bc_data: SegDesc,
    _bc_code: SegDesc,
    _bc_code_write: SegDesc,
}

impl Pxe {
    pub fn from_real_mode(real_mode_addr: RealModeAddr) -> Option<Self> {
        let pxe_ptr = real_mode_addr.linear() as *const Pxe;

        // Check is not null
        if pxe_ptr.is_null() {
            return None;
        }
        // Read the structure, without moving it
        let pxenv = unsafe { pxe_ptr.read() };

        // Check signature
        if &pxenv.signature != b"!PXE" { return None; }

        // Compute the checksum
        let length = pxenv.length;

        // Add all the bytes creating the structure
        let checksum = unsafe {
            (0..length).fold(0u8, |acc, idx| {
                acc.wrapping_add((pxe_ptr as *const u8).add(idx as usize).read())
            })
        };

        // If it is not zero, the checksum is invalid
        if checksum != 0 {
            return None;
        }

        Some(pxenv)
    }

    pub fn get_cached_info(&self) -> Result<(GetCachedInfo, BootCachedPacket), PxeError> {
        let mut cached_info = GetCachedInfo::default();
        // Create a default Bootstrap Protocol packet.
        let mut bootp_packet = BootCachedPacket::default();

        cached_info.packet_type = PXENV_PACKET_TYPE_DHCP_ACK;
        cached_info.buffer_size = core::mem::size_of::<BootCachedPacket>() as u16;
        cached_info.buffer.off = &mut bootp_packet as *mut _ as u16;

        unsafe {
            pxe_call(
                self.entry_point_sp.seg,
                self.entry_point_sp.off,
                0,
                &mut cached_info as *mut _ as u16,
                opcode::PREBOOT_GET_CACHED_INFO
            )
        };

        if cached_info.status != 0 {
            return Err(PxeError::ApiStatus(cached_info.status));
        }

        Ok((cached_info, bootp_packet))
    }

    /// Queries the TFTP server for the file size of the given `filename`, which is a ASCI null
    /// terminated string of at most 128 characters.
    pub fn tftp_get_file_size(&self, server_ip: &Ip4, filename: &[u8]) -> Result<u32, PxeError> {
        let mut get_file_size = tftp::GetFileSize::default();

        if filename.len() + 1 > get_file_size.file_name.0.len() {
            return Err(PxeError::FilenameTooLarge);
        }

        get_file_size.file_name.0.get_mut(0..filename.len())
            .ok_or(PxeError::InvalidRange(0..filename.len()))?
            .copy_from_slice(filename);

        get_file_size.server_ip = *server_ip;

        unsafe {
            pxe_call(
                self.entry_point_sp.seg,
                self.entry_point_sp.off,
                0,
                &mut get_file_size as *mut _ as u16,
                opcode::TFTP_GET_FILE_SIZE,
            )
        };

        if get_file_size.status != 0 {
            return Err(PxeError::ApiStatus(get_file_size.status));
        }

        Ok(get_file_size.file_size)
    }

    /// Opens a TFTP connection for reading/writing. At any one time there can be only one open
    /// connection. The connection must be closed before another can be opened. Returns the
    /// negotiated size of TFTP packet, in bytes, which can be equal to the requested size or less.
    /// Warning: Service cannot be used if an MTFTP or UDP connection is active.
    /// Service cannot be used in protected mode if the StatusCallout field in the !PXE structure
    /// is set to zero.
    /// Service cannot be used with a 32-bit stack segment.
    pub fn tftp_open(&self, server_ip: &Ip4, filename: &[u8], size: u16) -> Result<u16, PxeError> {
        let mut tftp_open = tftp::TftpOpen::default();

        if filename.len() + 1 > tftp_open.file_name.0.len() {
            return Err(PxeError::FilenameTooLarge);
        }

        tftp_open.file_name.0.get_mut(0..filename.len())
            .ok_or(PxeError::InvalidRange(0..filename.len()))?
            .copy_from_slice(filename);

        // IPs and Ports in this structure have to be network order (big endian)
        tftp_open.server_ip = *server_ip;
        tftp_open.packet_size = size;
        tftp_open.port = 69u16.to_be();

        unsafe {
            pxe_call(
                self.entry_point_sp.seg,
                self.entry_point_sp.off,
                0,
                &mut tftp_open as *mut _ as u16,
                opcode::TFTP_OPEN,
            )
        };

        if tftp_open.status != 0 {
            return Err(PxeError::ApiStatus(tftp_open.status));
        }

        Ok(tftp_open.packet_size)
    }

    /// Reads the `file_name` from the open TFTP connection, where `file_size` is the size of the
    /// entire file and `buffer_size` is the size of the packet.
    pub fn tftp_read(&self, size: u32) -> Result<Vec<u8>, PxeError> {
        // Allocate a new buffer to store the data. We allocate directly, such that we avoid
        // multiple allocations
        let mut buffer = Vec::with_capacity(size as usize);

        const BUFFER_SIZE: u16 = 512;

        // Create a new read structure
        let mut tftp_read = tftp::TftpRead::default();
        let mut temp_buffer = [0u8; BUFFER_SIZE as usize];
        tftp_read.buffer = RealModeAddr { off: &mut temp_buffer as *mut _ as u16, seg: 0u16 };

        loop {
            unsafe {
                pxe_call(
                    self.entry_point_sp.seg,
                    self.entry_point_sp.off,
                    0,
                    &mut tftp_read as *mut _ as u16,
                    opcode::TFTP_READ,
                )
            };

            // If status is unsuccessful or if the buffer size is corrupted, return error
            if tftp_read.status != 0 || BUFFER_SIZE < tftp_read.buffer_size {
                return Err(PxeError::ApiStatus(tftp_read.status));
            }

            // Extend our allocation by the contents of the buffer
            buffer.extend(&temp_buffer[..tftp_read.buffer_size as usize]);

            // If we read less than the packer size, we know this is the last packet
            if tftp_read.buffer_size < BUFFER_SIZE {
                break;
            }
        }

        Ok(buffer)
    }

    /// Close a previously opened TFTP connection.
    /// Warning: Service cannot be used if there is not an active MTFTP connection.
    /// Service cannot be used in protected mode if the StatusCallout field in the !PXE structure
    /// is set to zero.
    /// Service cannot be used with a 32-bit stack segment.
    pub fn tftp_close(&self) -> Result<(), PxeError> {
        let mut status: u16 = 0;

        unsafe {
            pxe_call(
                self.entry_point_sp.seg,
                self.entry_point_sp.off,
                0,
                &mut status as *mut _ as u16,
                opcode::TFTP_CLOSE,
            )
        };

        if status != 0 {
            return Err(PxeError::ApiStatus(status));
        }

        Ok(())
    }
}


/// Checks wheter PXE is installed through the real mode interrupt 0x1A, function 0x5650
pub fn install_check() -> Option<RealModeAddr> {
    let mut reg_state = RegSelState::default();
    // Set eax to the function code
    reg_state.eax = 0x5650;
    // Call real mode interrupt
    unsafe { real_mode_int(0x1A, &mut reg_state); };

    if reg_state.eax == 0x564E && reg_state.eflags & 1 == 0 {
        Some(RealModeAddr::new(reg_state.es, reg_state.ebx as u16))
    } else {
        None
    }
}

/// Guarder call(in multithreaded contexts) to PXE installation check and TFTP new file downloads.
pub fn build() -> Result<(), PxeError> {
    // Make sure this is multithread safe. The lock gets dropped at the end of this function
    let _pxe_lock = PXE_LOCK.lock();

    let pxenv_addr = install_check()
        .ok_or(PxeError::InstallCheck)?;

    let pxenv = PxeNvPlus::from_real_mode(pxenv_addr)
        .ok_or(PxeError::PxeNvPlus)?;

    // If the API version is 0x201 or higher, we have a `!PXE` structure
    let pxe = if pxenv.version() >= 0x201 {
        Pxe::from_real_mode(pxenv.pxe_ptr()).ok_or(PxeError::Pxe)
    } else {
        Err(PxeError::Pxe)
    }?;

    let (_cached_info, bootp_packet)= pxe.get_cached_info()?;

    let file_name = b"test_file2";

    // Get the file size of the desired file
    let file_size = pxe.tftp_get_file_size(&bootp_packet.next_server_ip, file_name)?;
    // Open a TFTP connection and negotiate for a packet size
    let _negotiated_packet_size = pxe.tftp_open(&bootp_packet.next_server_ip, file_name, 512)?;
    // Read the file
    let downloaded = pxe.tftp_read(file_size)?;
    println!("We downloaded {} bytes", downloaded.len());
    // Close the TFTP connection
    let _ = pxe.tftp_close()?;

    Ok(())
}
