//! A [`Runtime`] checked identifier
//!
//! It uses an id allocator (usually a simple id_alloc) to generate new ids,
//! then uses those ids to verifiy it's identity. You can either use the `Global`
//! allocator, which uses the largest unsigned non-zero integer possible (up to 64-bits),
//! or you can create your own id allocator using the [`make_global_id_alloc`](make_global_id_alloc)
//!
//! ```
//! type BackingScalar = [u8; 3];
//! # #[cfg(feature = "atomic")]
//! pui::make_global_id_alloc! {
//!     pub type MyCustomIdAllocator(CustomId) = BackingScalar;
//! }
//! ```
//!
// You can use any primitive integer type and their non-zero variants

mod macros;
mod pool;
pub use pool::*;

/// an opaque [`Runtime`] id
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeId<T>(T);

impl<T> RuntimeId<T> {
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

/// A id_alloc that allocates new ids
///
/// # Safety
///
/// two equal ids may never exist together on the same thread
///
/// This implies that `IdAlloc::*next()` may never repeat an id on the same thread,
/// and if it can repeat itself on different threads then `Self: !Send + !Sync`
/// and `Self::Id:: !Send + !Sync`.
pub unsafe trait IdAlloc {
    /// The unique identifiers that this allocator produces
    type Id: Copy + Eq;

    /// Get the next id, panics if there are no next ids
    fn alloc(&mut self) -> Self::Id;

    /// Try to get the next id, returns `None` if there are no next ids
    fn try_alloc(&mut self) -> Option<Self::Id>;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "atomic")] {
        macro_rules! make_global {
            (($(#[$meta:meta])*) ($(#[$id_meta:meta])*)) => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "nightly")] {
                        cfg_if::cfg_if! {
                            if #[cfg(target_has_atomic = "64")] {
                                crate::make_global_id_alloc! {
                                    $(#[$meta])*
                                    pub type Global($(#[$id_meta])* GlobalId) = core::num::NonZeroU64;
                                }
                            } else if #[cfg(target_has_atomic = "32")]{
                                crate::make_global_id_alloc! {
                                    $(#[$meta])*
                                    pub type Global($(#[$id_meta])* GlobalId) = core::num::NonZeroU32;
                                }
                            } else if #[cfg(target_has_atomic = "16")]{
                                crate::make_global_id_alloc! {
                                    $(#[$meta])*
                                    pub type Global($(#[$id_meta])* GlobalId) = core::num::NonZeroU16;
                                }
                            } else {
                                crate::make_global_id_alloc! {
                                    $(#[$meta])*
                                    pub type Global($(#[$id_meta])* GlobalId) = core::num::NonZeroU8;
                                }
                            }
                        }
                    } else {
                        crate::make_global_id_alloc! {
                            $(#[$meta])*
                            pub type Global($(#[$id_meta])* GlobalId) = core::num::NonZeroU64;
                        }
                    }
                }
            };
        }

        make_global! {
            (
                /// A gobal allocator for [`Runtime`] ids (not to be confused with a memory allocator)
                ///
                /// This can be used with [`Runtime`](super::[`Runtime`]::Runtime) to easily
                /// create a new [`Runtime`](super::[`Runtime`]::Runtime) [`Identifier`](super::Identifier)
                #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            )
            (
                /// The Id used by [`Global`](crate::[`Runtime`]::Global)'s [[`IdAlloc`]](crate::[`Runtime`]::IdAlloc)
                /// implementation
                #[derive(PartialOrd, Ord, Hash)]
            )
        }

        /// A [`Runtime`] checked identifier
        ///
        /// This uses a [`Runtime`] id to verify it's identity, this id is provided
        /// by the [[`IdAlloc`]](IdAlloc) trait, and ids may be reused via the [`PoolMut<I::Id>`](PoolMut)
        pub struct Runtime<I: IdAlloc = Global, P: PoolMut<I::Id> = ()> {
            id: I::Id,
            pool: P,
        }

        /// A handle to a [`Runtime`](Runtime) identifier
        #[repr(transparent)]
        pub struct RuntimeHandle<I: IdAlloc = Global>(pub I::Id);

        impl Runtime {
            /// Create a new [`Runtime`] using [`Global`](Global) without reusing ids
            pub fn new() -> Self {
                Self::with_id_alloc_and_pool(&mut Global, ())
            }
        }

        impl<P: PoolMut<GlobalId>> Runtime<Global, P> {
            /// Create a new [`Runtime`] using [`Global`](Global), reusing ids from the
            /// given pool
            pub fn with_pool(pool: P) -> Self {
                Self::with_id_alloc_and_pool(&mut Global, pool)
            }
        }
    } else {
        /// A [`Runtime`] checked identifier
        ///
        /// This uses a [`Runtime`] id to verify it's identity, this id is provided
        /// by the [[`IdAlloc`]](IdAlloc) trait, and ids may be reused via the [`PoolMut<I::Id>`](PoolMut)
        pub struct Runtime<I: IdAlloc, P: PoolMut<I::Id> = ()> {
            id: I::Id,
            pool: P,
        }

        /// A handle to a [`Runtime`](Runtime) identifier
        #[repr(transparent)]
        pub struct RuntimeHandle<I: IdAlloc>(pub I::Id);
    }
}

impl<I: IdAlloc> Runtime<I> {
    /// Create a new [`Runtime`] using the selected [`IdAlloc`] without reusing ids
    pub fn with_id_alloc(id_alloc: &mut I) -> Self { Self::with_id_alloc_and_pool(id_alloc, ()) }

    /// Try to create a new [`Runtime`] using the selected [`IdAlloc`] without reusing ids
    pub fn try_with_id_alloc(id_alloc: &mut I) -> Option<Self> { Self::try_with_id_alloc_and_pool(id_alloc, ()) }
}

impl<I: IdAlloc, P: PoolMut<I::Id>> Runtime<I, P> {
    /// Create a new [`Runtime`] using the selected [`IdAlloc`] reusing ids
    /// from the given pool. If the pool is empty it will aquire a new id from
    /// the [`IdAlloc`]
    pub fn with_id_alloc_and_pool(id_alloc: &mut I, mut pool: P) -> Self {
        let id = match pool.take_mut() {
            Some(id_alloc) => id_alloc.0,
            None => id_alloc.alloc(),
        };

        Runtime { id, pool }
    }

    /// Try to create a new [`Runtime`] using the selected [`IdAlloc`] reusing ids
    /// from the given pool. If the pool is empty it will attempt to aquire a new id
    /// from the [`IdAlloc`]
    pub fn try_with_id_alloc_and_pool(id_alloc: &mut I, mut pool: P) -> Option<Self> {
        let id = match pool.take_mut() {
            Some(id_alloc) => id_alloc.0,
            None => id_alloc.try_alloc()?,
        };

        Some(Runtime { id, pool })
    }

    #[inline]
    /// A handle that this [`Runtime`] identifier owns
    pub fn handle(&self) -> RuntimeHandle<I> { RuntimeHandle(self.id) }
}

impl<I: IdAlloc> Trivial for RuntimeHandle<I>
where
    I::Id: Trivial,
{
    const INSTANCE: Self = Self(Trivial::INSTANCE);
}

unsafe impl<I: IdAlloc> crate::Handle for RuntimeHandle<I> {}
unsafe impl<I: IdAlloc, P: PoolMut<I::Id>> crate::Identifier for Runtime<I, P> {
    type Handle = RuntimeHandle<I>;

    #[inline]
    fn handle(&self) -> Self::Handle { self.handle() }

    #[inline]
    fn owns(&self, handle: &Self::Handle) -> bool { self.id == handle.0 }
}

impl<I: IdAlloc, P: PoolMut<I::Id>> Drop for Runtime<I, P> {
    #[inline]
    fn drop(&mut self) { let _ = self.pool.try_put_mut(RuntimeId(self.id)); }
}

impl<I: IdAlloc, P: PoolMut<I::Id>> Eq for Runtime<I, P> {}
impl<I: IdAlloc, P: PoolMut<I::Id>> PartialEq for Runtime<I, P> {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

use core::fmt;
impl<I: IdAlloc, P: PoolMut<I::Id>> fmt::Debug for Runtime<I, P>
where
    I::Id: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Runtime({:?})", self.id) }
}

use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use crate::Trivial;

impl<I: IdAlloc> fmt::Debug for RuntimeHandle<I>
where
    I::Id: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeHandle").field("inner", &self.0).finish()
    }
}

impl<I: IdAlloc> Copy for RuntimeHandle<I> {}
impl<I: IdAlloc> Clone for RuntimeHandle<I> {
    fn clone(&self) -> Self { *self }
}

impl<I: IdAlloc> Eq for RuntimeHandle<I> {}
impl<I: IdAlloc> PartialEq for RuntimeHandle<I> {
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl<I: IdAlloc> PartialOrd for RuntimeHandle<I>
where
    I::Id: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.0.partial_cmp(&other.0) }
}

impl<I: IdAlloc> Ord for RuntimeHandle<I>
where
    I::Id: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering { self.0.cmp(&other.0) }
}

impl<I: IdAlloc> Hash for RuntimeHandle<I>
where
    I::Id: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state) }
}
