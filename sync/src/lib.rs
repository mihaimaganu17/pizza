pub mod lockcell;

pub use lockcell::LockCell;

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    #[test]
    fn it_works() {
        let cell = LockCell::new(0xbeef);

        {
            let lock = cell.lock();
            println!("{:x?}", *lock);
        }
    }
}
