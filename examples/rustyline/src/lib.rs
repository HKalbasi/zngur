use std::ffi::CStr;

mod generated;

pub fn rust_str<'a>(c: *const i8) -> &'a str {
    let x = unsafe { CStr::from_ptr(c) };
    unsafe { core::str::from_utf8_unchecked(x.to_bytes()) }
}
