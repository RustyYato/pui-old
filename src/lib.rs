#![no_std]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![cfg_attr(doc, feature(doc_cfg))]
#![cfg_attr(feature = "nightly", feature(cfg_target_has_atomic))]

//! A set of process unique identifiers that can be used to
//! identifry types with minimal overhead within a single process
//!
//! see the [`Identifier`](crate::Identifier) trait for details
//!
//! ### features
//!
//! `std` (default) - if you have the `std` feature on, it will supercede the `alloc` feature.
//!     This allows you to use:
//!      * `std` types to implement various traits, for example `Box<I>` will implemnt `Identifier` `I`
//!      * `thread_local` types (from the `*_tl`)
//!      * `make_global_reuse` (this requires internal locking using a `Mutex`)
//!
//! `alloc` - this allows you to use without pulling in all of `std`:
//!      * `alloc` types to implement various traits, for example `Box<I>` will implemnt `Identifier` `I`
//!
//! `nightly` -  this allows you to use:
//!      * atomics on `no_std` targets that don't support 64-bit atomics

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[doc(hidden)]
pub mod macros;

pub mod runtime;
pub mod scoped;
pub mod typeid;
#[cfg(any(feature = "std", doc))]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
pub mod typeid_tl;

#[cfg(all(feature = "test", feature = "std"))]
#[doc(hidden)]
pub mod test_setup;

pub use macros::Scalar;

struct Invariant<T: ?Sized>(fn() -> *mut T);
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThreadLocal(*mut ());

/// An `Identifier` is a process unique identifier
///
/// you are guaranteed that two instances of this identifier will *never* compare equal
/// You can also get a cheap handle to the identifier, which you can use to mark other types
/// as logically owned by the identifier.
///
/// For example, this pattern is sound
///
/// ```rust
/// use pui::Identifier;
/// use std::cell::UnsafeCell;
///
/// struct Owner<I> {
///     ident: I,
/// }
///
/// struct Handle<H, T: ?Sized> {
///     handle: H,
///     value: UnsafeCell<T>,
/// }
///
/// impl<H, T> Handle<H, T> {
///     pub fn new(handle: H, value: T) -> Self {
///         Self { handle, value: UnsafeCell::new(value) }
///     }
/// }
///
/// impl<I> Owner<I> {
///     pub fn new(ident: I) -> Self {
///         Self { ident }
///     }
/// }
///
/// impl<I: Identifier> Owner<I> {
///     pub fn read<'a, T: ?Sized>(&'a self, handle: &'a Handle<I::Handle, T>) -> &'a T {
///         assert!(self.ident.owns(&handle.handle));
///         
///         // This is safe because `ident` owns the `handle`, which means that `self`
///         // is the only `Owner` that could shared access the underlying value
///         // This is because:
///         //  * the `Owner` owns the `Identifier`
///         //  * when we read/write, we bind the lifetime of `self` and `Handle` to the lifetime of
///         //      the output reference
///         //  * we have shared access to `*self`
///         
///         unsafe { &*handle.value.get() }
///     }
///
///     pub fn write<'a, T: ?Sized>(&'a mut self, handle: &'a Handle<I::Handle, T>) -> &'a mut T {
///         assert!(self.ident.owns(&handle.handle));
///         
///         // This is safe because `ident` owns the `handle`, which means that `self`
///         // is the only `Owner` that could exclusive access the underlying value
///         // This is because:
///         //  * the `Owner` owns the `Identifier`
///         //  * when we read/write, we bind the lifetime of `self` and `Handle` to the lifetime of
///         //      the output reference
///         //  * we have exclusive access to `*self`
///         
///         unsafe { &mut *handle.value.get() }
///     }
/// }
/// ```
///
/// # Safety
///
/// * Calling `Self::owns` with any handle acquired from `Self::handle` must return true
/// * `Self::Handle` cannot change it's equality function with respect to `Self::owns`
///     * i.e. if two handles compare equal now, they will always compare equal
///         if they are not mutated via an exclusive reference.
/// * If two handles compare equal, then `Identifier::owns` must act the same for both of them
///     * i.e. it must return false for both handles, or it must return true for both handles
/// * The following function must never panic, no matter what inputs are passed in
///
/// ```rust
/// # use pui::Identifier;
/// fn check_identifier_invariant<I: Identifier>(a: I, b: I) {
///     assert!(a != b);
///     assert!(a.handle() != b.handle());
/// }
/// ```
pub unsafe trait Identifier: Eq {
    /// A handle which can be used to mark other types
    type Handle: Clone + Eq;

    /// Create a handle that this identifier owns
    fn handle(&self) -> Self::Handle;

    /// Check the current identifier owns the given handle
    fn owns(&self, handle: &Self::Handle) -> bool;
}

/// A zero-sized, 1 byte aligned type that has no validity (language) invariants or safety (library) invariants
pub unsafe trait Trivial {}

unsafe impl<I: Identifier + ?Sized> Identifier for &mut I {
    type Handle = I::Handle;

    #[inline]
    fn handle(&self) -> Self::Handle {
        I::handle(self)
    }

    #[inline]
    fn owns(&self, handle: &Self::Handle) -> bool {
        I::owns(self, handle)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
unsafe impl<I: Identifier + ?Sized> Identifier for std::boxed::Box<I> {
    type Handle = I::Handle;

    #[inline]
    fn handle(&self) -> Self::Handle {
        I::handle(self)
    }

    #[inline]
    fn owns(&self, handle: &Self::Handle) -> bool {
        I::owns(self, handle)
    }
}
