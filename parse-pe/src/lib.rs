#![no_std]

mod read;
mod pe;

#[cfg(test)]
mod tests {
    use super::*;
    use read::Reader;
    use pe::Pe;

    #[test]
    fn it_works() {
        let boot_bytes = include_bytes!(
            "../../bootloader/target/i586-pc-windows-msvc/release/bootloader.exe");
        let mut reader = Reader::from(boot_bytes.as_ref());
        let pe = Pe::parse(&mut reader).expect("Failed to parse bootloader");
    }
}
