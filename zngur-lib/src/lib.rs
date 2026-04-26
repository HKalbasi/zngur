use core::mem::MaybeUninit;

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
pub unsafe trait ZngCppStackObject: ZngCppObject {
    /// Default constructs the C++ stack-allocated object using the the C++ default constructor
    fn new() -> Self
    where
        Self: ZngCppDefaultConstruct,
    {
        let mut this: MaybeUninit<Self> = MaybeUninit::uninit();
        // SAFETY: C++ objects can be uninitialized for the default constructor.
        unsafe { ZngCppDefaultConstruct::construct(&mut this) };
        // SAFETY: The inner type is now fully constructed.
        unsafe { this.assume_init() }
    }
}

/// Trait for invoking the C++ default constructor on an uninitialized object.
///
/// # Safety
///
/// Implementer must abide to the documentation specified for the trait methods.
pub unsafe trait ZngCppDefaultConstruct: ZngCppObject {
    /// Runs the C++ default constructor on the uninitialized object
    ///
    /// # Safety
    ///
    /// The object must be uninitialized at this point such as by creating
    /// a fresh new object or by running the destructor on `self`
    unsafe fn construct(this: &mut MaybeUninit<Self>);
}

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
