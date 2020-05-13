use super::RuntimeId;

/// A pool of ids that can be used to reuse ids.
pub trait PoolMut<T> {
    /// Put a new id into the pool
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>>;

    /// Take an idea out of the pool
    fn take_mut(&mut self) -> Option<RuntimeId<T>>;
}

/// A pool of ids that can be used to reuse ids.
pub trait Pool<T>: PoolMut<T> {
    /// Put a new id into the pool
    fn try_put(&self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>>;

    /// Take an idea out of the pool
    fn take(&self) -> Option<RuntimeId<T>>;
}

impl<P: ?Sized + Pool<T>, T> PoolMut<T> for &P {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        Pool::try_put_mut(self, value)
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        Pool::take_mut(self)
    }
}

impl<T> PoolMut<T> for () {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        Err(value)
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        None
    }
}

impl<T> Pool<T> for () {
    #[inline]
    fn try_put(&self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        Err(value)
    }

    #[inline]
    fn take(&self) -> Option<RuntimeId<T>> {
        None
    }
}

impl<T, R: ?Sized + PoolMut<T>> PoolMut<T> for &mut R {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        R::try_put_mut(self, value)
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        R::take_mut(self)
    }
}

impl<T, R: ?Sized + Pool<T>> Pool<T> for &R {
    #[inline]
    fn try_put(&self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        R::try_put(self, value)
    }

    #[inline]
    fn take(&self) -> Option<RuntimeId<T>> {
        R::take(self)
    }
}

use core::cell::{Cell, RefCell};
#[cfg(feature = "std")]
use std::sync::Mutex;

impl<T, U: PoolMut<T>> PoolMut<T> for Cell<U> {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        self.get_mut().try_put_mut(value)
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        self.get_mut().take_mut()
    }
}

impl<T, U: PoolMut<T>> PoolMut<T> for RefCell<U> {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        self.get_mut().try_put_mut(value)
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        self.get_mut().take_mut()
    }
}

impl<T, U: Default + PoolMut<T>> Pool<T> for Cell<U> {
    fn try_put(&self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        let mut inner = Cell::take(self);
        let output = inner.try_put_mut(value);
        self.set(inner);
        output
    }

    fn take(&self) -> Option<RuntimeId<T>> {
        let mut inner = Cell::take(self);
        let value = inner.take_mut();
        self.set(inner);
        value
    }
}

impl<T, U: PoolMut<T>> Pool<T> for RefCell<U> {
    fn try_put(&self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        if let Ok(mut x) = self.try_borrow_mut() {
            x.try_put_mut(value)
        } else {
            Err(value)
        }
    }

    fn take(&self) -> Option<RuntimeId<T>> {
        self.try_borrow_mut().ok()?.take_mut()
    }
}

#[cfg(feature = "std")]
impl<T, U: PoolMut<T>> PoolMut<T> for Mutex<U> {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        if let Ok(x) = self.get_mut() {
            x.try_put_mut(value)
        } else {
            Err(value)
        }
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        self.get_mut().ok()?.take_mut()
    }
}

#[cfg(feature = "std")]
impl<T, U: PoolMut<T>> Pool<T> for Mutex<U> {
    fn try_put(&self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        if let Ok(mut inner) = self.lock() {
            inner.try_put_mut(value)
        } else {
            Err(value)
        }
    }

    fn take(&self) -> Option<RuntimeId<T>> {
        self.lock().ok()?.take_mut()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
use std::{collections::VecDeque, vec::Vec};

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> PoolMut<T> for Vec<RuntimeId<T>> {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        self.push(value);
        Ok(())
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        self.pop()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> PoolMut<T> for VecDeque<RuntimeId<T>> {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        self.push_back(value);
        Ok(())
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        self.pop_front()
    }
}

impl<T> PoolMut<T> for Option<RuntimeId<T>> {
    #[inline]
    fn try_put_mut(&mut self, value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        if self.is_none() {
            *self = Some(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    #[inline]
    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        self.take()
    }
}

impl<P: PoolMut<T>, T> PoolMut<T> for [P] {
    fn try_put_mut(&mut self, mut value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        for slot in self.iter_mut() {
            if let Err(val) = slot.try_put_mut(value) {
                value = val;
            } else {
                return Ok(());
            }
        }

        Err(value)
    }

    fn take_mut(&mut self) -> Option<RuntimeId<T>> {
        self.iter_mut().map(PoolMut::take_mut).flatten().next()
    }
}

impl<P: Pool<T>, T> Pool<T> for [P] {
    fn try_put(&self, mut value: RuntimeId<T>) -> Result<(), RuntimeId<T>> {
        for slot in self.iter() {
            if let Err(val) = slot.try_put(value) {
                value = val;
            } else {
                return Ok(());
            }
        }

        Err(value)
    }

    fn take(&self) -> Option<RuntimeId<T>> {
        self.iter().map(Pool::take).flatten().next()
    }
}
