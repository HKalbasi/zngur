
#[allow(dead_code)]
mod zngur_types {
    pub struct ZngurCppOpaqueBorrowedObject(());

    #[repr(C)]
    pub struct ZngurCppOpaqueOwnedObject {
        data: *mut u8,
        destructor: extern "C" fn(*mut u8),
    }

    impl ZngurCppOpaqueOwnedObject {
        pub unsafe fn new(
            data: *mut u8,
            destructor: extern "C" fn(*mut u8),            
        ) -> Self {
            Self { data, destructor }
        }

        pub fn ptr(&self) -> *mut u8 {
            self.data
        }
    }

    impl Drop for ZngurCppOpaqueOwnedObject {
        fn drop(&mut self) {
            (self.destructor)(self.data)
        }
    }
}

#[allow(unused_imports)]
pub use zngur_types::ZngurCppOpaqueOwnedObject;
#[allow(unused_imports)]
pub use zngur_types::ZngurCppOpaqueBorrowedObject;
const _: [(); 4] = [(); ::std::mem::size_of::<char>()];
const _: [(); 4] = [(); ::std::mem::align_of::<char>()];
const _: () = {
                const fn static_assert_is_copy<T: Copy>() {}
                static_assert_is_copy::<char>();
            };
const _: [(); 1] = [(); ::std::mem::size_of::<bool>()];
const _: [(); 1] = [(); ::std::mem::align_of::<bool>()];
const _: () = {
                const fn static_assert_is_copy<T: Copy>() {}
                static_assert_is_copy::<bool>();
            };
const _: [(); 0] = [(); ::std::mem::size_of::<crate::CharPrinter>()];
const _: [(); 1] = [(); ::std::mem::align_of::<crate::CharPrinter>()];

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_CharPrinter_drop_in_place_s12e24(v: *mut u8) { unsafe {
    ::std::ptr::drop_in_place(v as *mut crate::CharPrinter);
} }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[allow(unused_parens)]
pub extern "C" fn _zngur__crate_CharPrinter_print___x7s13n25m31y32_a4954a65fb(i0: *mut u8, o: *mut u8) { unsafe {
    ::std::ptr::write(o as *mut (),  <crate::CharPrinter>::print::<>((::std::ptr::read(i0 as *mut char)), ));
 } }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[allow(unused_parens)]
pub extern "C" fn _zngur__crate_CharPrinter_is_alphabetic___x7s13n25m39y40_a4954a65fb(i0: *mut u8, o: *mut u8) { unsafe {
    ::std::ptr::write(o as *mut bool,  <crate::CharPrinter>::is_alphabetic::<>((::std::ptr::read(i0 as *mut char)), ));
 } }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[allow(unused_parens)]
pub extern "C" fn _zngur__crate_CharPrinter_to_uppercase___x7s13n25m38y39_a4954a65fb(i0: *mut u8, o: *mut u8) { unsafe {
    ::std::ptr::write(o as *mut char,  <crate::CharPrinter>::to_uppercase::<>((::std::ptr::read(i0 as *mut char)), ));
 } }
const _: [(); 0] = [(); ::std::mem::size_of::<()>()];
const _: [(); 1] = [(); ::std::mem::align_of::<()>()];
const _: () = {
                const fn static_assert_is_copy<T: Copy>() {}
                static_assert_is_copy::<()>();
            };
