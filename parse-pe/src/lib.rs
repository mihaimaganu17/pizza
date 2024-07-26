#![no_std]

mod pe;

#[cfg(test)]
mod tests {
    use super::*;
    use read_me::Reader;
    use pe::Pe;

    extern crate std;

    #[test]
    fn it_works() {
        let boot_bytes = include_bytes!(
            "../../bootloader/target/i586-pc-windows-msvc/release/bootloader.exe");
        let pe = Pe::parse(boot_bytes).expect("Failed to parse bootloader");

        for (i, sh) in pe.section_headers().enumerate() {
            if i == 0 {
                assert!(&sh.name == b".text\0\0\0");
            }
        }

        pe.access_sections(|section_header, bytes| {
            std::println!("Section {:#?} has len {}", section_header, bytes.len());
            Some(())
        });
    }
}
