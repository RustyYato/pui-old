//! A type based identifier that uses a unique type parameter
//! to assert uniqueness
//!
//! A given type parameter may only be associated with a single instance of
//! [`Type<T>`](Type) in the current thread, this is achieved by a runtime
//! check that tracks the lifetime of the type. This is done via the
//! [`make_typeid_tl`](make_typeid_tl) macro. It defines a new type that
//! tracks it's lifetime in the current thread and only allows a new `Type<T>`
//! on non-overlapping lifetimes.

use core::fmt;
use core::marker::PhantomData;

mod macros;

/// A type thread local based identifier
///
/// This handle is guaranteed to be zero-sized and 1 byte aligned
///
/// see module docs for details
#[repr(transparent)]
pub struct Type<T>(TypeHandle<T>, T);

/// A thread local handle to to a [`Type`](Type) identifier
///
/// This handle is guaranteed to be zero-sized and 1 byte aligned
///
/// see module docs for details
pub struct TypeHandle<T>(PhantomData<(crate::Invariant<T>, crate::ThreadLocal)>);

impl<T> Type<T> {
    /// Create a new `Type<T>` handle
    ///
    /// # Panic
    ///
    /// If the given type is not 0-sized and 1-byte aligned, then this function panics
    ///
    /// # Safety
    ///
    /// There must be no other instances of `Type<T>` in the current thread
    pub const unsafe fn new_unchecked(value: T, handle: TypeHandle<T>) -> Self {
        [()][core::mem::size_of::<T>()];
        [()][core::mem::align_of::<T>().wrapping_sub(1)];
        Self(handle, value)
    }

    /// get a handle with the same type parameter
    #[inline]
    pub const fn handle(&self) -> TypeHandle<T> {
        TypeHandle(PhantomData)
    }
}

impl<T> TypeHandle<T> {
    /// Create a new handle
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

unsafe impl<T> crate::Identifier for Type<T> {
    type Handle = TypeHandle<T>;

    #[inline]
    fn handle(&self) -> TypeHandle<T> {
        self.handle()
    }

    fn owns(&self, _: &Self::Handle) -> bool {
        true
    }
}

// common traits

impl<T> Copy for TypeHandle<T> {}
impl<T> Clone for TypeHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Type<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type({})", core::any::type_name::<T>())
    }
}

impl<T> fmt::Debug for TypeHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeHandle({})", core::any::type_name::<T>())
    }
}

impl<T> Eq for Type<T> {}
impl<T> PartialEq for Type<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> PartialOrd for Type<T> {
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> {
        Some(core::cmp::Ordering::Equal)
    }
}

impl<T> Ord for Type<T> {
    fn cmp(&self, _: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}

impl<T> core::hash::Hash for Type<T> {
    fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}
}

impl<T> Eq for TypeHandle<T> {}
impl<T> PartialEq for TypeHandle<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> PartialOrd for TypeHandle<T> {
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> {
        Some(core::cmp::Ordering::Equal)
    }
}

impl<T> Ord for TypeHandle<T> {
    fn cmp(&self, _: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}

impl<T> core::hash::Hash for TypeHandle<T> {
    fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}
}
