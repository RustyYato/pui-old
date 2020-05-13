/// Create a new type that can be used with [`typeid::Type`](typeid::Type)
///
/// calling `make_typeid` like so,
/// ```
/// # #[cfg(feature = "atomic")]
/// pui::make_typeid! {
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
///     pub fn new() -> pui::typeid::Type<Self> {
///         Self::try_new().unwrap()
///     }
///
///     pub fn try_new() -> Option<pui::typeid::Type<Self>> {
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
macro_rules! make_typeid {
    ($(#[$meta:meta])*$v:vis once type $ident:ident;) => {
        $(#[$meta])*
        $v struct $ident;

        impl $ident {
            $crate::doc_item! {
                #[doc = $crate::macros::concat!("Create a new instance of `pui::typeid::Type<", $crate::macros::stringify!($ident), ">`")]
                ///
                /// # Panic
                ///
                #[doc = $crate::macros::concat!("If an instance of `pui::typeid::Type<", $crate::macros::stringify!($ident), ">` already exists")]
                // then panic
                pub fn new() -> $crate::typeid::Type<Self> {
                    Self::try_new().expect($crate::macros::concat!(
                        "Cannot not create multiple `Type<",
                        $crate::macros::stringify!($ident),
                        ">`"
                    ))
                }
            }

            $crate::doc_item! {
                #[doc = $crate::macros::concat!("Try to create a new instance of `pui::typeid::Type<", $crate::macros::stringify!($ident), ">`")]
                /// If an instance already exists, then return None
                pub fn try_new() -> $crate::macros::Option<$crate::typeid::Type<Self>> {
                    #[allow(non_upper_case_globals)]
                    static make_typeid_FLAG: $crate::macros::OnceFlag =
                        $crate::macros::OnceFlag::new();

                    if make_typeid_FLAG.take() {
                        unsafe {
                            $crate::macros::Option::Some($crate::typeid::Type::new_unchecked(
                                Self,
                                $crate::typeid::TypeHandle::new(),
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
            unsafe fn __make_typeid_get_it() -> &'static $crate::macros::ResettableOnceFlag {
                #[allow(non_upper_case_globals)]
                static make_typeid_FLAG: $crate::macros::ResettableOnceFlag =
                    $crate::macros::ResettableOnceFlag::new();

                &make_typeid_FLAG
            }

            $crate::doc_item! {
                #[doc = $crate::macros::concat!("Create a new instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">`")]
                ///
                /// # Panic
                ///
                #[doc = $crate::macros::concat!("If an instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">` already exists")]
                // then panic
                pub fn new() -> $crate::typeid::Type<Self> {
                    unsafe {
                        if Self::__make_typeid_get_it().acquire() {
                            $crate::typeid::Type::new_unchecked(
                                Self($crate::macros::MacroConstructed::new()),
                                $crate::typeid::TypeHandle::new(),
                            )
                        } else {
                            panic!($crate::macros::concat!(
                                "Cannot not create multiple `Type<",
                                $crate::macros::stringify!($ident),
                                ">` at the same time"
                            ))
                        }
                    }
                }
            }

            $crate::doc_item! {
                #[doc = $crate::macros::concat!("Try to create a new instance of `pui::typeid_tl::Type<", $crate::macros::stringify!($ident), ">`")]
                /// If an instance already exists, then return None
                pub fn try_new() -> $crate::macros::Option<$crate::typeid::Type<Self>> {
                    unsafe {
                        if Self::__make_typeid_get_it().try_acquire() {
                            $crate::macros::Option::Some($crate::typeid::Type::new_unchecked(
                                Self($crate::macros::MacroConstructed::new()),
                                $crate::typeid::TypeHandle::new(),
                            ))
                        } else {
                            $crate::macros::Option::None
                        }
                    }
                }
            }
        }

        impl $crate::macros::Drop for $ident {
            fn drop(&mut self) {
                unsafe {
                    Self::__make_typeid_get_it().release()
                }
            }
        }
    };
}

/// Create a new [`typeid::Type`](typeid::Type) that is guaranteed to be unique
///
/// ```
/// # use pui::{make_anon_typeid, Identifier};
/// let typeid = make_anon_typeid!();
/// assert!(typeid.owns(&typeid.handle()));
/// ```
///
#[macro_export]
macro_rules! make_anon_typeid {
    () => {
        unsafe { $crate::typeid::Type::new_unchecked(|| (), $crate::typeid::TypeHandle::new()) }
    };
}
