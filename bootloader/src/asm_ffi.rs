#[repr(C)]
#[derive(Default, Debug)]
pub struct RegSelState {
    // All 32-bit registers (protected mode), except esp
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,
    pub eflags: u32,
    // All 16-bit selectors, except cs
    pub ds: u16,
    pub es: u16,
    pub ss: u16,
    pub gs: u16,
    pub fs: u16,
}

extern "C" {
    // Call a real mode interrrupt with interrupt number `int_code` and with the given register and
    // selector state from `reg_sel_state`.
    pub fn real_mode_int(int_code: u8, reg_sel_state: *mut RegSelState);
    // Call a PXE API service given by `pxe_code` id.
    pub fn pxe_call(code_seg: u16, seg_offset: u16, data_seg: u16, data_off: u16, pxe_code: u16);
}

// Represents a x86 cpu real mode address, which is constructred by a segment u16 value and a u16
// offset into that segment
#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct RealModeAddr {
    pub off: u16,
    pub seg: u16,
}

impl RealModeAddr {
    pub fn new(seg: u16, off: u16) -> Self {
        Self { seg, off }
    }
    // Returns the real mode address as a linear value
    pub fn linear(&self) -> u32 {
        (u32::from(self.seg) << 4) + u32::from(self.off)
    }

    /// Creates a new real mode address where segment is the most significant nibble(4th nibble,
    /// starting at 0) shift 24 and address is the 4 least significant nibbles. If we receive a value
    /// which cannot fit in 20 bits, we return `None`.
    /// A nibble is half a byte.
    pub fn _from_linear(addr: u32) -> Option<Self> {
        if addr > (1 << 20) - 1 {
            return None;
        }

        // Segment is the 4th nibble
        let seg = (((0xf << 16) & addr) >> 4) as u16;
        let off = (addr & 0xffff) as u16;

        Some(Self {
            off,
            seg,
        })
    }
}
