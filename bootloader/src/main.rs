#![no_std]
#![no_main]

mod compiler_builtins;

use core::panic::PanicInfo;
use core::arch::asm;

#[no_mangle]
extern "C" fn entry() {
    unsafe {
        // https://wiki.osdev.org/Printing_To_Screen
        let com_ptr: *const u16 = 0x0400 as *const u16;
        for i in 0..4 {
            let port: u16 = unsafe { com_ptr.offset(i).read() };
            if port == 0 {
                continue;
            }
            init_serial(port);
            let value = 0x4du8;
            asm!("out dx, al", in("dx") port, in("al") value);
            //write_data(port, 0x4d);
        }
        core::ptr::write(0xB8000 as *mut u16, 0x0f4d);
        loop {}
        asm!(
            "cli",
            "hlt",
        );
    }
}

// Initialize a serial communication port at `port`
fn init_serial(port: u16) {
    unsafe {
    // Make a pointer from the diven port
    let port_ptr: *mut u8 = port as *mut u16 as *mut u8;
    // Disable interupts
    port_ptr.add(1).write(0x00);
    // Set the DLAB (Divisor Access Bit) in order to set the divisor
    port_ptr.add(3).write(1 << 7);
    // Set divisor to 3 (lo bytes) = 38400 baud rate
    port_ptr.add(0).write(3);
    // Set divisor to 3 (hi byte) = 38400 baud rate
    port_ptr.add(1).write(0);
    // Set 8 data bits, no parity and 1 stop bit. Also disable DLAB
    port_ptr.add(3).write(0b00000011);
    // Enable FIFO Buffer state
    port_ptr.add(2).write(0xC7);
    port_ptr.add(4).write(0x0B);
    // Enable loopback mode in Modem Control Register, in order to test the port
    port_ptr.add(4).write(0b11110);
    // Test serial chip (send bytes 0x4D and check if serial returns the same byte)
    port_ptr.write(0x4d);

    assert!(port_ptr.read() == 0x4d);

    // If the serial is not faulty, set it in normal operation mode
    port_ptr.add(4).write(0xF);
    }
}

fn transmitter_empty(port: u16) -> u8 {
    unsafe {
    // Make a pointer from the given port
    let port_ptr: *mut u8 = port as *mut u16 as *mut u8;
    // Check if the Transmitter holding register is empty
    port_ptr.add(5).read() & 0x20
    }
}

fn write_data(port: u16, value: u8) {
    // Wait until we can transmit bytes
    while transmitter_empty(port) == 0 {}

    unsafe {
        // Make a pointer from the given port
        let port_ptr: *mut u8 = port as *mut u16 as *mut u8;
        port_ptr.write(value);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
