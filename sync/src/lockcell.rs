//! Module defining and implementing a `LockCell` wrapper over a type in order to obtain exclusive
//! access to that type.
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

/// Provides immutable and mutable exclusive access to the wrapped value. It is implementation is
/// based on the ticket lock, while the inner value is accessed through an `UnsafeCell`.
#[repr(C)]
pub struct LockCell<T: ?Sized> {
    serving: AtomicU32,
    release: AtomicU32,
    inner: UnsafeCell<T>,
}

impl<T> LockCell<T> {
    /// Create a new cell from `value`
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            serving: AtomicU32::new(0),
            release: AtomicU32::new(0),
        }
    }
}

impl<T: ?Sized> LockCell<T> {
    /// Get exclusive access to the underlying inner `UnsafeCell`
    pub fn lock(&self) -> LockCellGuard<'_, T> {
        // Get the current ticket and increment for the next interation
        let ticket = self.serving.fetch_add(1, Ordering::SeqCst);

        // While the current ticket is not the same as the released, keep blocking
        while ticket != self.release.load(Ordering::SeqCst) {
            // Send a machine instruction to signal a running busy-wait spin-loop
            core::hint::spin_loop();
        }

        // If we are here, we have exclusive access
        LockCellGuard {
            lock_cell: self
        }
    }
}

unsafe impl<T: ?Sized> Sync for LockCell<T> {}

pub struct LockCellGuard<'a, T: ?Sized> {
    lock_cell: &'a LockCell<T>,
}

impl<'a, T: ?Sized> Drop for LockCellGuard<'a, T> {
    fn drop(&mut self) {
        // We incread the `release` ticket, such that the next [`lock`] call from `LockCell` is not
        // blocking
        self.lock_cell.release.fetch_add(1, Ordering::SeqCst);
    }
}

impl<'a, T: ?Sized> Deref for LockCellGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock_cell.inner.get() }
    }
}

impl<'a, T> DerefMut for LockCellGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
       unsafe { &mut *self.lock_cell.inner.get() }
    }
}

