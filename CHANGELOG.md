# v 0.1.1

* Added `trait Trivial` for marking handles which are trivial to construct, and have no validity or safety invariants
* Removed the `Default` bound for `impl<T, U: PoolMut<T>> Pool<T> for Mutex<U>`
* made all trivial functions (one line that just constructs something, or calls something) `#[inline]`

# v 0.1.0 - Initial Release
