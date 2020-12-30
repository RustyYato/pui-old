pub use core::{
    cell::{Cell, UnsafeCell},
    compile_error, concat,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Drop,
    option::Option,
    stringify,
};

#[cfg(feature = "std")]
pub use std::sync::{Condvar, Mutex, MutexGuard, Once};

#[cfg(any(feature = "std", feature = "alloc"))]
pub use std::{collections::VecDeque, vec::Vec};

#[cfg(feature = "std")]
pub use std::thread_local;

#[cfg(not(feature = "std"))]
pub use crate::thread_local;

use core::num::*;
#[cfg(feature = "atomic")]
use core::sync::atomic::{Ordering::*, *};

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "std"))]
macro_rules! thread_local {
    ($(#[$meta:meta])* static $name:ident: $type:ty = $init:expr;) => {
        $(#[$meta])*
        static $name: $crate::macros::LocalKey<$type> = $crate::macros::LocalKey::new();
        $crate::macros::compile_error! {"the `std` feature on `pui` must be turned on to allow thread local storage"}
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "atomic")]
macro_rules! if_atomic_feature {
    ($($tokens:tt)*) => { $($tokens)* };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "atomic"))]
macro_rules! if_atomic_feature {
    ($($tokens:tt)*) => {
        compile_error! { "the `atomic` feature on `pui` must be turned on to allow syncronized globals" }
    };
}

#[cfg(feature = "atomic")]
pub struct OnceFlag(AtomicBool);

#[cfg(feature = "atomic")]
impl OnceFlag {
    pub const fn new() -> Self { Self(AtomicBool::new(true)) }

    pub fn take(&self) -> bool { self.0.compare_and_swap(true, false, Relaxed) }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub struct ResettableOnceFlag {
            locked: AtomicBool,
            once: Once,
            mutex: UnsafeCell<MaybeUninit<Mutex<()>>>,
            cv: UnsafeCell<MaybeUninit<Condvar>>,
        }

        unsafe impl Send for ResettableOnceFlag {}
        unsafe impl Sync for ResettableOnceFlag {}

        impl ResettableOnceFlag {
            pub const fn new() -> Self {
                Self {
                    locked: AtomicBool::new(false),
                    once: Once::new(),
                    mutex: UnsafeCell::new(MaybeUninit::uninit()),
                    cv: UnsafeCell::new(MaybeUninit::uninit()),
                }
            }

            fn init(&self) -> (&Mutex<()>, &Condvar) {
                unsafe {
                    let mutex = self.mutex.get().cast::<Mutex<()>>();
                    let condvar = self.cv.get().cast::<Condvar>();

                    self.once.call_once(|| {
                        mutex.write(Mutex::new(()));
                        condvar.write(Condvar::new());
                    });

                    (&*mutex, &*condvar)
                }
            }

            pub fn acquire(&self) -> bool {
                let locked = self.locked.swap(true, Acquire);

                if locked {
                    let (mutex, cv) = self.init();

                    let mut guard = mutex.lock().unwrap();

                    while self.locked.compare_and_swap(false, true, Acquire) {
                        guard = cv.wait(guard).unwrap();
                    }
                }

                true
            }

            pub fn try_acquire(&self) -> bool {
                !self.locked.swap(true, Acquire)
            }

            pub fn release(&self) {
                self.locked.store(false, Release);

                self.init().1.notify_one();
            }
        }
    } else if #[cfg(feature = "atomic")] {
        pub struct ResettableOnceFlag(AtomicBool);

        impl ResettableOnceFlag {
            pub const fn new() -> Self {
                Self(AtomicBool::new(true))
            }

            pub fn acquire(&self) -> bool {
                self.0.compare_and_swap(true, false, Acquire)
            }

            pub fn try_acquire(&self) -> bool {
                self.acquire()
            }

            pub fn release(&self) {
                self.0.store(true, Release);
            }
        }
    }
}

#[cfg(feature = "atomic")]
pub struct InitFlag(AtomicU8);

#[cfg(feature = "atomic")]
impl InitFlag {
    pub const fn new() -> Self { Self(AtomicU8::new(0)) }

    pub fn start_init(&self) -> bool { 0b00 == self.0.compare_and_swap(0b00, 0b10, Acquire) }

    pub fn finish_init(&self) { self.0.store(0b11, Release); }

    pub fn start_take(&self) -> bool { 0b11 == self.0.compare_and_swap(0b11, 0b01, Acquire) }

    pub fn finish_take(&self) { self.0.store(0b00, Release); }
}

pub struct LocalKey<T>(PhantomData<T>);

impl<T> LocalKey<T> {
    pub const fn new() -> Self { LocalKey(PhantomData) }

    pub fn with<F: FnOnce(&T) -> R, R>(&self, _: F) -> R { todo!() }
}

pub struct LocalOnceFlag(Cell<bool>);

impl LocalOnceFlag {
    pub const fn new() -> Self { Self(Cell::new(true)) }

    pub fn take(&self) -> bool {
        let flag = self.0.get();
        self.0.set(false);
        flag
    }

    pub fn reset(&self) { self.0.set(true); }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacroConstructed(());

impl MacroConstructed {
    /// This function should only be called from a macro defined in pui
    pub const unsafe fn new() -> Self { Self(()) }
}

pub(crate) use private::Private;
mod private {
    pub trait Private {}
}

/// a type that can be used as the backing type in `make_global_id_alloc` an `make_global_id_alloc_tl`
pub unsafe trait Scalar: Private + Copy + Eq {
    #[doc(hidden)]
    type Local;
    #[doc(hidden)]
    #[cfg(feature = "atomic")]
    type Atomic;

    #[doc(hidden)]
    const INIT_LOCAL: Self::Local;
    #[doc(hidden)]
    #[cfg(feature = "atomic")]
    const INIT_ATOMIC: Self::Atomic;

    #[doc(hidden)]
    fn inc_local(_: Self::Local) -> Option<(Self::Local, Self)>;
    #[doc(hidden)]
    #[cfg(feature = "atomic")]
    fn inc_atomic(_: &Self::Atomic) -> Option<Self>;
}

impl Private for () {}
impl crate::Trivial for () {
    const INSTANCE: Self = ();
}
unsafe impl Scalar for () {
    #[doc(hidden)]
    type Local = bool;
    #[doc(hidden)]
    #[cfg(feature = "atomic")]
    type Atomic = AtomicBool;

    #[doc(hidden)]
    const INIT_LOCAL: Self::Local = true;
    #[cfg(feature = "atomic")]
    #[doc(hidden)]
    const INIT_ATOMIC: Self::Atomic = AtomicBool::new(true);

    #[doc(hidden)]
    fn inc_local(this: Self::Local) -> Option<(Self::Local, Self)> {
        if this {
            Some((false, ()))
        } else {
            None
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "atomic")]
    fn inc_atomic(this: &Self::Atomic) -> Option<Self> {
        if this.compare_and_swap(true, false, Relaxed) {
            Some(())
        } else {
            None
        }
    }
}

macro_rules! num {
    () => {};

    (($num:ty, $cfg:literal, $atomic:ty) $($rest:tt)*) => {
        num!{($num, $num, $cfg, $atomic, <$num>::MIN, None, x -> x)}

        num!{$($rest)*}
    };

    (($num:ty, $local:ty, $cfg:literal, $atomic:ty, $min:expr, $max:expr, $ident:ident -> $convert:expr) $($rest:tt)*) => {
        #[cfg_attr(feature = "nightly", cfg(target_has_atomic = $cfg))]
        impl Private for $num {}
        #[cfg_attr(feature = "nightly", cfg(target_has_atomic = $cfg))]
        unsafe impl Scalar for $num {
            #[doc(hidden)]
            type Local = $local;
            #[doc(hidden)]
            #[cfg(feature = "atomic")]
            type Atomic = $atomic;
            #[doc(hidden)]
            const INIT_LOCAL: Self::Local = $min;
            #[doc(hidden)]
            #[cfg(feature = "atomic")]
            const INIT_ATOMIC: Self::Atomic = <$atomic>::new($min);

            #[doc(hidden)]
            fn inc_local(value: Self::Local) -> Option<(Self::Local, Self)> {
                let next = value.checked_add(1);

                let is_out_of_bounds = match (next, $max) {
                    (None, _) => true,
                    (_, None) => false,
                    (Some(next), Some(max)) => next >= max,
                };

                if is_out_of_bounds {
                    return None
                }

                let next = next.unwrap();

                let $ident = value;
                Some((next, $convert))
            }

            #[doc(hidden)]
            #[cfg(feature = "atomic")]
            fn inc_atomic(atomic: &Self::Atomic) -> Option<Self> {
                let mut value = atomic.load(Relaxed);

                loop {
                    let next = value.checked_add(1);

                    let is_out_of_bounds = match (next, $max) {
                        (None, _) => true,
                        (_, None) => false,
                        (Some(next), Some(max)) => next >= max,
                    };

                    if is_out_of_bounds {
                        break None
                    }

                    let next = next.unwrap();

                    if let Err(old_value) = atomic.compare_exchange_weak(value, next, Relaxed, Relaxed) {
                        value = old_value;
                    } else {
                        let $ident = value;
                        break Some($convert)
                    }
                }
            }
        }

        num!{$($rest)*}
    };
}

fn cast(x: u8) -> i8 { i8::from_ne_bytes([x]) }

num! {
    (u8, "8", AtomicU8)
    (u16, "16", AtomicU16)
    (u32, "32", AtomicU32)
    (u64, "64", AtomicU64)
    (usize, "ptr", AtomicUsize)

    (i8, "8", AtomicI8)
    (i16, "16", AtomicI16)
    (i32, "32", AtomicI32)
    (i64, "64", AtomicI64)
    (isize, "ptr", AtomicIsize)

    (NonZeroU8, u8, "8", AtomicU8, 1, None, x -> unsafe { NonZeroU8::new_unchecked(x) })
    (NonZeroU16, u16, "16", AtomicU16, 1, None, x -> unsafe { NonZeroU16::new_unchecked(x) })
    (NonZeroU32, u32, "32", AtomicU32, 1, None, x -> unsafe { NonZeroU32::new_unchecked(x) })
    (NonZeroU64, u64, "64", AtomicU64, 1, None, x -> unsafe { NonZeroU64::new_unchecked(x) })
    (NonZeroUsize, usize, "ptr", AtomicUsize, 1, None, x -> unsafe { NonZeroUsize::new_unchecked(x) })

    (NonZeroI8, u8, "8", AtomicU8, 1, None, x -> unsafe { NonZeroI8::new_unchecked(cast(x)) })
    (NonZeroI16, u16, "16", AtomicU16, 1, None, x -> unsafe {
        let [a, b] = u16::to_ne_bytes(x);
        NonZeroI16::new_unchecked(i16::from_ne_bytes([a, b]))
    })
    (NonZeroI32, u32, "32", AtomicU32, 1, None, x -> unsafe {
        let [a, b, c, d] = u32::to_ne_bytes(x);
        NonZeroI32::new_unchecked(i32::from_ne_bytes([a, b, c, d]))
    })
    (NonZeroI64, u64, "64", AtomicU64, 1, None, x -> unsafe {
        let [a, b, c, d, e, f, g, h] = u64::to_ne_bytes(x);
        NonZeroI64::new_unchecked(i64::from_ne_bytes([a, b, c, d, e, f, g, h]))
    })
    (NonZeroIsize, usize, "ptr", AtomicUsize, 1, None, x -> unsafe {
        NonZeroIsize::new_unchecked(core::mem::transmute::<usize, isize>(x))
    })

    ([u8; 1], u8, "8", AtomicU8, 0, None, x -> [x])
    ([u8; 2], u16, "16", AtomicU16, 0, None, x -> u16::to_ne_bytes(x))
    ([u8; 4], u32, "32", AtomicU32, 0, None, x -> u32::to_ne_bytes(x))
    ([u8; 8], u64, "64", AtomicU64, 0, None, x -> u64::to_ne_bytes(x))

    ([u8; 3], u32, "32", AtomicU32, 0, Some(1 << (3 * 8)), x -> {
        let [a, b, c, ..] = u32::to_le_bytes(x);
        [a, b, c]
    })
    ([u8; 5], u64, "64", AtomicU64, 0, Some(1 << (5 * 8)), x -> {
        let [a, b, c, d, e, ..] = u64::to_le_bytes(x);
        [a, b, c, d, e]
    })
    ([u8; 6], u64, "64", AtomicU64, 0, Some(1 << (6 * 8)), x -> {
        let [a, b, c, d, e, f, ..] = u64::to_le_bytes(x);
        [a, b, c, d, e, f]
    })
    ([u8; 7], u64, "64", AtomicU64, 0, Some(1 << (7 * 8)), x -> {
        let [a, b, c, d, e, f, g, ..] = u64::to_le_bytes(x);
        [a, b, c, d, e, f, g]
    })

    ([u16; 1], u16, "16", AtomicU16, 0, None, x -> [x])
    ([u16; 2], u32, "32", AtomicU32, 0, None, x -> {
        let [a, b, c, d] = u32::to_ne_bytes(x);
        [u16::from_ne_bytes([a, b]), u16::from_ne_bytes([c, d])]
    })
    ([u16; 3], u64, "64", AtomicU64, 0, Some(1 << (6 * 8)), x -> {
        let [a, b, c, d, e, f, ..] = u64::to_le_bytes(x);
        [u16::from_ne_bytes([a, b]), u16::from_ne_bytes([c, d]), u16::from_ne_bytes([e, f])]
    })
    ([u16; 4], u64, "64", AtomicU64, 0, None, x -> {
        let [a, b, c, d, e, f, g, h] = u64::to_le_bytes(x);
        [u16::from_ne_bytes([a, b]), u16::from_ne_bytes([c, d]), u16::from_ne_bytes([e, f]), u16::from_ne_bytes([g, h])]
    })

    ([u32; 1], u32, "32", AtomicU32, 0, None, x -> [x])
    ([u32; 2], u64, "64", AtomicU64, 0, None, x -> {
        let [a, b, c, d, e, f, g, h] = u64::to_le_bytes(x);
        [u32::from_ne_bytes([a, b, c, d]), u32::from_ne_bytes([e, f, g, h])]
    })

    ([u64; 1], u64, "64", AtomicU64, 0, None, x -> [x])

    ([i8; 1], u8, "8", AtomicU8, 0, None, x -> [cast(x)])
    ([i8; 2], u16, "16", AtomicU16, 0, None, x -> {
        let [a, b] = u16::to_ne_bytes(x);
        [cast(a), cast(b)]
    })
    ([i8; 4], u32, "32", AtomicU32, 0, None, x -> {
        let [a, b, c, d] = u32::to_ne_bytes(x);
        [cast(a), cast(b), cast(c), cast(d)]
    })
    ([i8; 8], u64, "64", AtomicU64, 0, None, x -> {
        let [a, b, c, d, e, f, g, h] = u64::to_ne_bytes(x);
        [cast(a), cast(b), cast(c), cast(d), cast(e), cast(f), cast(g), cast(h)]
    })

    ([i8; 3], u32, "32", AtomicU32, 0, Some(1 << (3 * 8)), x -> {
        let [a, b, c, ..] = u32::to_le_bytes(x);
        [cast(a), cast(b), cast(c)]
    })
    ([i8; 5], u64, "64", AtomicU64, 0, Some(1 << (5 * 8)), x -> {
        let [a, b, c, d, e, ..] = u64::to_le_bytes(x);
        [cast(a), cast(b), cast(c), cast(d), cast(e)]
    })
    ([i8; 6], u64, "64", AtomicU64, 0, Some(1 << (6 * 8)), x -> {
        let [a, b, c, d, e, f, ..] = u64::to_le_bytes(x);
        [cast(a), cast(b), cast(c), cast(d), cast(e), cast(f)]
    })
    ([i8; 7], u64, "64", AtomicU64, 0, Some(1 << (7 * 8)), x -> {
        let [a, b, c, d, e, f, g, ..] = u64::to_le_bytes(x);
        [cast(a), cast(b), cast(c), cast(d), cast(e), cast(f), cast(g)]
    })

    ([i16; 1], u16, "16", AtomicU16, 0, None, x -> {
        let [a, b] = u16::to_ne_bytes(x);
        [i16::from_ne_bytes([a, b])]
    })
    ([i16; 2], u32, "32", AtomicU32, 0, None, x -> {
        let [a, b, c, d] = u32::to_ne_bytes(x);
        [i16::from_ne_bytes([a, b]), i16::from_ne_bytes([c, d])]
    })
    ([i16; 3], u64, "64", AtomicU64, 0, Some(1 << (6 * 8)), x -> {
        let [a, b, c, d, e, f, ..] = u64::to_le_bytes(x);
        [i16::from_ne_bytes([a, b]), i16::from_ne_bytes([c, d]), i16::from_ne_bytes([e, f])]
    })
    ([i16; 4], u64, "64", AtomicU64, 0, None, x -> {
        let [a, b, c, d, e, f, g, h] = u64::to_le_bytes(x);
        [i16::from_ne_bytes([a, b]), i16::from_ne_bytes([c, d]), i16::from_ne_bytes([e, f]), i16::from_ne_bytes([g, h])]
    })

    ([i32; 1], u32, "32", AtomicU32, 0, None, x -> {
        let [a, b, c, d] = u32::to_ne_bytes(x);
        [i32::from_ne_bytes([a, b, c, d])]
    })
    ([i32; 2], u64, "64", AtomicU64, 0, None, x -> {
        let [a, b, c, d, e, f, g, h] = u64::to_le_bytes(x);
        [i32::from_ne_bytes([a, b, c, d]), i32::from_ne_bytes([e, f, g, h])]
    })

    ([i64; 1], u64, "64", AtomicU64, 0, None, x -> {
        let [a, b, c, d, e, f, g, h] = u64::to_le_bytes(x);
        [i64::from_ne_bytes([a, b, c, d, e, f, g, h])]
    })
}

#[doc(hidden)]
#[macro_export]
macro_rules! doc_item {
    ($(#[doc = $doc:expr])* pub $($rest:tt)*) => {
        $(#[doc = $doc])*
        pub $($rest)*
    }
}
