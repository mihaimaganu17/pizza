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
