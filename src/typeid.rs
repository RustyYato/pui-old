//! A type based identifier that uses a unique type parameter
//! to assert uniqueness
//!
//! A given type parameter may only be associated with a single instance of
//! [`Type<T>`](Type), and this can be achieved in two ways. One is through a
//! runtime check that tracks the lifetime of the type, and the other is by creating
//! a unique type every you create an instance of [`Type<T>`](Type).
//!
//! The [`make_typeid`](make_typeid) macro uses the former approach. It defines a new type that
//! tracks it's lifetime and only allows a new `Type<T>` on non-overlapping lifetimes.
//!
//! The [`make_anon_typeid`](make_anon_typeid) macro uses the later approach. It creates a new
//! unique instance of `Type` every time it is called.

use core::{fmt, marker::PhantomData};

mod macros;

/// A type based identifier
///
/// This handle is guaranteed to be zero-sized and 1 byte aligned
///
/// see module docs for details
#[repr(transparent)]
pub struct Type<T>(TypeHandle<T>, T);

/// A handle to to a [`Type`](Type) identifier
///
/// This handle is guaranteed to be zero-sized and 1 byte aligned
///
/// see module docs for details
#[repr(C)]
pub struct TypeHandle<T>(PhantomData<crate::Invariant<T>>);

impl<T> crate::Trivial for TypeHandle<T> {
    const INSTANCE: Self = Self(PhantomData);
}

impl<T> Type<T> {
    /// Create a new `Type<T>` handle
    ///
    /// # Panic
    ///
    /// If the given type is not 0-sized and 1-byte aligned, then this function panics
    ///
    /// # Safety
    ///
    /// There must be no other instances of `Type<T>` in the current process
    #[inline]
    pub const unsafe fn new_unchecked(value: T, handle: TypeHandle<T>) -> Self {
        [()][core::mem::size_of::<T>()];
        [()][core::mem::align_of::<T>().wrapping_sub(1)];
        Self(handle, value)
    }

    /// get a handle with the same type parameter
    #[inline]
    pub const fn handle(&self) -> TypeHandle<T> { TypeHandle(PhantomData) }
}

impl<T> TypeHandle<T> {
    /// Create a new handle
    #[inline]
    pub const fn new() -> Self { Self(PhantomData) }
}

unsafe impl<T> crate::Handle for TypeHandle<T> {}
unsafe impl<T> crate::Identifier for Type<T> {
    type Handle = TypeHandle<T>;

    #[inline]
    fn handle(&self) -> TypeHandle<T> { self.handle() }

    #[inline]
    fn owns(&self, _: &Self::Handle) -> bool { true }
}

// common traits

impl<T> Copy for TypeHandle<T> {}
impl<T> Clone for TypeHandle<T> {
    #[inline]
    fn clone(&self) -> Self { *self }
}

impl<T> fmt::Debug for Type<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Type({})", core::any::type_name::<T>()) }
}

impl<T> fmt::Debug for TypeHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeHandle({})", core::any::type_name::<T>())
    }
}

impl<T> Eq for Type<T> {}
impl<T> PartialEq for Type<T> {
    #[inline]
    fn eq(&self, _: &Self) -> bool { true }
}

impl<T> PartialOrd for Type<T> {
    #[inline]
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> { Some(core::cmp::Ordering::Equal) }
}

impl<T> Ord for Type<T> {
    #[inline]
    fn cmp(&self, _: &Self) -> core::cmp::Ordering { core::cmp::Ordering::Equal }
}

impl<T> core::hash::Hash for Type<T> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}
}

impl<T> Eq for TypeHandle<T> {}
impl<T> PartialEq for TypeHandle<T> {
    #[inline]
    fn eq(&self, _: &Self) -> bool { true }
}

impl<T> PartialOrd for TypeHandle<T> {
    #[inline]
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> { Some(core::cmp::Ordering::Equal) }
}

impl<T> Ord for TypeHandle<T> {
    #[inline]
    fn cmp(&self, _: &Self) -> core::cmp::Ordering { core::cmp::Ordering::Equal }
}

impl<T> core::hash::Hash for TypeHandle<T> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}
}
