use std::ffi::CStr;

use rustyline::DefaultEditor;

mod generated;

pub fn build_editor() -> ::rustyline::DefaultEditor {
    DefaultEditor::new().unwrap()
}

pub fn rust_str<'a>(c: *const i8) -> &'a str {
    let x = unsafe { CStr::from_ptr(c) };
    unsafe { core::str::from_utf8_unchecked(x.to_bytes()) }
}

fn as_ptr(k: &str) -> *const u8 {
    k.as_ptr()
}

fn len(k: &str) -> usize {
    k.len()
}

fn foo() {
    // use ::rustyline::DefaultEditor;
    // let mut k = DefaultEditor::new().unwrap();
    // let f = k.add_history_entry(">> ");
    // let f = f.unwrap_err();
}
