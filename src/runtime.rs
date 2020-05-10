//! A runtime checked identifier
//!
//! It uses an id allocator (usually a simple counter) to generate new ids,
//! then uses those ids to verifiy it's identity. You can either use the `Global`
//! allocator, which uses the largest unsigned non-zero integer possible (up to 64-bits),
//! or you can create your own id allocator using the [`make_counter`](make_counter)
//!
//! ```
//! type BackingScalar = [u8; 3];
//! pui::make_counter! {
//!     pub type MyCustomIdAllocator = BackingScalar;
//! }
//! ```
//!
// You can use any primitive integer type and their non-zero variants

mod macros;
mod pool;
pub use pool::*;

/// A counter that allocates new ids
///
/// # Safety
///
/// * These ids may *never* repeat
/// * There must be no other way to create instances of `Counter`
pub unsafe trait Counter: Copy + Eq {
    /// Get the next id, panics if there are no next ids
    fn next() -> Self;

    /// Try to get the next id, returns `None` if there are no next ids
    fn try_next() -> Option<Self>;
}

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
}

/// A runtime checked identifier
///
/// This uses a runtime id to verify it's identity, this id is provided
/// by the [`Counter`](Counter) trait, and ids may be reused via the [`PoolMut<C>`](PoolMut)
pub struct Runtime<C = Global, R: PoolMut<C> = ()> {
    id: C,
    pool: R,
}

/// A handle to a [`Runtime`](Runtime) identifier
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeHandle<C>(pub C);

impl Runtime {
    /// Create a new runtime using [`Global`](Global) without reusing ids
    pub fn new() -> Self {
        Self::with_counter_and_reuse(())
    }
}

impl<C: Counter> Runtime<C, ()> {
    /// Create a new runtime using the selected `Counter` without reusing ids
    ///
    /// note: Rust will likely have a hard time inferring which counter to use
    /// so you will likely have to qualify which type to use `Runtime::<MyCounter, _>::with_counter()`
    pub fn with_counter() -> Self {
        Self::with_counter_and_reuse(())
    }

    /// Try to create a new runtime using the selected `Counter` without reusing ids
    ///
    /// If the `Counter` is exhausted, then this will return `None`. Otherwise it will
    /// return a valid instance of `Runtime`
    ///
    /// note: Rust will likely have a hard time inferring which counter to use
    /// so you will likely have to qualify which type to use `Runtime::<MyCounter, _>::with_counter()`
    pub fn try_with_counter() -> Option<Self> {
        Self::try_with_counter_and_reuse(())
    }
}

impl<C: Counter, R: PoolMut<C>> Runtime<C, R> {
    /// Create a new runtime using the selected `Counter` reusing ids with `PoolMut`
    ///
    /// note: Rust will likely have a hard time inferring which counter to use
    /// so you will likely have to qualify which type to use
    /// `Runtime::<MyCounter, _>::with_counter_and_reuse(pool)`
    pub fn with_counter_and_reuse(pool: R) -> Self {
        Self::try_with_counter_and_reuse(pool).expect("Could not allocate a new runtime id")
    }

    /// Try to create a new runtime using the selected `Counter` reusing ids with `PoolMut`
    ///
    /// It will first try and pool an id, and if that's not possible,
    /// it will generate a new id with `Counter`. If the `Counter` is exhausted
    /// then this will return `None`. Otherwise it will return a valid instance of `Runtime`
    ///
    /// note: Rust will likely have a hard time inferring which counter to use
    /// so you will likely have to qualify which type to use
    /// `Runtime::<MyCounter, _>::with_counter_and_reuse(pool)`
    pub fn try_with_counter_and_reuse(mut pool: R) -> Option<Self> {
        let id = pool.take().or_else(C::try_next)?;

        Some(Runtime { id, pool })
    }

    /// A handle that this runtime identifier owns
    pub fn handle(&self) -> RuntimeHandle<C> {
        RuntimeHandle(self.id)
    }

    /// The pool mechanism that this runtime identifier uses
    pub fn pool(&self) -> &R {
        &self.pool
    }

    /// The pool mechanism that this runtime identifier uses
    pub fn reuse_mut(&mut self) -> &mut R {
        &mut self.pool
    }
}

unsafe impl<C: Counter, R: PoolMut<C>> crate::Identifier for Runtime<C, R> {
    type Handle = RuntimeHandle<C>;

    fn handle(&self) -> Self::Handle {
        self.handle()
    }

    fn owns(&self, handle: &Self::Handle) -> bool {
        self.id == handle.0
    }
}

impl<C, R: PoolMut<C>> Drop for Runtime<C, R> {
    fn drop(&mut self) {
        // # Safety
        //
        // here `C: Counter` -> `C: Copy` (because we only construct such `C`)
        let _ = self.pool.try_put(unsafe { core::ptr::read(&self.id) });
    }
}

impl<C: Counter, R: PoolMut<C>> Eq for Runtime<C, R> {}
impl<C: Counter, R: PoolMut<C>> PartialEq for Runtime<C, R> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
