//! Use lifetimes to guarantee unique identifiers.
//!
//! Two scoped identifiers will never have the same lifetime id, and so it checks
//! at *compile time* that they are indeed different identifiers.
//!
//! There are two main ways to create a scoped identifier, either you can use
//! `Scoped::with` and pass it a callback that will be called with a new scoped identifier,
//! or you can use `pui::make_scoped` to create a scoped identifier in the current scope.
//!
//! ```
//! fn pass() {
//!     # use pui::scoped::Scoped;
//!     Scoped::with(|foo| {
//!         let _foo_handle = foo.handle();
//!     })
//! }
//! ```
//! ```
//! fn pass() {
//!     pui::make_scoped!(foo);
//!     let _foo_handle = foo.handle();
//! }
//! ```
//! Note: you cannot intermix different scoped identifiers declared in the same scope
//!
//! ```compile_fail
//! fn fail() {
//!     pui::make_scoped!(foo);
//!     pui::make_scoped!(bar);
//!     assert_eq!(foo, bar);
//! }
//! ```
//!

use core::marker::PhantomData;

/// Create a new scoped identifier with the provided name
///
/// this identifier may be used until the end of the scope
///
/// for example, this works:
/// ```
/// fn pass() {
///     pui::make_scoped!(foo);
///     let _foo_handle = foo.handle();
/// }
/// ```
///
/// but this fails
/// ```compile_fail
/// fn fail() {
///     {
///         pui::make_scoped!(foo);
///     }
///     let _foo_handle = foo.handle();
/// }
/// ```
///
/// Note: you cannot intermix different scoped identifiers declared in the same scope
///
/// ```compile_fail
/// fn fail() {
///     pui::make_scoped!(foo);
///     pui::make_scoped!(bar);
///     assert_eq!(foo, bar);
/// }
/// ```
///
#[macro_export]
macro_rules! make_scoped {
    // credit: [`generativity`](crates.io/crates/generativity), showed me how to create unique
    // lifetimes with macros!
    ($ident:ident) => {
        let mut inv = $crate::scoped::ScopedHandle::new();
        let $ident = unsafe { $crate::scoped::Scoped::new_unchecked(inv) };
        let _assert_unique_lifetime = $crate::scoped::AssertUniqueLifetime(&mut inv);
    };
}

/// A scoped identifier
///
/// This identifier is guaranteed to be zero-sized and 1 byte aligned
///
/// see module docs for details
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Scoped<'id>(ScopedHandle<'id>);

/// A handle to to a [`Scoped`](Scoped) identifier
///
/// This handle is guaranteed to be zero-sized and 1 byte aligned
///
/// see module docs for details
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopedHandle<'id>(PhantomData<crate::Invariant<&'id ()>>);

impl crate::Trivial for ScopedHandle<'_> {
    const INSTANCE: Self = Self(PhantomData);
}

impl<'id> Scoped<'id> {
    #[inline]
    /// Use a callback to to create a scoped identifier
    pub fn with<R, F: FnOnce(Scoped<'_>) -> R>(callback: F) -> R {
        // # Safety for `Identifier`
        //
        // Because there is a higher rank lifetime bound, no other lifetimes will match the given `Scoped<'id>`
        // so `Scoped<'id>` is guaranteed to be unique at compile time.

        callback(unsafe { Self::new_unchecked(ScopedHandle::new()) })
    }

    /// Create a new scoped identifier without checking if it is indeed unique
    ///
    /// You shouldn't use this function directly, instead use [`make_scoped`](make_scoped)
    #[inline]
    pub const unsafe fn new_unchecked(handle: ScopedHandle<'id>) -> Self { Self(handle) }

    /// get a handle with the same lifetime id
    #[inline]
    pub const fn handle(&self) -> ScopedHandle<'id> { ScopedHandle::new() }
}

impl<'id> ScopedHandle<'id> {
    /// Create a new handle
    #[inline]
    pub const fn new() -> Self { Self(PhantomData) }
}

#[doc(hidden)]
pub struct AssertUniqueLifetime<'id>(pub &'id mut ScopedHandle<'id>);

impl Drop for AssertUniqueLifetime<'_> {
    #[inline]
    fn drop(&mut self) {}
}

unsafe impl crate::Handle for ScopedHandle<'_> {}
unsafe impl<'id> crate::Identifier for Scoped<'id> {
    type Handle = ScopedHandle<'id>;

    #[inline]
    fn handle(&self) -> ScopedHandle<'id> { self.handle() }

    #[inline]
    fn owns(&self, _: &Self::Handle) -> bool { true }
}
