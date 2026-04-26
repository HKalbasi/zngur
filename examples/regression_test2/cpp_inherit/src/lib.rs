use zngur_lib::ZngCppStackObject;

#[repr(C)]
pub struct CppInherit<Base: ZngCppStackObject, T> {
    pub base: Base,
    pub inner: T,
}

impl<Base: ZngCppStackObject, T> CppInherit<Base, T> {
    pub fn new(inner: T, base: Base) -> Self {
        CppInherit { base, inner }
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
}
