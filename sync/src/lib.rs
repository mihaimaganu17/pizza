#![no_std]
pub mod lockcell;

pub use lockcell::LockCell;

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    #[test]
    fn lock_exclusive_access_and_mutability() {
        let cell = LockCell::new(0xbeef);

        {
            let lock = cell.lock();
            assert!(0xbeef == *lock);
        }
        {
            let mut lock = cell.lock();
            *lock = 0x1ee7;
            assert!(0x1ee7 == *lock);
        }
        {
            let lock = cell.lock();
            assert!(0x1ee7 == *lock);
        }
    }

    #[test]
    fn static_lock_exclusive_access_and_mutability() {
        static CELL_VAR: LockCell<usize> = LockCell::new(0xbeef);

        {
            let lock = CELL_VAR.lock();
            assert!(0xbeef == *lock);
        }
        {
            let mut lock = CELL_VAR.lock();
            *lock = 0x1ee7;
            assert!(0x1ee7 == *lock);
        }
        {
            let lock = CELL_VAR.lock();
            assert!(0x1ee7 == *lock);
        }
    }
}
