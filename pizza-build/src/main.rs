use parse_pe::Pe;
use std::{
    io::Write,
    process::Command,
};

fn main() {
    // First, build the bootloader
    let build_bootloader = Command::new("cargo")
        .current_dir("../bootloader")
        .args(["build", "--target", "i586-pc-windows-msvc"])
        .output()
        .expect("Failed to build bootloader!");
    // If the build status was not successful, print the error
    if !build_bootloader.status.success() {
        println!("Bootloader build error: {:?}", String::from_utf8(build_bootloader.stderr));
    }

    // Get the bytes of the bootloader that we've built above
    let bootloader_bytes =
        std::fs::read("../bootloader/target/i586-pc-windows-msvc/release/bootloader.exe")
        .unwrap();

    // Parse the PE
    let bootloader_pe = Pe::parse(&bootloader_bytes).expect("Failed to parse bootloader");

    // Create the desired file where we want to flat our bootloader
    let mut flat_file = std::fs::File::create("../bootloader/build/pizza.flat")
        .expect("Failed to create flatten PE bootloader");

    // Dump all the sections into the flat bootloader
    bootloader_pe.access_sections(|base, size, bytes| {
        // Write the contents of the section into the flat bootloader file
        flat_file.write(bytes).ok()?;
        Some(())
    });

    // Get the entry point
    let entry_point = bootloader_pe.entry_point();

    // Call nasm to build the bootloader to be executed by the BIOS
    let nasm_build = Command::new("nasm")
        .current_dir("../bootloader/build")
        .args(["-f", "bin", &format!("-Dentry_point={}", entry_point), "-o", "pizza.boot", "stage0.asm"])
        .output()
        .expect("Failed to compile bootloader with nasm");

    // If the build status was not successful, print the error
    if !nasm_build.status.success() {
        println!("Bootloader nasm compile error: {:?}", String::from_utf8(nasm_build.stderr));
    }

    // Copy the new bootloader where the PXE server can access it
}
