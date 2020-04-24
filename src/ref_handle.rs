// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::pin::Pin;

use crate::counter::Counter;
use crate::pointer::Pointer;
use crate::refbox::{assume_init, data_ptr, RefBox};
use crate::types::ElementPointer;
use crate::{Pool, PoolClone, PoolDefault};

/// A reference counted pointer to a pool allocated value of `A`.
pub struct PoolRef<A> {
    pub(crate) handle: ElementPointer<A>,
}

impl<A> PoolRef<A> {
    /// Construct a `PoolRef` with a newly initialised value of `A`.
    ///
    /// This uses [`PoolDefault::default_uninit()`][default_uninit] to initialise a
    /// default value, which may be faster than constructing a `PoolRef` from an
    /// existing value using [`PoolRef::new()`][new], depending on the data
    /// type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let zero = PoolRef::default(&pool);
    /// assert_eq!(0, *zero);
    /// ```
    ///
    /// [new]: #method.new
    /// [default_uninit]: trait.PoolDefault.html#tymethod.default_uninit
    pub fn default(pool: &Pool<A>) -> Self
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
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let number = PoolRef::new(&pool, 1337);
    /// assert_eq!(1337, *number);
    /// ```
    ///
    /// [default]: #method.default
    pub fn new(pool: &Pool<A>, value: A) -> Self {
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
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<Vec<usize>> = Pool::new(1);
    /// let vec = vec![1, 2, 3];
    /// let ref1 = PoolRef::clone_from(&pool, &vec);
    /// assert_eq!(vec, *ref1);
    /// ```
    ///
    /// [new]: #method.new
    /// [clone_uninit]: trait.PoolClone.html#tymethod.clone_uninit
    pub fn clone_from(pool: &Pool<A>, value: &A) -> Self
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
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let zero = PoolRef::pin_default(&pool);
    /// assert_eq!(0, *zero);
    /// ```
    ///
    /// [Pin]: https://doc.rust-lang.org/std/pin/struct.Pin.html
    pub fn pin_default(pool: &Pool<A>) -> Pin<Self>
    where
        A: PoolDefault,
    {
        unsafe { Pin::new_unchecked(Self::default(pool)) }
    }

    /// Construct a [`Pin`][Pin]ned `PoolRef` with the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let number = PoolRef::pin(&pool, 1337);
    /// assert_eq!(1337, *number);
    /// ```
    ///
    /// [Pin]: https://doc.rust-lang.org/std/pin/struct.Pin.html
    pub fn pin(pool: &Pool<A>, value: A) -> Pin<Self> {
        unsafe { Pin::new_unchecked(Self::new(pool, value)) }
    }

    /// Clone the value inside a `PoolRef` and return a new `PoolRef` to it.
    ///
    /// This will use [`PoolClone::clone_uninit()`][clone_uninit] to perform
    /// the clone, which may be more efficient than using
    /// [`PoolRef::new((*this_ref).clone())`][new].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let mut number = PoolRef::new(&pool, 1337);
    /// let other_number = PoolRef::cloned(&pool, &number);
    /// *PoolRef::make_mut(&pool, &mut number) = 123;
    /// assert_eq!(123, *number);
    /// assert_eq!(1337, *other_number);
    /// ```
    ///
    /// [new]: #method.new
    /// [clone_uninit]: trait.PoolClone.html#tymethod.clone_uninit
    pub fn cloned(pool: &Pool<A>, this: &Self) -> Self
    where
        A: PoolClone,
    {
        let mut handle = pool.pop();
        unsafe {
            this.deref().clone_uninit(data_ptr(&mut handle));
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
    /// let pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::new(&pool, 1);
    /// let mut ref2 = ref1.clone();
    /// *PoolRef::make_mut(&pool, &mut ref2) = 2;
    /// assert_eq!(1, *ref1);
    /// assert_eq!(2, *ref2);
    /// ```
    pub fn make_mut<'a>(pool: &Pool<A>, this: &'a mut Self) -> &'a mut A
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
            this.handle = ElementPointer::wrap(Box::into_raw(new_handle));
        }
        this.box_ref_mut().value_as_mut()
    }

    /// Attempt to get a mutable reference to the value inside a `PoolRef`.
    ///
    /// This will produce a `None` if this `PoolRef` isn't a unique reference
    /// to the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(128);
    /// let mut number = PoolRef::new(&pool, 1337);
    /// assert_eq!(1337, *number);
    /// if let Some(number_ref) = PoolRef::get_mut(&mut number) {
    ///     *number_ref = 123;
    /// } else {
    ///     panic!("Couldn't get a unique reference!");
    /// }
    /// assert_eq!(123, *number);
    /// ```
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
    /// let pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::default(&pool);
    /// let ref2 = ref1.clone();
    /// let unwrap_result = PoolRef::try_unwrap(ref1);
    /// assert!(unwrap_result.is_err());
    /// ```
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::new(&pool, 1337);
    /// if let Ok(number) = PoolRef::try_unwrap(ref1) {
    ///     assert_eq!(1337, number);
    /// } else {
    ///     panic!("couldn't unwrap the number after all!");
    /// }
    /// ```
    pub fn try_unwrap(this: Self) -> Result<A, Self> {
        if this.box_ref().is_shared() {
            Err(this)
        } else {
            let handle = unsafe { Box::from_raw(this.handle.get_ptr()) };
            std::mem::forget(this);
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(1);
    /// let number = PoolRef::new(&pool, 1337);
    /// let other_ref = number.clone();
    /// assert_eq!(1337, PoolRef::unwrap_or_clone(other_ref));
    /// ```
    pub fn unwrap_or_clone(this: Self) -> A
    where
        A: PoolClone,
    {
        if this.box_ref().is_shared() {
            this.deref().clone()
        } else {
            let handle = unsafe { Box::from_raw(this.handle.get_ptr()) };
            std::mem::forget(this);
            handle.value
        }
    }

    /// Test two `PoolRef`s for pointer equality.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::default(&pool);
    /// let ref2 = ref1.clone();
    /// assert!(PoolRef::ptr_eq(&ref1, &ref2));
    /// ```
    pub fn ptr_eq(left: &Self, right: &Self) -> bool {
        std::ptr::eq(left.handle.get_ptr(), right.handle.get_ptr())
    }

    /// Get the current number of `LocalRef` references to the wrapped value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::default(&pool);
    /// let ref2 = ref1.clone();
    /// let ref3 = ref2.clone();
    /// assert_eq!(3, PoolRef::strong_count(&ref1));
    /// ```
    pub fn strong_count(this: &Self) -> usize {
        this.box_ref().count.count()
    }

    /// Consume the `PoolRef` and return a pointer to the contents.
    ///
    /// The pointer is guaranteed to be non-null.
    ///
    /// Please note that the only proper way to drop the value pointed to
    /// is by using `PoolRef::from_raw` to turn it back into a `PoolRef`, because
    /// the value is followed by `PoolRef` metadata which also needs to
    /// be dropped.
    pub fn into_raw(b: PoolRef<A>) -> *const A {
        let ptr = b.handle.cast();
        std::mem::forget(b);
        ptr.as_ptr()
    }

    /// Turn a raw pointer back into a `PoolRef`.
    ///
    /// The pointer must be non-null and obtained from a previous call to
    /// `PoolRef::into_raw` or `PoolRef::into_raw_non_null`.
    ///
    /// # Safety
    ///
    /// This must *only* be called on pointers obtained through `PoolRef::into_raw`.
    /// It's not OK to call it on a pointer to a value of `A` you've allocated
    /// yourself.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolRef::new(&pool, 31337);
    ///
    /// // Turn the PoolRef into a raw pointer and see if it still works.
    /// let ptr = PoolRef::into_raw(ref1);
    /// assert_eq!(31337, unsafe { *ptr });
    ///
    /// // Turn it back into a PoolRef and see, again, if it still works.
    /// let ref2 = unsafe { PoolRef::from_raw(ptr) };
    /// assert_eq!(31337, *ref2);
    /// ```
    pub unsafe fn from_raw(ptr: *const A) -> Self {
        Self {
            handle: ElementPointer::wrap((ptr as *mut A).cast()),
        }
    }

    fn box_ref(&self) -> &RefBox<A> {
        unsafe { &*self.handle.get_ptr() }
    }

    fn box_ref_mut(&mut self) -> &mut RefBox<A> {
        unsafe { &mut *self.handle.get_ptr() }
    }
}

impl<A> Drop for PoolRef<A> {
    fn drop(&mut self) {
        if self.box_ref_mut().dec() != 1 {
            return;
        }
        let handle = unsafe { Box::from_raw(self.handle.get_ptr()) };
        handle.return_to_pool();
    }
}

impl<A> Clone for PoolRef<A> {
    fn clone(&self) -> Self {
        let mut new_ref: Self = PoolRef {
            handle: ElementPointer::wrap(self.handle.get_ptr()),
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

impl<A> std::fmt::Pointer for PoolRef<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Pointer::fmt(&(&**self as *const A), f)
    }
}
