/// Writes character `value` into the `dest` for `len` times. Actual signature of the function in
/// C has different argument names.
/// Returns the `dest` buffer
#[no_mangle]
pub extern "C" fn memset(dest: *mut u8, value: i32, len: usize) -> *mut u8 {
    dest
}
