#![no_std]
//! A serial port implementation as described in https://wiki.osdev.org/Serial_Ports
use sync::LockCell;
use cpu::x86::{out_u8, in_u8};

pub struct Serial {
    ports: [Option<u16>; 4],
}

/// Provides mutually exclusive global access to serial ports from COM1 to COM4
static SERIAL: LockCell<Serial> = LockCell::new( Serial { ports: [None; 4] });

impl Serial {
    /// Initialize all found serial ports
    pub fn init() {
        // Lock the ports, such that no one can access them
        let mut serial = SERIAL.lock();
        // Go to the known address of where the COM port addresses are stored
        let com_ptr: *const u16 = 0x0400 as *const u16;

        for (id, port) in serial.ports.iter_mut().enumerate() {
            // Go to the `i`th serial port
            let port_addr: u16 = unsafe { com_ptr.add(id).read() };
            // If null, ignore
            if port_addr == 0 {
                continue;
            }
            // Initialize the port
            init_serial(port_addr);
            *port = Some(port_addr);
        }
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
    write_data(port, b'M');

    // Wait until we can read
    while data_ready(port) == 0 {}
    // Test we got the same byte
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

    // Write value to port
    out_u8(port, value);
}

// Check data ready bit is set, meaning we can read from the serial port
fn data_ready(port: u16) -> u8 {
    in_u8(port.saturating_add(5)) & 1
}

// Broadcast write `value` to all known and initialized serial ports
fn write(port: u16, value: u8) {
    // Write a CR prior to all LFs
    if value == b'\n' { write_data(port, b'\r'); }
    // Write the actual byte
    write_data(port, value);
}

/// Broadcast write `bytes` to all known and initialized serial ports
pub fn write_bytes(bytes: &[u8]) {
    let serial = SERIAL.lock();

    for value in bytes {
        for maybe_port in &serial.ports {
            if let Some(port) = maybe_port {
                write(*port, *value);
            }
        }
    }
}

/// Broadcast write `text` to all known and initialized serial ports
pub fn write_str(text: &str) {
    write_bytes(text.as_bytes());
}
