#![forbid(unsafe_code)]

/// A pool of ids that can be used to reuse ids.
pub trait PoolMut<T> {
    /// Put a new id into the pool
    fn try_put(&mut self, value: T) -> Result<(), T>;

    /// Take an idea out of the pool
    fn take(&mut self) -> Option<T>;
}

/// A pool of ids that can be used to reuse ids.
pub trait Pool<T> {
    /// Put a new id into the pool
    fn try_put(&self, value: T) -> Result<(), T>;

    /// Take an idea out of the pool
    fn take(&self) -> Option<T>;
}

impl<P: ?Sized + Pool<T>, T> PoolMut<T> for &P {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        Pool::try_put(self, value)
    }

    fn take(&mut self) -> Option<T> {
        Pool::take(self)
    }
}

impl<T> PoolMut<T> for () {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        Err(value)
    }

    fn take(&mut self) -> Option<T> {
        None
    }
}

impl<T> Pool<T> for () {
    fn try_put(&self, value: T) -> Result<(), T> {
        Err(value)
    }

    fn take(&self) -> Option<T> {
        None
    }
}

impl<T, R: ?Sized + PoolMut<T>> PoolMut<T> for &mut R {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        R::try_put(self, value)
    }

    fn take(&mut self) -> Option<T> {
        R::take(self)
    }
}

impl<T, R: ?Sized + Pool<T>> Pool<T> for &R {
    fn try_put(&self, value: T) -> Result<(), T> {
        R::try_put(self, value)
    }

    fn take(&self) -> Option<T> {
        R::take(self)
    }
}

use core::cell::{Cell, RefCell};
#[cfg(feature = "std")]
use std::sync::Mutex;

impl<T, U: PoolMut<T>> PoolMut<T> for Cell<U> {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        self.get_mut().try_put(value)
    }

    fn take(&mut self) -> Option<T> {
        self.get_mut().take()
    }
}

impl<T, U: PoolMut<T>> PoolMut<T> for RefCell<U> {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        self.get_mut().try_put(value)
    }

    fn take(&mut self) -> Option<T> {
        self.get_mut().take()
    }
}

impl<T, U: Default + PoolMut<T>> Pool<T> for Cell<U> {
    fn try_put(&self, value: T) -> Result<(), T> {
        let mut inner = Cell::take(self);
        let output = inner.try_put(value);
        self.set(inner);
        output
    }

    fn take(&self) -> Option<T> {
        let mut inner = Cell::take(self);
        let value = inner.take();
        self.set(inner);
        value
    }
}

impl<T, U: PoolMut<T>> Pool<T> for RefCell<U> {
    fn try_put(&self, value: T) -> Result<(), T> {
        if let Ok(mut x) = self.try_borrow_mut() {
            x.try_put(value)
        } else {
            Err(value)
        }
    }

    fn take(&self) -> Option<T> {
        self.try_borrow_mut().ok()?.take()
    }
}

#[cfg(feature = "std")]
impl<T, U: PoolMut<T>> PoolMut<T> for Mutex<U> {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        if let Ok(x) = self.get_mut() {
            x.try_put(value)
        } else {
            Err(value)
        }
    }

    fn take(&mut self) -> Option<T> {
        self.get_mut().ok()?.take()
    }
}

#[cfg(feature = "std")]
impl<T, U: Default + PoolMut<T>> Pool<T> for Mutex<U> {
    fn try_put(&self, value: T) -> Result<(), T> {
        if let Ok(mut inner) = self.lock() {
            inner.try_put(value)
        } else {
            Err(value)
        }
    }

    fn take(&self) -> Option<T> {
        self.lock().ok()?.take()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
use std::{collections::VecDeque, vec::Vec};

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> PoolMut<T> for Vec<T> {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        self.push(value);
        Ok(())
    }

    fn take(&mut self) -> Option<T> {
        self.pop()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> PoolMut<T> for VecDeque<T> {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        self.push_back(value);
        Ok(())
    }

    fn take(&mut self) -> Option<T> {
        self.pop_front()
    }
}

impl<T> PoolMut<T> for Option<T> {
    fn try_put(&mut self, value: T) -> Result<(), T> {
        if self.is_none() {
            *self = Some(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    fn take(&mut self) -> Option<T> {
        self.take()
    }
}

impl<P: PoolMut<T>, T> PoolMut<T> for [P] {
    fn try_put(&mut self, mut value: T) -> Result<(), T> {
        for slot in self.iter_mut() {
            if let Err(val) = slot.try_put(value) {
                value = val;
            } else {
                return Ok(());
            }
        }

        Err(value)
    }

    fn take(&mut self) -> Option<T> {
        self.iter_mut().map(PoolMut::take).flatten().next()
    }
}

impl<P: Pool<T>, T> Pool<T> for [P] {
    fn try_put(&self, mut value: T) -> Result<(), T> {
        for slot in self.iter() {
            if let Err(val) = slot.try_put(value) {
                value = val;
            } else {
                return Ok(());
            }
        }

        Err(value)
    }

    fn take(&self) -> Option<T> {
        self.iter().map(Pool::take).flatten().next()
    }
}
