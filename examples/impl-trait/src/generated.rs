
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
unsafe extern "C" {
fn _zngur_crate_Greeter_s12_greet(data: *mut u8,  o: *mut u8);
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Greeter_s12(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    o: *mut u8,
) {
    struct Wrapper { 
        value: ZngurCppOpaqueOwnedObject,
    }
    impl crate::Greeter for Wrapper {

        fn greet(&self) -> ::std::string::String { unsafe {
            let data = self.value.ptr();
let mut r = ::core::mem::MaybeUninit::uninit();
_zngur_crate_Greeter_s12_greet(data, r.as_mut_ptr() as *mut u8);
r.assume_init()
        } }

    }
    unsafe { 
        let this = Wrapper {
            value: ZngurCppOpaqueOwnedObject::new(data, destructor),
        };
        let r: Box<dyn crate::Greeter> = Box::new(this);
        std::ptr::write(o as *mut _, r)
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Greeter_s12_borrowed(
    data: *mut u8,
    o: *mut u8,
) {
    struct Wrapper(ZngurCppOpaqueBorrowedObject);
    impl crate::Greeter for Wrapper {

        fn greet(&self) -> ::std::string::String { unsafe {
            let data = ::std::mem::transmute::<_, *mut u8>(self);
let mut r = ::core::mem::MaybeUninit::uninit();
_zngur_crate_Greeter_s12_greet(data, r.as_mut_ptr() as *mut u8);
r.assume_init()
        } }

    }
    unsafe { 
        let this = data as *mut Wrapper;
        let r: &dyn crate::Greeter = &*this;
        std::ptr::write(o as *mut _, r)
    }
}
thread_local! {
            pub static PANIC_PAYLOAD: ::std::cell::Cell<Option<()>> = ::std::cell::Cell::new(None);
        }
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        pub fn _zngur__detect_panic_z7() -> u8 {
            PANIC_PAYLOAD.with(|p| {
                let pp = p.take();
                let r = if pp.is_some() { 1 } else { 0 };
                p.set(pp);
                r
            })
        }

        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        pub fn _zngur__take_panic_z7() {
            PANIC_PAYLOAD.with(|p| {
                p.take();
            })
        }
        

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur__str_to_owned___x7n11m20y21(i0: *mut u8, o: *mut u8) { unsafe {
let e = ::std::panic::catch_unwind(|| {
    ::std::ptr::write(o as *mut ::std::string::String, <str>::to_owned::<>(::std::ptr::read(i0 as *mut &str), ));
});
if let Err(_) = e { PANIC_PAYLOAD.with(|p| p.set(Some(()))) }
 } }
const _: [(); 24] = [(); ::std::mem::size_of::<::std::string::String>()];
const _: [(); 8] = [(); ::std::mem::align_of::<::std::string::String>()];

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur__std_string_String_debug_pretty_s7s11s18e25(v: *mut u8) {
    eprintln!("{:#?}", unsafe { &*(v as *mut ::std::string::String) });
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur__std_string_String_debug_print_s7s11s18e25(v: *mut u8) {
    eprintln!("{:?}", unsafe { &*(v as *mut ::std::string::String) });
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur__std_string_String_drop_in_place_s7s11s18e25(v: *mut u8) { unsafe {
    ::std::ptr::drop_in_place(v as *mut ::std::string::String);
} }
const _: [(); 24] = [(); ::std::mem::size_of::<crate::Person>()];
const _: [(); 8] = [(); ::std::mem::align_of::<crate::Person>()];

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Person_s12(f_name: *mut u8, o: *mut u8) { unsafe {
    ::std::ptr::write(o as *mut _, crate::Person { name: ::std::ptr::read(f_name as *mut ::std::string::String), }) } }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Person_s12_check(i: *mut u8, o: *mut u8) { unsafe {
    *o = matches!(&*(i as *mut &_), crate::Person { .. }) as u8;
} }
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Person_debug_pretty_s12e19(v: *mut u8) {
    eprintln!("{:#?}", unsafe { &*(v as *mut crate::Person) });
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Person_debug_print_s12e19(v: *mut u8) {
    eprintln!("{:?}", unsafe { &*(v as *mut crate::Person) });
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Person_drop_in_place_s12e19(v: *mut u8) { unsafe {
    ::std::ptr::drop_in_place(v as *mut crate::Person);
} }
const _: [(); 4] = [(); ::std::mem::size_of::<crate::Robot>()];
const _: [(); 4] = [(); ::std::mem::align_of::<crate::Robot>()];

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Robot_s12(f_id: *mut u8, o: *mut u8) { unsafe {
    ::std::ptr::write(o as *mut _, crate::Robot { id: ::std::ptr::read(f_id as *mut u32), }) } }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Robot_s12_check(i: *mut u8, o: *mut u8) { unsafe {
    *o = matches!(&*(i as *mut &_), crate::Robot { .. }) as u8;
} }
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Robot_debug_pretty_s12e18(v: *mut u8) {
    eprintln!("{:#?}", unsafe { &*(v as *mut crate::Robot) });
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Robot_debug_print_s12e18(v: *mut u8) {
    eprintln!("{:?}", unsafe { &*(v as *mut crate::Robot) });
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_Robot_drop_in_place_s12e18(v: *mut u8) { unsafe {
    ::std::ptr::drop_in_place(v as *mut crate::Robot);
} }
const _: [(); 16] = [(); ::std::mem::size_of::<Box<dyn crate::Greeter>>()];
const _: [(); 8] = [(); ::std::mem::align_of::<Box<dyn crate::Greeter>>()];

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_Box_dyncrate_Greeter__drop_in_place_x10s19y27e28(v: *mut u8) { unsafe {
    ::std::ptr::drop_in_place(v as *mut Box<dyn crate::Greeter>);
} }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur__Box_dyncrate_Greeter__greet___x7x11s20y28n29m35y36(i0: *mut u8, o: *mut u8) { unsafe {
let e = ::std::panic::catch_unwind(|| {
    ::std::ptr::write(o as *mut ::std::string::String, <Box<dyn crate::Greeter>>::greet::<>(::std::ptr::read(i0 as *mut &Box<dyn crate::Greeter>), ));
});
if let Err(_) = e { PANIC_PAYLOAD.with(|p| p.set(Some(()))) }
 } }
const _: [(); 0] = [(); ::std::mem::size_of::<()>()];
const _: [(); 1] = [(); ::std::mem::align_of::<()>()];
const _: () = {
                const fn static_assert_is_copy<T: Copy>() {}
                static_assert_is_copy::<()>();
            };

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_create_greeter_by_type_s12(i0: *mut u8, i1: *mut u8, i2: *mut u8, o: *mut u8) { unsafe {
let e = ::std::panic::catch_unwind(|| {
    ::std::ptr::write(o as *mut Box<dyn crate::Greeter>, crate::create_greeter_by_type(::std::ptr::read(i0 as *mut bool), ::std::ptr::read(i1 as *mut ::std::string::String), ::std::ptr::read(i2 as *mut u32), ));
});
if let Err(_) = e { PANIC_PAYLOAD.with(|p| p.set(Some(()))) }
 } }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_create_person_s12(i0: *mut u8, o: *mut u8) { unsafe {
let e = ::std::panic::catch_unwind(|| {
    ::std::ptr::write(o as *mut Box<dyn crate::Greeter>, Box::new(crate::create_person(::std::ptr::read(i0 as *mut ::std::string::String), )));
});
if let Err(_) = e { PANIC_PAYLOAD.with(|p| p.set(Some(()))) }
 } }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_create_robot_s12(i0: *mut u8, o: *mut u8) { unsafe {
let e = ::std::panic::catch_unwind(|| {
    ::std::ptr::write(o as *mut Box<dyn crate::Greeter>, Box::new(crate::create_robot(::std::ptr::read(i0 as *mut u32), )));
});
if let Err(_) = e { PANIC_PAYLOAD.with(|p| p.set(Some(()))) }
 } }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_print_greeting_person_s12(i0: *mut u8, o: *mut u8) { unsafe {
let e = ::std::panic::catch_unwind(|| {
    ::std::ptr::write(o as *mut (), crate::print_greeting_person(::std::ptr::read(i0 as *mut crate::Person), ));
});
if let Err(_) = e { PANIC_PAYLOAD.with(|p| p.set(Some(()))) }
 } }

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn _zngur_crate_print_greeting_robot_s12(i0: *mut u8, o: *mut u8) { unsafe {
let e = ::std::panic::catch_unwind(|| {
    ::std::ptr::write(o as *mut (), crate::print_greeting_robot(::std::ptr::read(i0 as *mut crate::Robot), ));
});
if let Err(_) = e { PANIC_PAYLOAD.with(|p| p.set(Some(()))) }
 } }
