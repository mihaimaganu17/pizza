use parse_pe::Pe;
use std::io::Write;

fn main() {
    let bootloader_bytes =
        std::fs::read("../bootloader/target/i586-pc-windows-msvc/release/bootloader.exe")
        .unwrap();

    let bootloader_pe = Pe::parse(&bootloader_bytes).expect("Failed to parse bootloader");

    // Create the desired file where we want to flat our bootloader
    let mut flat_file = std::fs::File::create("build/pizza.flat")
        .expect("Failed to create flatten PE bootloader");

    bootloader_pe.access_sections(|base, size, bytes| {
        // Write the contents of the section into the flat bootloader file
        flat_file.write(bytes).ok()?;
        Some(())
    });

    println!("Entrypoint {:x?}", bootloader_pe.entry_point());
}
