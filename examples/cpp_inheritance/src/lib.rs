use std::mem::MaybeUninit;

pub unsafe trait Cpp {}

#[repr(C)]
pub struct CppInherit<Base, T> {
    pub base: Base,
    pub inner: T,
}

impl<Base, T> CppInherit<Base, T> {
    pub fn new_with<F>(inner: T, base_init: F) -> Self
    where
        F: FnOnce(&mut Base),
    {
        let mut this = CppInherit {
            // SAFETY: The base class is just a buffer at this point and we properly initialize it below
            base: unsafe { MaybeUninit::<Base>::uninit().assume_init() },
            inner,
        };
        base_init(&mut this.base);
        this
    }

    pub fn base(&self) -> &Base {
        &self.base
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub unsafe fn inner_from_raw_mut(this: *mut Self) -> *mut T {
        unsafe { &mut (*this).inner }
    }
}
