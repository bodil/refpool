// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::pointer::Pointer;
use crate::pool::Pool;
use crate::refbox::assume_init;
use crate::refbox::data_ptr;
use crate::refbox::RefBox;
use crate::PoolClone;
use crate::{types::ElementPointer, PoolDefault};
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::DerefMut;
use std::ptr::NonNull;
use std::{ops::Deref, pin::Pin};

/// A unique pointer to a pool allocated value of `A`.
pub struct PoolBox<A> {
    pub(crate) handle: ElementPointer<A>,
}

impl<A> PoolBox<A> {
    /// Construct a `PoolBox` with a newly initialised value of `A`.
    ///
    /// This uses [`PoolDefault::default_uninit()`][default_uninit] to initialise a
    /// default value, which may be faster than constructing a `PoolBox` from an
    /// existing value using [`PoolBox::new()`][new], depending on the data
    /// type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolBox};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let zero = PoolBox::default(&pool);
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
        .into_box()
    }

    /// Wrap a value in a `PoolBox`.
    ///
    /// This will copy the entire value into the memory handled by the
    /// `PoolBox`, which may be slower than using
    /// [`PoolBox::default()`][default], so it's not recommended to use this to
    /// construct the default value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolBox};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let number = PoolBox::new(&pool, 1337);
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
        .into_box()
    }

    /// Clone a value and return a new `PoolBox` to it.
    ///
    /// This will use [`PoolClone::clone_uninit()`][clone_uninit] to perform the
    /// clone, which may be more efficient than using
    /// [`PoolBox::new(value.clone())`][new].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolBox};
    /// let pool: Pool<Vec<usize>> = Pool::new(1);
    /// let vec = vec![1, 2, 3];
    /// let ref1 = PoolBox::clone_from(&pool, &vec);
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
        .into_box()
    }

    /// Construct a [`Pin`][Pin]ned `PoolBox` with a default value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolBox};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let zero = PoolBox::pin_default(&pool);
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

    /// Construct a [`Pin`][Pin]ned `PoolBox` with the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolBox};
    /// let pool: Pool<usize> = Pool::new(256);
    /// let number = PoolBox::pin(&pool, 1337);
    /// assert_eq!(1337, *number);
    /// ```
    ///
    /// [Pin]: https://doc.rust-lang.org/std/pin/struct.Pin.html
    pub fn pin(pool: &Pool<A>, value: A) -> Pin<Self> {
        unsafe { Pin::new_unchecked(Self::new(pool, value)) }
    }

    /// Test two `PoolBox`es for pointer equality.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolBox};
    /// let pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolBox::default(&pool);
    /// assert!(PoolBox::ptr_eq(&ref1, &ref1));
    /// ```
    pub fn ptr_eq(left: &Self, right: &Self) -> bool {
        std::ptr::eq(left.handle.get_ptr(), right.handle.get_ptr())
    }

    /// Consume the `PoolBox` and return a pointer to the contents.
    ///
    /// Please note that the only proper way to drop the value pointed to
    /// is by using `PoolBox::from_raw` to turn it back into a `PoolBox`, because
    /// the value is followed by `PoolBox` metadata which also needs to
    /// be dropped.
    pub fn into_raw_non_null(b: PoolBox<A>) -> NonNull<A> {
        let ptr = b.handle.cast();
        std::mem::forget(b);
        ptr
    }

    /// Consume the `PoolBox` and return a pointer to the contents.
    ///
    /// The pointer is guaranteed to be non-null.
    ///
    /// Please note that the only proper way to drop the value pointed to
    /// is by using `PoolBox::from_raw` to turn it back into a `PoolBox`, because
    /// the value is followed by `PoolBox` metadata which also needs to
    /// be dropped.
    pub fn into_raw(b: PoolBox<A>) -> *mut A {
        Self::into_raw_non_null(b).as_ptr()
    }

    /// Turn a raw pointer back into a `PoolBox`.
    ///
    /// The pointer must be non-null and obtained from a previous call to
    /// `PoolBox::into_raw` or `PoolBox::into_raw_non_null`.
    ///
    /// # Safety
    ///
    /// This must *only* be called on pointers obtained through `PoolBox::into_raw`.
    /// It's not OK to call it on a pointer to a value of `A` you've allocated
    /// yourself.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolBox};
    /// let pool: Pool<usize> = Pool::new(1);
    /// let ref1 = PoolBox::new(&pool, 31337);
    ///
    /// // Turn the PoolBox into a raw pointer and see if it still works.
    /// let ptr = PoolBox::into_raw(ref1);
    /// assert_eq!(31337, unsafe { *ptr });
    ///
    /// // Turn it back into a PoolBox and see, again, if it still works.
    /// let ref2 = unsafe { PoolBox::from_raw(ptr) };
    /// assert_eq!(31337, *ref2);
    /// ```
    pub unsafe fn from_raw(ptr: *mut A) -> Self {
        Self {
            handle: ElementPointer::wrap(ptr.cast()),
        }
    }

    fn box_ref(&self) -> &RefBox<A> {
        unsafe { &*self.handle.get_ptr() }
    }

    fn box_ref_mut(&mut self) -> &mut RefBox<A> {
        unsafe { &mut *self.handle.get_ptr() }
    }
}

impl<A> Drop for PoolBox<A> {
    fn drop(&mut self) {
        let handle = unsafe { Box::from_raw(self.handle.get_ptr()) };
        handle.return_to_pool();
    }
}

impl<A> Clone for PoolBox<A>
where
    A: PoolClone,
{
    /// Clone a `PoolBox` and its contents.
    ///
    /// This will use [`PoolClone::clone_uninit()`][clone_uninit] to perform the
    /// clone, which may be more efficient than using
    /// [`PoolBox::new(value.clone())`][new].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolBox};
    /// let pool: Pool<Vec<usize>> = Pool::new(1);
    /// let vec1 = PoolBox::new(&pool, vec![1, 2, 3]);
    /// let vec2 = vec1.clone();
    /// assert_eq!(vec1, vec2);
    /// ```
    ///
    /// [new]: #method.new
    /// [clone_uninit]: trait.PoolClone.html#tymethod.clone_uninit
    fn clone(&self) -> Self {
        let mut handle = self.box_ref().pool.pop();
        unsafe {
            self.clone_uninit(data_ptr(&mut handle));
            assume_init(handle)
        }
        .into_box()
    }
}

impl<A> Deref for PoolBox<A> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        self.box_ref().value_as_ref()
    }
}

impl<A> DerefMut for PoolBox<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.box_ref_mut().value_as_mut()
    }
}

impl<A> AsRef<A> for PoolBox<A> {
    fn as_ref(&self) -> &A {
        self.deref()
    }
}

impl<A> AsMut<A> for PoolBox<A> {
    fn as_mut(&mut self) -> &mut A {
        self.deref_mut()
    }
}

impl<A> Borrow<A> for PoolBox<A> {
    fn borrow(&self) -> &A {
        self.deref()
    }
}

impl<A> BorrowMut<A> for PoolBox<A> {
    fn borrow_mut(&mut self) -> &mut A {
        self.deref_mut()
    }
}

impl<A> PartialEq for PoolBox<A>
where
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (**self) == (**other)
    }
}

impl<A> Eq for PoolBox<A> where A: Eq {}

impl<A> PartialOrd for PoolBox<A>
where
    A: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<A> Ord for PoolBox<A>
where
    A: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<A> Hash for PoolBox<A>
where
    A: Hash,
{
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (**self).hash(hasher)
    }
}

impl<A> Display for PoolBox<A>
where
    A: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        (**self).fmt(f)
    }
}

impl<A> Debug for PoolBox<A>
where
    A: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        (**self).fmt(f)
    }
}

impl<A> std::fmt::Pointer for PoolBox<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        std::fmt::Pointer::fmt(&(&**self as *const A), f)
    }
}
