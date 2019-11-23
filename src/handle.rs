// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Error, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::pin::Pin;

use crate::{Pool, PoolClone, PoolDefault};

unsafe fn assume_init<A>(maybe_boxed: Box<MaybeUninit<A>>) -> Box<A> {
    Box::from_raw(Box::into_raw(maybe_boxed).cast())
    // TODO: Change this to `maybe_boxed.assume_init()` when the `new_uninit`
    // feature stabilises.
}

unsafe fn data_ptr<A>(this: &mut MaybeUninit<RefBox<A>>) -> &mut MaybeUninit<A> {
    (*this.as_mut_ptr())
        .value_as_mut_ptr()
        .cast::<MaybeUninit<A>>()
        .as_mut()
        .unwrap()
}

/// A reference counted pointer to `A`.
pub struct PoolRef<A> {
    pub(crate) handle: *mut RefBox<A>,
}

impl<A> PoolRef<A> {
    /// Construct a `PoolRef` with a newly initialised value of `A`.
    ///
    /// This uses [`PoolDefault::default_uninit()`][default_uninit] to initialise a
    /// default value, which may be faster than constructing a `PoolRef` from an
    /// existing value using [`PoolRef::new()`][new], depending on the data
    /// type.
    ///
    /// [new]: #method.new
    /// [default_uninit]: trait.PoolDefault.html#tymethod.default_uninit
    pub fn default(pool: &mut Pool<A>) -> Self
    where
        A: PoolDefault,
    {
        let mut handle = pool.pop();
        unsafe {
            PoolDefault::default_uninit(data_ptr(&mut handle));
            assume_init(handle)
        }
        .into_ref()
    }

    /// Wrap a value in a `PoolRef`.
    ///
    /// This will copy the entire value into the memory handled by the
    /// `PoolRef`, which may be slower than using
    /// [`PoolRef::default()`][default], so it's not recommended to use this to
    /// construct the default value.
    ///
    /// [default]: #method.default
    pub fn new(pool: &mut Pool<A>, value: A) -> Self {
        let mut handle = pool.pop();
        unsafe {
            data_ptr(&mut handle).as_mut_ptr().write(value);
            assume_init(handle)
        }
        .into_ref()
    }

    /// Clone a value and return a new `PoolRef` to it.
    ///
    /// This will use [`PoolClone::clone_uninit()`][clone_uninit] to perform the
    /// clone, which may be more efficient than using
    /// [`PoolRef::new(value.clone())`][new].
    ///
    /// [new]: #method.new
    /// [clone_uninit]: trait.PoolClone.html#tymethod.clone_uninit

    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let mut pool: Pool<Vec<usize>> = Pool::new(1);
    /// let vec = vec![1, 2, 3];
    /// let ref1 = PoolRef::clone_from(&mut pool, &vec);
    /// assert_eq!(vec, *ref1);
    /// ```
    pub fn clone_from(pool: &mut Pool<A>, value: &A) -> Self
    where
        A: PoolClone,
    {
        let mut handle = pool.pop();
        unsafe {
            value.clone_uninit(data_ptr(&mut handle));
            assume_init(handle)
        }
        .into_ref()
    }

    /// Construct a [`Pin`][Pin]ned `PoolRef` with a default value.
    ///
    /// [Pin]: https://doc.rust-lang.org/std/pin/struct.Pin.html
    pub fn pin_default(pool: &mut Pool<A>) -> Pin<Self>
    where
        A: PoolDefault,
    {
        unsafe { Pin::new_unchecked(Self::default(pool)) }
    }

    /// Construct a [`Pin`][Pin]ned `PoolRef` with the given value.
    ///
    /// [Pin]: https://doc.rust-lang.org/std/pin/struct.Pin.html
    pub fn pin(pool: &mut Pool<A>, value: A) -> Pin<Self> {
        unsafe { Pin::new_unchecked(Self::new(pool, value)) }
    }

    /// Clone the value inside a `PoolRef` and return a new `PoolRef` to it.
    ///
    /// This will use [`PoolClone::clone_uninit()`][clone_uninit] to perform
    /// the clone, which may be more efficient than using
    /// [`PoolRef::new((*this_ref).clone())`][new].
    ///
    /// [new]: #method.new
    /// [clone_uninit]: trait.PoolClone.html#tymethod.clone_uninit
    pub fn cloned(&self, pool: &mut Pool<A>) -> Self
    where
        A: PoolClone,
    {
        let mut handle = pool.pop();
        unsafe {
            self.clone_uninit(data_ptr(&mut handle));
            assume_init(handle)
        }
        .into_ref()
    }

    /// Get a mutable reference to the value inside a `PoolRef`, cloning it
    /// first if this `PoolRef` isn't a unique reference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let mut pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::new(&mut pool, 1);
    /// let mut ref2 = ref1.clone();
    /// *PoolRef::make_mut(&mut pool, &mut ref2) = 2;
    /// assert_eq!(1, *ref1);
    /// assert_eq!(2, *ref2);
    /// ```
    pub fn make_mut<'a>(pool: &mut Pool<A>, this: &'a mut Self) -> &'a mut A
    where
        A: PoolClone,
    {
        if this.box_ref().is_shared() {
            let mut new_handle = pool.pop();
            let mut new_handle = unsafe {
                this.clone_uninit(data_ptr(&mut new_handle));
                assume_init(new_handle)
            };
            new_handle.inc();
            this.box_ref_mut().dec();
            this.handle = Box::into_raw(new_handle);
        }
        this.box_ref_mut().value_as_mut()
    }

    /// Attempt to get a mutable reference to the value inside a `PoolRef`.
    ///
    /// This will produce a `None` if this `PoolRef` isn't a unique reference
    /// to the value.
    pub fn get_mut(this: &mut Self) -> Option<&mut A> {
        let handle = this.box_ref_mut();
        if handle.is_shared() {
            None
        } else {
            Some(handle.value_as_mut())
        }
    }

    /// Attempt to unwrap the value inside a `PoolRef`.
    ///
    /// If this `PoolRef` isn't the only reference to the value, ownership of
    /// the `PoolRef` is passed back to you in the `Err` value.
    ///
    /// Please note that the unwrapped value is not reclaimed by the pool when
    /// dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let mut pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::default(&mut pool);
    /// let ref2 = ref1.clone();
    /// let unwrap_result = PoolRef::try_unwrap(ref1);
    /// assert!(unwrap_result.is_err());
    /// ```
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let mut pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::new(&mut pool, 1337);
    /// if let Ok(number) = PoolRef::try_unwrap(ref1) {
    ///     assert_eq!(1337, number);
    /// } else {
    ///     panic!("couldn't unwrap the number after all!");
    /// }
    /// ```
    pub fn try_unwrap(mut this: Self) -> Result<A, Self> {
        if this.box_ref().is_shared() {
            Err(this)
        } else {
            let handle = unsafe { Box::from_raw(this.handle) };
            this.handle = std::ptr::null_mut();
            Ok(handle.value)
        }
    }

    /// Unwrap the value inside a `PoolRef`, cloning if necessary.
    ///
    /// If this `PoolRef` is a unique reference to the value, the value is
    /// unwrapped and returned, consuming the `PoolRef`. Otherwise, the value
    /// is cloned and the clone is returned.
    ///
    /// Please note that the unwrapped value is not reclaimed by the pool when
    /// dropped.
    pub fn unwrap_or_clone(mut this: Self) -> A
    where
        A: PoolClone,
    {
        if this.box_ref().is_shared() {
            this.deref().clone()
        } else {
            let handle = unsafe { Box::from_raw(this.handle) };
            this.handle = std::ptr::null_mut();
            handle.value
        }
    }

    /// Test two `PoolRef`s for pointer equality.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let mut pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::default(&mut pool);
    /// let ref2 = ref1.clone();
    /// assert!(PoolRef::ptr_eq(&ref1, &ref2));
    /// ```
    pub fn ptr_eq(left: &Self, right: &Self) -> bool {
        std::ptr::eq(left.handle, right.handle)
    }

    /// Get the current number of `LocalRef` references to the wrapped value.
    pub fn strong_count(this: &Self) -> usize {
        let handle = unsafe { &mut (*this.handle) };
        handle.count
    }

    fn box_ref(&self) -> &RefBox<A> {
        unsafe { &*self.handle }
    }

    fn box_ref_mut(&mut self) -> &mut RefBox<A> {
        unsafe { &mut *self.handle }
    }
}

impl<A> Drop for PoolRef<A> {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }
        if self.box_ref_mut().dec() > 0 {
            return;
        }
        let handle = unsafe { Box::from_raw(self.handle) };
        handle.return_to_pool();
    }
}

impl<A> Clone for PoolRef<A> {
    fn clone(&self) -> Self {
        let mut new_ref = PoolRef {
            handle: self.handle,
        };
        new_ref.box_ref_mut().inc();
        new_ref
    }
}

impl<A> Deref for PoolRef<A> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        self.box_ref().value_as_ref()
    }
}

impl<A> AsRef<A> for PoolRef<A> {
    fn as_ref(&self) -> &A {
        self.deref()
    }
}

impl<A> Borrow<A> for PoolRef<A> {
    fn borrow(&self) -> &A {
        self.deref()
    }
}

impl<A> PartialEq for PoolRef<A>
where
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (**self) == (**other)
    }
}

impl<A> Eq for PoolRef<A> where A: Eq {}

impl<A> PartialOrd for PoolRef<A>
where
    A: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<A> Ord for PoolRef<A>
where
    A: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<A> Hash for PoolRef<A>
where
    A: Hash,
{
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (**self).hash(hasher)
    }
}

impl<A> Display for PoolRef<A>
where
    A: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        (**self).fmt(f)
    }
}

impl<A> Debug for PoolRef<A>
where
    A: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        (**self).fmt(f)
    }
}

impl<A> Pointer for PoolRef<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        Pointer::fmt(&(&**self as *const A), f)
    }
}

// RefBox

pub(crate) struct RefBox<A> {
    pub(crate) count: usize,
    pub(crate) pool: *mut Pool<A>,
    pub(crate) value: A,
}

impl<A> RefBox<A> {
    fn into_ref(mut self: Box<Self>) -> PoolRef<A> {
        let ref_handle = self.new_ref();
        Box::leak(self);
        ref_handle
    }

    fn new_ref(&mut self) -> PoolRef<A> {
        self.count += 1;
        PoolRef { handle: self }
    }

    fn return_to_pool(self: Box<Self>) {
        let pool = unsafe { &mut *self.pool };
        if !pool.is_full() {
            let ptr = Box::into_raw(self);
            unsafe { ptr.drop_in_place() };
            pool.push(ptr);
        }
    }

    fn value_as_ref(&self) -> &A {
        &self.value
    }

    fn value_as_mut(&mut self) -> &mut A {
        &mut self.value
    }

    unsafe fn value_as_mut_ptr(&mut self) -> *mut A {
        &mut self.value
    }

    fn inc(&mut self) {
        self.count += 1;
    }

    fn dec(&mut self) -> usize {
        self.count -= 1;
        self.count
    }

    fn is_shared(&self) -> bool {
        self.count > 1
    }
}
