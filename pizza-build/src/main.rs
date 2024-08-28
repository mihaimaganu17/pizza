use parse_pe::Pe;
use std::{
    io::{Write, Seek, SeekFrom},
    process::Command,
};
use std::path::Path;

// Actually this is a recommended size and not the maximum possible value
const PXE_MAX_SIZE: u64 = 32 * 1024;

const BOOTLOADER_BASE: usize = 0x7e00;

fn main() {
    // Call nasm to build the bootloader to be executed by the BIOS
    let nasm_build = Command::new("nasm")
        .current_dir("../bootloader/build")
        .args(["-f", "win32", &format!("-Dimage_base={}", BOOTLOADER_BASE), "-o", "utils.obj", "utils.asm"])
        .status()
        .expect("Failed to compile assembly utilites for the bootloader to use");

    // If the build status was not successful stop the building
    if !nasm_build.success() {
        std::process::exit(1);
    }

    // First, build the bootloader
    let build_bootloader = Command::new("cargo")
        .current_dir("../bootloader")
        .args(["build", "--target", "i586-pc-windows-msvc", "--release"])
        .status()
        .expect("Failed to build bootloader!");
    // If the build status was not successful stop the building
    if !build_bootloader.success() {
        std::process::exit(1);
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

    let (image_start, _image_end) = bootloader_pe
        .image_bounds()
        .expect("Failed to get image bounds");

    if image_start != BOOTLOADER_BASE {
        panic!("Base address for bootloader did not match expected");
    }


    // Dump all the sections into the flat bootloader
    bootloader_pe.access_sections(|base, _size, bytes| {
        // Compute the offset into the file
        let flat_offset = u64::try_from(base.saturating_sub(image_start))
            .expect("Cannot convert to u64");
        // Seek to that offset
        flat_file.seek(SeekFrom::Start(flat_offset)).expect("Failed to seek");
        // Write the contents of the section into the flat bootloader file
        flat_file.write(bytes).ok()?;
        Some(())
    });

    // Get the entry point
    let entry_point = bootloader_pe.entry_point();

    let boot_flat = Path::new("pizza.boot");

    // Call nasm to build the bootloader to be executed by the BIOS
    let nasm_build = Command::new("nasm")
        .current_dir("../bootloader/build")
        .args(["-f", "bin", &format!("-Dentry_point={}", entry_point), "-o", boot_flat.to_str().unwrap(), "stage0.asm"])
        .status()
        .expect("Failed to compile bootloader with nasm");

    // If the build status was not successful, stop building
    if !nasm_build.success() {
        std::process::exit(1);
    }

    // Check the size of the bootloader
    let size = std::fs::metadata("../bootloader/build/pizza.boot")
        .expect("Failed to query bootfile metadata").len();

    assert!( size < PXE_MAX_SIZE);

    println!("PXE Remote.0 size: {}", size);

    // Build the kernel
    let build_kernel = Command::new("cargo")
        .current_dir("../kernel")
        .args(["build", "--target", "x86_64-pc-windows-msvc", "--release"])
        .status()
        .expect("Failed to build kernel!");
    // If the build status was not successful stop the building
    if !build_kernel.success() {
        std::process::exit(1);
    }
}
