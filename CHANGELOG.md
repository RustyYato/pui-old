# v 0.1.1

* Added `trait Trivial` for marking handles which are trivial to construct, and have no validity or safety invariants
* Added a feature gate for atomic ops
* Removed the `Default` bound for `impl<T, U: PoolMut<T>> Pool<T> for Mutex<U>`
  * This was a oversight from copypasta
* made all trivial functions (one line that just constructs something, or calls something) `#[inline]`
* made `typeid` wait for a new typeid when building with `std`

# v 0.1.0 - Initial Release
