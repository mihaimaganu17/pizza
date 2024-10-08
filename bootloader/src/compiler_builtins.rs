/// Writes character `value` into the `dest` for `len` times. Actual signature of the function in
/// C has different argument names.
/// Returns the `dest` buffer
#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, value: i32, len: usize) -> *mut u8 {
    // TODO: We should check if the pointer is null. However, this is not C behavioud :D
    let mut idx = 0;
    while idx < len {
        // *dest.offset can also be used
        *dest.offset(idx as isize) = value as u8;
        idx += 1;
    }
    dest
}

/// Dummy defined symbol for the entry point function of a windows binary which uses the
/// /Subsystem:Console compiler environment.
#[no_mangle]
pub unsafe extern "C" fn mainCRTStartup() -> i32 {
    // Notify the user that if it ever need this path of main execution, he has to implement it
    panic!("No mainCRTStartup implementation");
}

/// Copies `len` bytes from memory area `src` to memory area `dst`. If `dst` and `src`.
/// Applications in which `dst` and `src` might overlap should use `memmove` instead.
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: usize) -> *mut u8 {
    if src < dst as *const u8 {
        // copy backwards
        let mut idx = len;
        while idx != 0 {
            idx -= 1;
            *dst.add(idx) = *src.add(idx);
        }
    } else {
        // copy forwards
        let mut idx = 0;
        while idx < len {
            *dst.add(idx) = *src.add(idx);
            idx += 1;
        }
    }

    dst
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, len: usize) -> *mut u8 {
    memcpy(dst, src, len)
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut size = 0usize;
    while *s.add(size) != 0 {
        size = size.saturating_add(1);
    }
    size
}

/// Compared byte string `s1` against byte string `s2`. Both strings are assumed to be `len` bytes
/// long.
#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, len: usize) -> u8 {
    let mut idx = 0;

    // We assume the strings are the same
    let mut diff = 0;

    while idx < len {
        // We mimic the C's behaviour in case we underflow.
        diff = (*s1.add(idx)).wrapping_sub(*s2.add(idx));
        // If out assumption is wrong, we return the difference
        if diff != 0 {
            return diff;
        }

        idx += 1;
    }

    diff
}

/// Divides 2 64-bit unsigned integers returning the integer part of the division.
#[no_mangle]
pub extern "C" fn _aulldiv(a: u64, b: u64) -> u64 {
    a / b
}

/// Divides 2 64-bit unsigned integers, returning the remainder (modulo) of the division.
#[no_mangle]
pub extern "C" fn _aullrem(a: u64, b: u64) -> u64 {
    a % b
}

/// Internal CRT function. Used to handle structured exception frames.
#[no_mangle]
pub extern "C" fn __CxxFrameHandler3() -> *mut u8 {
    panic!("__CxxFrameHandler3 called");
}

#[no_mangle]
pub static _fltused: i32 = 0;
