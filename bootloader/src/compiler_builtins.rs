/// Writes character `value` into the `dest` for `len` times. Actual signature of the function in
/// C has different argument names.
/// Returns the `dest` buffer
#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, value: i32, len: usize) -> *mut u8 {
    // TODO: We should check if the pointer is null. However, this is not C behavioud :D
    let mut idx = 0;
    while idx < len {
        // *dest.offset can also be used
        *dest.add(idx) = value as u8;
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
    let mut idx = 0;

    while idx < len {
        *dst.add(idx) = *src.add(idx);
        idx += 1;
    }

    dst
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
