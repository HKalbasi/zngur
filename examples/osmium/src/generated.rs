
struct ZngurCppOpaqueObject {
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
}

impl Drop for ZngurCppOpaqueObject {
    fn drop(&mut self) {
        (self.destructor)(self.data)
    }
}
const _: () = assert!(::std::mem::size_of::<()>() == 0);
const _: () = assert!(::std::mem::align_of::<()>() == 1);
const _: () = assert!(::std::mem::size_of::<crate::Reader>() == 0);
const _: () = assert!(::std::mem::align_of::<crate::Reader>() == 1);

#[no_mangle]
pub extern "C" fn __zngur_crate_Reader_drop_in_place_s13e20(v: *mut u8) { unsafe {
    ::std::ptr::drop_in_place(v as *mut crate::Reader);
} }
const _: () = assert!(::std::mem::size_of::<crate::Flags>() == 1);
const _: () = assert!(::std::mem::align_of::<crate::Flags>() == 1);

#[no_mangle]
pub extern "C" fn __zngur_crate_Flags_drop_in_place_s13e19(v: *mut u8) { unsafe {
    ::std::ptr::drop_in_place(v as *mut crate::Flags);
} }

#[no_mangle]
pub extern "C" fn __zngur__crate_Flags_bits___x8s14n20m25y26(i0: *mut u8, o: *mut u8) { unsafe {
    ::std::ptr::write(o as *mut u8, <crate::Flags>::bits::<>(::std::ptr::read(i0 as *mut &crate::Flags), )) } }

extern "C" { fn __zngur_new_blob_store_client_(i0: *mut u8, o: *mut u8); }

pub(crate) fn new_blob_store_client(i0: crate::Flags, ) -> crate::Reader { unsafe {
let mut i0 = ::core::mem::MaybeUninit::new(i0);
let mut r = ::core::mem::MaybeUninit::uninit();
__zngur_new_blob_store_client_(i0.as_mut_ptr() as *mut u8, r.as_mut_ptr() as *mut u8);
r.assume_init()
} }
