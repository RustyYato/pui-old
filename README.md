# pui

[documentation](https://docs.rs/pui)

A set of process unique identifiers that can be used to
identify types with minimal overhead within a single process

### features

* `std` (default) - if you have the `std` feature on, it will supercede the `alloc` feature.
    This allows you to use:
     * `std` types to implement various traits, for example `Box<I>` will implemnt `Identifier` `I`
     * `thread_local` types (from the `*_tl`)
     * `make_global_reuse` (this requires internal locking using a `Mutex`)

* `alloc` - this allows you to use without pulling in all of `std`:
     * `alloc` types to implement various traits, for example `Box<I>` will implemnt `Identifier` `I`

* `nightly` -  this allows you to use:
     * atomics on `no_std` targets that don't support 64-bit atomics

### synopsis

You are guaranteed that two instances of `Identifier` identifier will *never* compare equal
You can also create a cheap handle to the identifier, which you can use to mark other types
as logically owned by the identifier.

For example, this pattern is sound

```rust
use pui::Identifier;
use std::cell::UnsafeCell;

struct Owner<I> {
    ident: I,
}

struct Handle<H, T: ?Sized> {
    handle: H,
    value: UnsafeCell<T>,
}

impl<H, T> Handle<H, T> {
    pub fn new(handle: H, value: T) -> Self {
        Self { handle, value: UnsafeCell::new(value) }
    }
}

impl<I> Owner<I> {
    pub fn new(ident: I) -> Self {
        Self { ident }
    }
}

impl<I: Identifier> Owner<I> {
    pub fn read<'a, T: ?Sized>(&'a self, handle: &'a Handle<I::Handle, T>) -> &'a T {
        assert!(self.ident.owns(&handle.handle));
        
        // This is safe because `ident` owns the `handle`, which means that `self`
        // is the only `Owner` that could shared access the underlying value
        // This is because:
        //  * the `Owner` owns the `Identifier`
        //  * when we read/write, we bind the lifetime of `self` and `Handle` to the lifetime of
        //      the output reference
        //  * we have shared access to `*self`
        
        unsafe { &*handle.value.get() }
    }

    pub fn write<'a, T: ?Sized>(&'a mut self, handle: &'a Handle<I::Handle, T>) -> &'a mut T {
        assert!(self.ident.owns(&handle.handle));
        
        // This is safe because `ident` owns the `handle`, which means that `self`
        // is the only `Owner` that could exclusive access the underlying value
        // This is because:
        //  * the `Owner` owns the `Identifier`
        //  * when we read/write, we bind the lifetime of `self` and `Handle` to the lifetime of
        //      the output reference
        //  * we have exclusive access to `*self`
        
        unsafe { &mut *handle.value.get() }
    }
}
```

You can also use this for safe unchecked accesses

```rust
pub struct Slice<'a, T, I> {
    data: &'a [T],
    ident: I,
}

pub struct Idx<H> {
    index: usize,
    handle: H,
}

impl<'a, T, I: Identifier> Slice<'a, T, I> {
    fn from_ref(data: &'a [T], ident: I) -> Self {
        Self { data, ident }
    }

    pub fn get_indicies<'b>(&self) -> impl 'b + Iterator<Item = Idx<I::Handle>>
    where
        I::Handle: 'b,
    {
        (0..self.data.len())
            .zip(std::iter::repeat(self.ident.handle()))
            .map(move |(index, handle)| Idx { index, handle })
    }

    pub fn contains_index(&self, index: usize) -> Option<Idx<I::Handle>> {
        if index < self.data.len() {
            Some(Idx { index, handle: self.ident.handle() })
        } else {
            None
        }
    }
}

impl<'a, T, I: Identifier> std::ops::Index<Idx<I::Handle>> for Slice<'a, T, I> {
    type Output = T;

    fn index(&self, idx: Idx<I::Handle>) -> &Self::Output {
        /// This is safe because self owns `Idx<I::Handle>`
        /// If `I::Handle: Trivial`, then this assert is a no-op
        assert!(self.ident.owns(&idx.handle));
        unsafe { self.data.get_unchecked(idx.index) }
    }
}
```