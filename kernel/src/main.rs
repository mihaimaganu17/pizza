#![no_std]
#![no_main]

mod compiler_builtins;
mod mm;
mod tls;

use cpu::x86;
use core::panic::PanicInfo;
use state::BootState;

#[no_mangle]
extern "C" fn entry(boot_state: &'static BootState) {
    // Initialise the current local core storage
    tls::init(boot_state);

    unsafe {
        println!("ID: {:x?}", core!().id());
    }

    let screen = unsafe {
        core::slice::from_raw_parts_mut(0xb8000 as *mut u16, 80 * 25)
    };
    screen.iter_mut().for_each(|x| *x = 0x0f75);

    {
        extern crate alloc;
        let v = alloc::vec![b'\xbb'; 5];
        println!("{:#x?}", v.get(..));
    }
    println!("{:#?}", "TOO MANY BALLS");
    x86::halt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Print the location where the panic occurred
    if let Some(loc) = info.location() {
        println!("System panic: {}:{}", loc.file(), loc.line());
    } else {
        println!("System panic: unknown location");
    }
    // Print the message for the panic
    println!("{:?}", info.message());
    x86::halt()
}

/// Writer for serial
#[repr(C)]
pub struct SerialWriter;

impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(serial) = unsafe { core!().state.serial.lock().as_mut() } {
            serial.write_str(s);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(&mut $crate::SerialWriter, core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(&mut $crate::SerialWriter,
            core::format_args!("{}\n", core::format_args!($($arg)*)));
    };
}
