#![no_std]
//! A serial port implementation as described in https://wiki.osdev.org/Serial_Ports
use sync::LockCell;
use cpu::x86::{out_u8, in_u8};

pub struct Serial {
    ports: [Option<LockCell<u16>>; 4],
}

impl Serial {
    pub fn init() -> Self {
        const INIT_PORT_VALUE: Option<LockCell<u16>> = None;
        let mut ports = [INIT_PORT_VALUE; 4];
        unsafe {
            // https://wiki.osdev.org/Printing_To_Screen
            let com_ptr: *const u16 = 0x0400 as *const u16;
            for i in 0usize..4usize {
                let port: u16 = unsafe { com_ptr.add(i).read() };
                if port == 0 {
                    continue;
                }
                init_serial(port);
                ports[i] = Some(LockCell::new(port));
            }
        }
        Self { ports }
    }
}

// Initialize a serial communication port at `port`
fn init_serial(port: u16) {
    // Disable interupts
    out_u8(port.saturating_add(1), 0x00);
    // Set the DLAB (Divisor Access Bit) in order to set the divisor
    out_u8(port.saturating_add(3), 0x80);
    // Set divisor to 1 (lo bytes)
    out_u8(port, 1);
    // Set divisor to 1 (hi byte)
    out_u8(port.saturating_add(1), 0);
    // Set 8 data bits, no parity and 1 stop bit (8n1). Also disable DLAB
    out_u8(port.saturating_add(3), 0b00000011);
    // Disable FIFO Buffer state (not present in all processors)
    out_u8(port.saturating_add(2), 0x00);
    // RTS/DTR set
    out_u8(port.saturating_add(4), 0x03);
    // Enable loopback mode in Modem Control Register, in order to test the port
    out_u8(port.saturating_add(4), 0b11110);

    // Wait until we can transmit bytes
    while transmitter_empty(port) == 0 {}
    // Test serial chip (send byte 0x4d = M and check if serial returns the same byte)
    out_u8(port, b'M');

    // Wait until we can read
    while data_ready(port) == 0 {}
    assert!(in_u8(port) == b'M');

    // If the serial is not faulty, set it in normal operation mode
    out_u8(port.saturating_add(4), 0x0f);
}

fn transmitter_empty(port: u16) -> u8 {
    // Check if the Transmitter holding register is empty
    in_u8(port.saturating_add(5)) & 0x20
}

fn write_data(port: u16, value: u8) {
    // Wait until we can transmit bytes
    while transmitter_empty(port) == 0 {}

    out_u8(port, value);
}

// Check data ready bit is set, meaning we can read from the serial port
fn data_ready(port: u16) -> u8 {
    in_u8(port.saturating_add(5)) & 1
}
