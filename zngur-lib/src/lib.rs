/// Marker trait for C++ objects
///
/// # Safety
///
/// Only generated C++ objects by zngur should implement this trait.
pub unsafe trait ZngCppObject: Sized {}
// Note on `Sized` bound: For now all C++ objects are Sized. Stack objects are by definition Sized.
// Heap and Ref objects aren't as clear but they are currently represented as 0-sized types since
// extern_types don't exist yet. MaybeUninit<T> requires T: Sized for now. Maybe by the time we
// have extern types, this will change, since that could be the first !Sized referenced with a
// thin-pointer and there'll be more of a reason MaybeUninit would need to support T: ?Sized.

/// Trait for representing C++ objects in the Rust stack
///
/// # Safety
///
/// Only generated objects by zngur should implement this trait.
pub unsafe trait ZngCppStackObject: ZngCppObject {}

/// Trait for invoking the C++ destructor
///
/// # Safety
///
/// Implementer must abide to the documentation specified for the trait methods.
pub unsafe trait ZngCppDestruct: ZngCppObject {
    /// Calls the destructor for the C++ object
    ///
    /// # Safety
    ///
    /// * The object must be properly initialized.
    /// * The only things you can do with `self` after this call is
    ///   reinitialize it with a constructor call or drop it.
    unsafe fn destruct(&mut self);
}
