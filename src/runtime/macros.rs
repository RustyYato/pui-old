/// Create a new type that implements [`Counter`](runtime::Counter)
/// that can be used with [`Runtime`](runtime::Runtime)
///
/// For example,
///
/// ```
/// pui::make_counter! {
///     pub type MyCounter = [u8; 3];
/// }
/// ```
///
/// will generate a 24-bit counter that is 1 byte aligned. You can use any type that implements
/// [`Scalar`](Scalar) as the backing type of a counter.
///
/// You can then use it like so,
/// ```
/// # use pui::runtime::Runtime;
/// # pui::make_counter! { type MyCounter = [u8; 3]; }
/// let runtime_counter /* : Runtime<MyCounter> */ = MyCounter::new_runtime();
/// ```
/// or if you want to plug in a custom [`PoolMut<_>`](runtime::PoolMut),
/// ```
/// # use pui::runtime::Runtime;
/// # pui::make_counter! { type MyCounter = [u8; 3]; }
/// # let pool = ();
/// let runtime_counter /* : Runtime<MyCounter, _> */ = MyCounter::with_pool(pool);
/// ```
#[macro_export]
macro_rules! make_counter {
    ($(#[$meta:meta])*$v:vis type $name:ident = $inner:ty;) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq)]
        $v struct $name($inner);

        impl $name {
            /// Create a new new `Runtime`
            ///
            /// panic if the counter is exhausted
            pub fn new_runtime() -> $crate::runtime::Runtime<Self> {
                $crate::runtime::Runtime::with_counter()
            }

            /// Tryr to create a new new `Runtime`, return `None` if this counter is exhausted
            pub fn try_new_runtime() -> Option<$crate::runtime::Runtime<Self>> {
                $crate::runtime::Runtime::try_with_counter()
            }

            /// Create a new new `Runtime` with the given pool
            ///
            /// panic if the pool is empty and the counter is exhausted
            pub fn with_pool<P: $crate::runtime::PoolMut<Self>>(pool: P) -> $crate::runtime::Runtime<Self, P> {
                $crate::runtime::Runtime::with_counter_and_pool(pool)
            }

            /// Try to create a new new `Runtime` with the given pool
            /// return None if the pool is empty and the counter is exhausted
            pub fn try_with_pool<P: $crate::runtime::PoolMut<Self>>(pool: P) -> Option<$crate::runtime::Runtime<Self, P>> {
                $crate::runtime::Runtime::try_with_counter_and_pool(pool)
            }
        }

        unsafe impl $crate::runtime::Counter for $name {
            fn next() -> Self {
                <Self as $crate::runtime::Counter>::try_next().expect($crate::macros::concat!(
                    "Cannot overflow <",
                    $crate::macros::stringify!($name),
                    " as pui::runtime::Counter>::next"
                ))
            }

            fn try_next() -> Option<Self> {
                #[allow(non_upper_case_globals)]
                static make_runtime_NEXT_ID: <$inner as $crate::macros::Scalar>::Atomic =
                    <$inner as $crate::macros::Scalar>::INIT_ATOMIC;

                <$inner as $crate::macros::Scalar>::inc_atomic(&make_runtime_NEXT_ID).map($name)
            }
        }
    };
}

/// Create a new type that implements [`Counter`](runtime::Counter)
/// that can be used with [`Runtime`](runtime::Runtime)
/// which is implemented using a thread-local count
///
/// For example,
///
/// ```
/// pui::make_counter_tl! {
///     pub type MyCounter = [u8; 3];
/// }
/// ```
///
/// will generate a 24-bit counter that is 1 byte aligned. You can use any type that implements
/// [`Scalar`](Scalar) as the backing type of a counter.
///
/// You can then use it like so,
/// ```
/// # use pui::runtime::Runtime;
/// # pui::make_counter_tl! { type MyCounter = [u8; 3]; }
/// let runtime_counter /* : Runtime<MyCounter> */ = MyCounter::new_runtime();
/// ```
/// or if you want to plug in a custom [`PoolMut<_>`](runtime::PoolMut),
/// ```
/// # use pui::runtime::Runtime;
/// # pui::make_counter_tl! { type MyCounter = [u8; 3]; }
/// # let pool = ();
/// let runtime_counter /* : Runtime<MyCounter, _> */ = MyCounter::with_pool(pool);
/// ```
#[macro_export]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
macro_rules! make_counter_tl {
    ($(#[$meta:meta])*$v:vis type $name:ident = $inner:ty;) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq)]
        $v struct $name($inner, $crate::macros::PhantomData<$crate::ThreadLocal>);

        impl $name {
            /// Create a new new `Runtime`
            ///
            /// panic if the counter is exhausted
            pub fn new_runtime() -> $crate::runtime::Runtime<Self> {
                $crate::runtime::Runtime::with_counter()
            }

            /// Tryr to create a new new `Runtime`, return `None` if this counter is exhausted
            pub fn try_new_runtime() -> Option<$crate::runtime::Runtime<Self>> {
                $crate::runtime::Runtime::try_with_counter()
            }

            /// Create a new new `Runtime` with the given pool
            ///
            /// panic if the pool is empty and the counter is exhausted
            pub fn with_pool<P: $crate::runtime::PoolMut<Self>>(pool: P) -> $crate::runtime::Runtime<Self, P> {
                $crate::runtime::Runtime::with_counter_and_pool(pool)
            }

            /// Try to create a new new `Runtime` with the given pool
            /// return None if the pool is empty and the counter is exhausted
            pub fn try_with_pool<P: $crate::runtime::PoolMut<Self>>(pool: P) -> Option<$crate::runtime::Runtime<Self, P>> {
                $crate::runtime::Runtime::try_with_counter_and_pool(pool)
            }
        }

        unsafe impl $crate::runtime::Counter for $name {
            fn next() -> Self {
                <Self as $crate::runtime::Counter>::try_next().expect($crate::macros::concat!(
                    "Cannot overflow <",
                    $crate::macros::stringify!($name),
                    " as pui::runtime::Counter>::next"
                ))
            }

            fn try_next() -> Option<Self> {
                $crate::macros::thread_local! {
                    #[allow(non_upper_case_globals)]
                    static make_runtime_NEXT_ID: $crate::macros::Cell<<$inner as $crate::macros::Scalar>::Local> =
                        $crate::macros::Cell::new(<$inner as $crate::macros::Scalar>::INIT_LOCAL);
                }

                make_runtime_NEXT_ID.with(|value| {
                    let (val, id) = <$inner as $crate::macros::Scalar>::inc_local(value.get())?;
                    value.set(val);
                    Some(id)
                }).map(|val| $name(val, $crate::macros::PhantomData))
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! make_global_option_pool {
    ($(#[$meta:meta])* $v:vis one $name:ident($item:ty);) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        $v struct $name;

        const _: () = {
            #[allow(non_upper_case_globals)]
            static make_global_option_pool: $crate::macros::InitFlag = $crate::macros::InitFlag::new();
            #[allow(non_upper_case_globals)]
            static mut make_global_SINGLE_REUSE_ITEM:
                $crate::macros::MaybeUninit<$crate::runtime::RuntimeId<$item>> = $crate::macros::MaybeUninit::uninit();

            impl $crate::runtime::PoolMut<$item> for $name {
                fn try_put_mut(&mut self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    if make_global_option_pool.start_init() {
                        unsafe { make_global_SINGLE_REUSE_ITEM = $crate::macros::MaybeUninit::new(value); }
                        make_global_option_pool.finish_init();
                        Ok(())
                    } else {
                        Err(value)
                    }
                }

                fn take_mut(&mut self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    if make_global_option_pool.start_take() {
                        let x = unsafe { make_global_SINGLE_REUSE_ITEM.as_ptr().read() };
                        make_global_option_pool.finish_take();
                        Some(x)
                    } else {
                        None
                    }
                }
            }

            impl $crate::runtime::Pool<$item> for $name {
                fn try_put(&self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::try_put_mut(&mut $name, value)
                }

                fn take(&self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::take_mut(&mut $name)
                }
            }
        };
    };
    ($(#[$meta:meta])* $v:vis thread_local one $name:ident($item:ty);) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        $v struct $name;

        const _: () = {
            $crate::macros::thread_local! {
                #[allow(non_upper_case_globals)]
                static make_global_SINGLE_REUSE_ITEM:
                    $crate::macros::Cell<$crate::macros::Option<$crate::runtime::RuntimeId<$item>>> = $crate::macros::Cell::default()
            }

            impl $crate::runtime::PoolMut<$item> for $name {
                fn try_put_mut(&mut self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    make_global_SINGLE_REUSE_ITEM.with(|c| {
                        if unsafe { (*c.as_ptr()).is_none() } {
                            c.set(Some(value));
                            Ok(())
                        } else {
                            Err(value)
                        }
                    })
                }

                fn take_mut(&mut self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    make_global_SINGLE_REUSE_ITEM.with(|c| c.take())
                }
            }

            impl $crate::runtime::Pool<$item> for $name {
                fn try_put(&self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::try_put_mut(&mut $name, value)
                }

                fn take(&self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::take_mut(&mut $name)
                }
            }
        };
    };
}

#[macro_export]
#[cfg(not(any(doc, feature = "std")))]
macro_rules! make_global_pool {
    ($(#[$meta:meta])* $v:vis stack $name:ident($item:ty);) => {
        $crate::macros::compile_error!{"the `std` feature on `pui` msut be turned on to allow global stack pool"}
    };
    ($(#[$meta:meta])* $v:vis queue $name:ident($item:ty);) => {
        $crate::macros::compile_error!{"the `std` feature on `pui` msut be turned on to allow global queue pool"}
    };
    ($(#[$meta:meta])* $v:vis one $name:ident($item:ty);) => {
        $crate::make_global_option_pool!{$(#[$meta])* $v one $name($item);}
    };
    ($(#[$meta:meta])* $v:vis thread_local stack $name:ident($item:ty);) => {
        $crate::macros::compile_error!{"the `std` feature on `pui` msut be turned on to allow thread local stack pool"}
    };
    ($(#[$meta:meta])* $v:vis thread_local queue $name:ident($item:ty);) => {
        $crate::macros::compile_error!{"the `std` feature on `pui` msut be turned on to allow thread local queue pool"}
    };
    ($(#[$meta:meta])* $v:vis thread_local one $name:ident($item:ty);) => {
        $crate::make_global_option_pool!{$(#[$meta])* $v thread_local one $name($item);}
    };
}

/// Create a new type that implements [`Pool`](runtime::Pool) and [`PoolMut`](runtime::PoolMut)
/// that can be used with [`Runtime`](runtime::Runtime)
///
/// For example,
///
/// ```
/// pui::make_global_pool! {
///     pub stack MyPool(pui::runtime::Global);
/// }
/// ```
///
/// will generate a global pool that yields used ids in FILO order.
///
/// in place of `stack` you can also use,
///
/// * stack - FILO order
/// * thread_local stack - FILO order, but stores ids in a thread local (this is best used with thread local ids)
/// * queue - FIFO order
/// * thread_local queue - FIFO order, but stores ids in a thread local (this is best used with thread local ids)
/// * one - stores a single id, best used with a counter backed by `()`
/// * thread_local one - stores a single id, best used with a thread_local id backed by `()`
///
/// in place of `pui::runtime::Global` you can use any type that implements `Counter`
#[macro_export]
#[cfg(any(doc, feature = "std"))]
macro_rules! make_global_pool {
    ($(#[$meta:meta])* $v:vis stack $name:ident($item:ty);) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        $v struct $name;

        const _: () = {
            #[allow(non_upper_case_globals)]
            fn make_global_get_it() -> $crate::macros::MutexGuard<'static, $crate::macros::Vec<$crate::runtime::RuntimeId<$item>>> {
                static mut make_global_REUSE: $crate::macros::MaybeUninit<$crate::macros::Mutex<$crate::macros::Vec<$crate::runtime::RuntimeId<$item>>>> =
                    $crate::macros::MaybeUninit::uninit();
                static make_global_ONCE: $crate::macros::Once = $crate::macros::Once::new();

                make_global_ONCE.call_once(|| unsafe {
                    make_global_REUSE = $crate::macros::MaybeUninit::new($crate::macros::Mutex::default());
                });

                let make_global = unsafe { &*make_global_REUSE.as_ptr() };
                make_global.lock().unwrap()
            }

            impl $crate::runtime::PoolMut<$item> for $name {
                fn try_put_mut(&mut self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    make_global_get_it().push(value);
                    Ok(())
                }

                fn take_mut(&mut self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    make_global_get_it().pop()
                }
            }

            impl $crate::runtime::Pool<$item> for $name {
                fn try_put(&self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::try_put_mut(&mut $name, value)
                }

                fn take(&self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::take_mut(&mut $name)
                }
            }
        };
    };
    ($(#[$meta:meta])* $v:vis queue $name:ident($item:ty);) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        $v struct $name;

        const _: () = {
            #[allow(non_upper_case_globals)]
            fn make_global_get_it() -> $crate::macros::MutexGuard<'static, $crate::macros::VecDeque<$crate::runtime::RuntimeId<$item>>> {
                static mut make_global_REUSE: $crate::macros::MaybeUninit<$crate::macros::Mutex<$crate::macros::VecDeque<$crate::runtime::RuntimeId<$item>>>> =
                    $crate::macros::MaybeUninit::uninit();
                static make_global_ONCE: $crate::macros::Once = $crate::macros::Once::new();

                make_global_ONCE.call_once(|| unsafe {
                    make_global_REUSE = $crate::macros::MaybeUninit::new($crate::macros::Mutex::default());
                });

                let make_global = unsafe { &*make_global_REUSE.as_ptr() };
                make_global.lock().unwrap()
            }

            impl $crate::runtime::PoolMut<$item> for $name {
                fn try_put_mut(&mut self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    make_global_get_it().push_back(value);
                    Ok(())
                }

                fn take_mut(&mut self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    make_global_get_it().pop_front()
                }
            }

            impl $crate::runtime::Pool<$item> for $name {
                fn try_put(&self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::try_put_mut(&mut $name, value)
                }

                fn take(&self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::take_mut(&mut $name)
                }
            }
        };
    };
    ($(#[$meta:meta])* $v:vis one $name:ident($item:ty);) => {
        $crate::make_global_option_pool!{$(#[$meta])* $v one $name($item);}
    };
    ($(#[$meta:meta])* $v:vis thread_local stack $name:ident($item:ty);) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        $v struct $name;

        const _: () = {
            $crate::macros::thread_local! {
                #[allow(non_upper_case_globals)]
                static make_global_REUSE: $crate::macros::UnsafeCell<$crate::macros::Vec<$crate::runtime::RuntimeId<$item>>> =
                    $crate::macros::UnsafeCell::default();
            }

            impl $crate::runtime::PoolMut<$item> for $name {
                fn try_put_mut(&mut self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    make_global_REUSE.with(|pool| unsafe {
                        (&mut *pool.get()).push(value)
                    });
                    Ok(())
                }

                fn take_mut(&mut self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    make_global_REUSE.with(|pool| unsafe {
                        (&mut *pool.get()).pop()
                    })
                }
            }

            impl $crate::runtime::Pool<$item> for $name {
                fn try_put(&self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::try_put_mut(&mut $name, value)
                }

                fn take(&self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::take_mut(&mut $name)
                }
            }
        };
    };
    ($(#[$meta:meta])* $v:vis thread_local queue $name:ident($item:ty);) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        $v struct $name;

        const _: () = {
            $crate::macros::thread_local! {
                #[allow(non_upper_case_globals)]
                static make_global_REUSE: $crate::macros::UnsafeCell<$crate::macros::VecDeque<$crate::runtime::RuntimeId<$item>>> =
                    $crate::macros::UnsafeCell::default();
            }

            impl $crate::runtime::PoolMut<$item> for $name {
                fn try_put_mut(&mut self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    make_global_REUSE.with(|pool| unsafe {
                        (&mut *pool.get()).push_back(value)
                    });
                    Ok(())
                }

                fn take_mut(&mut self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    make_global_REUSE.with(|pool| unsafe {
                        (&mut *pool.get()).pop_front()
                    })
                }
            }

            impl $crate::runtime::Pool<$item> for $name {
                fn try_put(&self, value: $crate::runtime::RuntimeId<$item>) -> Result<(), $crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::try_put_mut(&mut $name, value)
                }

                fn take(&self) -> Option<$crate::runtime::RuntimeId<$item>> {
                    <Self as $crate::runtime::PoolMut<$item>>::take_mut(&mut $name)
                }
            }
        };
    };
    ($(#[$meta:meta])* $v:vis thread_local one $name:ident($item:ty);) => {
        $crate::make_global_option_pool!{$(#[$meta])* $v thread_local one $name($item);}
    };
}
