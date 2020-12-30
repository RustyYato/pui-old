//! A runtime checked identifier
//!
//! It uses an id allocator (usually a simple counter) to generate new ids,
//! then uses those ids to verifiy it's identity. You can either use the `Global`
//! allocator, which uses the largest unsigned non-zero integer possible (up to 64-bits),
//! or you can create your own id allocator using the [`make_counter`](make_counter)
//!
//! ```
//! type BackingScalar = [u8; 3];
//! # #[cfg(feature = "atomic")]
//! pui::make_counter! {
//!     pub type MyCustomIdAllocator = BackingScalar;
//! }
//! ```
//!
// You can use any primitive integer type and their non-zero variants

mod macros;
mod pool;
pub use pool::*;

/// an opaque runtime id
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeId<T>(T);

impl<T> RuntimeId<T> {
    fn into_inner(self) -> T { self.0 }

    /// The underlying id
    pub fn get(&self) -> &T {
        // This is safe becuase the interface of `Pool*` demands that you must
        // either store a `RuntimeId`, return it, or drop it.
        //
        // In particular, the underlying value is useless because you cannot
        // convert it back to a `RuntimeId` outside of this module. This means
        // that all `RuntimeId`s must contain a unique id, and so `Pool*::take`
        // will return a unique id or `None`
        &self.0
    }
}

/// A counter that allocates new ids
///
/// # Safety
///
/// two equal ids may never exist together on the same thread
///
/// This implies that `Counter::*next()` may never repeat an id on the same thread,
/// and if it can repeat itself on different threads then `Self: !Send + !Sync`.
pub unsafe trait Counter: Copy + Eq {
    /// Get the next id, panics if there are no next ids
    fn next() -> Self;

    /// Try to get the next id, returns `None` if there are no next ids
    fn try_next() -> Option<Self>;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "atomic")] {
        macro_rules! make_global {
            ($(#[$meta:meta])*) => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "nightly")] {
                        cfg_if::cfg_if! {
                            if #[cfg(target_has_atomic = "64")] {
                                crate::make_counter! {
                                    $(#[$meta])*
                                    pub type Global = core::num::NonZeroU64;
                                }
                            } else if #[cfg(target_has_atomic = "32")]{
                                crate::make_counter! {
                                    $(#[$meta])*
                                    pub type Global = core::num::NonZeroU32;
                                }
                            } else if #[cfg(target_has_atomic = "16")]{
                                crate::make_counter! {
                                    $(#[$meta])*
                                    pub type Global = core::num::NonZeroU16;
                                }
                            } else {
                                crate::make_counter! {
                                    $(#[$meta])*
                                    pub type Global = core::num::NonZeroU8;
                                }
                            }
                        }
                    } else {
                        crate::make_counter! {
                            $(#[$meta])*
                            pub type Global = core::num::NonZeroU64;
                        }
                    }
                }
            };
        }

        make_global! {
            /// A gobal allocator for runtime ids (not to be confused with a memory allocator)
            ///
            /// This can be used with [`Runtime`](super::runtime::Runtime) to easily
            /// create a new [`Runtime`](super::runtime::Runtime) [`Identifier`](super::Identifier)
            #[derive(Debug, PartialOrd, Ord, Hash)]
        }

        /// A runtime checked identifier
        ///
        /// This uses a runtime id to verify it's identity, this id is provided
        /// by the [`Counter`](Counter) trait, and ids may be reused via the [`PoolMut<C>`](PoolMut)
        pub struct Runtime<C = Global, P: PoolMut<C> = ()> {
            id: C,
            pool: P,
        }

        /// A handle to a [`Runtime`](Runtime) identifier
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct RuntimeHandle<C = Global>(pub C);

        impl Runtime {
            /// Create a new runtime using [`Global`](Global) without reusing ids
            pub fn new() -> Self {
                Self::with_counter_and_pool(())
            }
        }
    } else {
        /// A runtime checked identifier
        ///
        /// This uses a runtime id to verify it's identity, this id is provided
        /// by the [`Counter`](Counter) trait, and ids may be reused via the [`PoolMut<C>`](PoolMut)
        pub struct Runtime<C, P: PoolMut<C> = ()> {
            id: C,
            pool: P,
        }

        /// A handle to a [`Runtime`](Runtime) identifier
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct RuntimeHandle<C>(pub C);
    }
}

impl<C: Counter> Runtime<C, ()> {
    /// Create a new runtime using the selected `Counter` without reusing ids
    ///
    /// note: Rust will likely have a hard time inferring which counter to use
    /// so you will likely have to qualify which type to use `Runtime::<MyCounter, _>::with_counter()`
    pub fn with_counter() -> Self { Self::with_counter_and_pool(()) }

    /// Try to create a new runtime using the selected `Counter` without reusing ids
    ///
    /// If the `Counter` is exhausted, then this will return `None`. Otherwise it will
    /// return a valid instance of `Runtime`
    ///
    /// note: Rust will likely have a hard time inferring which counter to use
    /// so you will likely have to qualify which type to use `Runtime::<MyCounter, _>::with_counter()`
    pub fn try_with_counter() -> Option<Self> { Self::try_with_counter_and_pool(()) }
}

impl<C: Counter, P: PoolMut<C>> Runtime<C, P> {
    /// Create a new runtime using the selected `Counter` reusing ids with `PoolMut`
    ///
    /// note: Rust will likely have a hard time inferring which counter to use
    /// so you will likely have to qualify which type to use
    /// `Runtime::<MyCounter, _>::with_counter_and_pool(pool)`
    pub fn with_counter_and_pool(pool: P) -> Self {
        Self::try_with_counter_and_pool(pool).expect("Could not allocate a new runtime id")
    }

    /// Try to create a new runtime using the selected `Counter` reusing ids with `PoolMut`
    ///
    /// It will first try and pool an id, and if that's not possible,
    /// it will generate a new id with `Counter`. If the `Counter` is exhausted
    /// then this will return `None`. Otherwise it will return a valid instance of `Runtime`
    ///
    /// note: Rust will likely have a hard time inferring which counter to use
    /// so you will likely have to qualify which type to use
    /// `Runtime::<MyCounter, _>::with_counter_and_pool(pool)`
    pub fn try_with_counter_and_pool(mut pool: P) -> Option<Self> {
        let id = pool.take_mut().map(RuntimeId::into_inner).or_else(C::try_next)?;

        Some(Runtime { id, pool })
    }

    #[inline]
    /// A handle that this runtime identifier owns
    pub fn handle(&self) -> RuntimeHandle<C> { RuntimeHandle(self.id) }
}

unsafe impl<C: Counter> crate::Handle for RuntimeHandle<C> {}
unsafe impl<C: Counter, P: PoolMut<C>> crate::Identifier for Runtime<C, P> {
    type Handle = RuntimeHandle<C>;

    #[inline]
    fn handle(&self) -> Self::Handle { self.handle() }

    #[inline]
    fn owns(&self, handle: &Self::Handle) -> bool { self.id == handle.0 }
}

impl<C, P: PoolMut<C>> Drop for Runtime<C, P> {
    #[inline]
    fn drop(&mut self) {
        // # Safety
        //
        // here `C: Counter` -> `C: Copy` (because we only construct such `C`)
        let _ = self.pool.try_put_mut(RuntimeId(unsafe { core::ptr::read(&self.id) }));
    }
}

impl<C: Counter, P: PoolMut<C>> Eq for Runtime<C, P> {}
impl<C: Counter, P: PoolMut<C>> PartialEq for Runtime<C, P> {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

use core::fmt;
impl<C: fmt::Debug, P: PoolMut<C>> fmt::Debug for Runtime<C, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Runtime({:?})", self.id) }
}
