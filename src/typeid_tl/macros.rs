/// Create a new type that can be used with [`typeid_tl::Type`](typeid_tl::Type)
///
/// calling `make_typeid_tl` like so,
/// ```
/// # #[cfg(feature = "std")]
/// pui::make_typeid_tl! {
///     once type OnceThreadLocal;
/// }
/// ```
///
/// will desugar to something like
///
/// ```
/// struct OnceThreadLocal;
///
/// impl OnceThreadLocal {
/// # #[cfg(feature = "std")]
///     pub fn new() -> pui::typeid_tl::Type<Self> {
///         Self::try_new().unwrap()
///     }
///
/// # #[cfg(feature = "std")]
///     pub fn try_new() -> Option<pui::typeid_tl::Type<Self>> {
///         // implementation details
/// # todo!()
///     }
/// }
/// ```
///
/// You can use `OnceThreadLocal::new()` to create a new thread local
/// identifier instance if you are sure there are no other instances
/// active, otherwise use `OnceThreadLocal::try_new()`
#[macro_export]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
macro_rules! make_typeid_tl {
    ($(#[$meta:meta])*$v:vis once type $ident:ident;) => {
        $(#[$meta])*
        $v struct $ident;

        impl $ident {
            $crate::doc_item! {
                #[doc = $crate::macros::concat!("Create a new instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">`")]
                ///
                /// # Panic
                ///
                #[doc = $crate::macros::concat!("If an instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">` already exists")]
                // then panic
                pub fn new() -> $crate::typeid_tl::Type<Self> {
                    Self::try_new().expect($crate::macros::concat!(
                        "Cannot not create multiple `Type<",
                        $crate::macros::stringify!($ident),
                        ">`"
                    ))
                }
            }

            $crate::doc_item! {
                #[doc = $crate::macros::concat!("Try to create a new instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">`")]
                /// If an instance already exists, then return None
                pub fn try_new() -> $crate::macros::Option<$crate::typeid_tl::Type<Self>> {
                    $crate::macros::thread_local! {
                        #[allow(non_upper_case_globals)]
                        static make_typeid_tl_FLAG: $crate::macros::LocalOnceFlag = $crate::macros::LocalOnceFlag::new();
                    }

                    let flag = make_typeid_tl_FLAG.with(|flag| flag.take());

                    if flag {
                        unsafe {
                            $crate::macros::Option::Some($crate::typeid_tl::Type::new_unchecked(
                                Self,
                                $crate::typeid_tl::TypeHandle::new(),
                            ))
                        }
                    } else {
                        $crate::macros::Option::None
                    }
                }
            }
        }
    };
    ($(#[$meta:meta])*$v:vis type $ident:ident;) => {
        $(#[$meta])*
        $v struct $ident($crate::macros::MacroConstructed);

        impl $ident {
            unsafe fn __make_typeid_tl_get_it<F: FnOnce(&$crate::macros::LocalOnceFlag) -> R, R>(f: F) -> R {
                $crate::macros::thread_local! {
                    #[allow(non_upper_case_globals)]
                    static make_typeid_tl_FLAG: $crate::macros::LocalOnceFlag = $crate::macros::LocalOnceFlag::new();
                }

                make_typeid_tl_FLAG.with(f)
            }

            $crate::doc_item! {
                #[doc = $crate::macros::concat!("Create a new instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">`")]
                ///
                /// # Panic
                ///
                #[doc = $crate::macros::concat!("If an instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">` already exists")]
                // then panic
                pub fn new() -> $crate::typeid_tl::Type<Self> {
                    Self::try_new().expect($crate::macros::concat!(
                        "Cannot not create multiple `Type<",
                        $crate::macros::stringify!($ident),
                        ">`"
                    ))
                }
            }

            $crate::doc_item! {
                #[doc = $crate::macros::concat!("Try to create a new instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">`")]
                /// If an instance already exists, then return None
                pub fn try_new() -> $crate::macros::Option<$crate::typeid_tl::Type<Self>> {
                    let flag = unsafe {
                        Self::__make_typeid_tl_get_it(|flag| flag.take())
                    };

                    if flag {
                        unsafe {
                            $crate::macros::Option::Some($crate::typeid_tl::Type::new_unchecked(
                                Self($crate::macros::MacroConstructed::new()),
                                $crate::typeid_tl::TypeHandle::new(),
                            ))
                        }
                    } else {
                        $crate::macros::Option::None
                    }
                }
            }
        }

        impl $crate::macros::Drop for $ident {
            fn drop(&mut self) {
                unsafe {
                    Self::__make_typeid_tl_get_it(|flag| flag.reset())
                }
            }
        }
    };
}
