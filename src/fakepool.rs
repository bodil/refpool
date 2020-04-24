// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Fake versions of `Pool`, `PoolRef` and `PoolBox`.
//!
//! This module provides zero cost wrappers for `Box` and `Rc`, as well as
//! a zero sized empty `Pool`, which are code compatible with their
//! real counterparts. These can be useful if you're writing code that only
//! optionally uses pooled allocation, allowing you to use the same code for
//! both situations, differing only in which versions of `Pool` and friends
//! you choose to import.

#![allow(dead_code, missing_docs, clippy::missing_safety_doc)]

use std::marker::PhantomData;
use std::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt::{Debug, Display, Error, Formatter},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    pin::Pin,
    rc::Rc,
};

use crate::{PoolClone, PoolDefault};

/// A fake `Pool` which is always empty.
///
/// Note that, unlike its non-fake counterpart, this pool will always report its
/// current and maximum sizes as zero, regardless of the value passed into the
/// constructor, and it will always report itself as being full, to be consistent
/// with the reported sizes. You should therefore avoid assuming that the size
/// passed into `Pool::new(size)` will have any bearing on the actual size of the
/// pool if you're writing code that might be using a fake pool.
pub struct Pool<A>(PhantomData<A>);

impl<A> Pool<A> {
    pub fn new(_size: usize) -> Self {
        Pool(PhantomData)
    }

    pub fn get_max_size(&self) -> usize {
        0
    }

    pub fn get_pool_size(&self) -> usize {
        0
    }

    pub fn is_full(&self) -> bool {
        true
    }

    pub fn fill(&self) {}

    pub fn cast<B>(&self) -> Pool<B> {
        Pool(PhantomData)
    }
}

impl<A> Clone for Pool<A> {
    fn clone(&self) -> Self {
        Self::new(0)
    }
}

impl<A> Debug for Pool<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "FakePool")
    }
}

/// A fake `PoolRef` which wraps an `Rc`.
#[derive(Default)]
pub struct PoolRef<A>(Rc<A>);

impl<A> PoolRef<A> {
    #[inline(always)]
    pub fn default(_pool: &Pool<A>) -> Self
    where
        A: PoolDefault,
    {
        Self(Default::default())
    }

    #[inline(always)]
    pub fn new(_pool: &Pool<A>, value: A) -> Self {
        Self(Rc::new(value))
    }

    #[inline(always)]
    pub fn clone_from(_pool: &Pool<A>, value: &A) -> Self
    where
        A: PoolClone,
    {
        Self(Rc::new(value.clone()))
    }

    #[inline(always)]
    pub fn pin_default(_pool: &Pool<A>) -> Pin<Self>
    where
        A: PoolDefault,
    {
        unsafe { Pin::new_unchecked(Self(Rc::new(A::default()))) }
    }

    #[inline(always)]
    pub fn pin(_pool: &Pool<A>, value: A) -> Pin<Self> {
        unsafe { Pin::new_unchecked(Self(Rc::new(value))) }
    }

    #[inline(always)]
    pub fn cloned(_pool: &Pool<A>, this: &Self) -> Self
    where
        A: PoolClone,
    {
        Self(Rc::new(this.deref().clone()))
    }

    #[inline(always)]
    pub fn make_mut<'a>(_pool: &Pool<A>, this: &'a mut Self) -> &'a mut A
    where
        A: PoolClone,
    {
        Rc::make_mut(&mut this.0)
    }

    #[inline(always)]
    pub fn get_mut(this: &mut Self) -> Option<&mut A> {
        Rc::get_mut(&mut this.0)
    }

    #[inline(always)]
    pub fn try_unwrap(this: Self) -> Result<A, Self> {
        Rc::try_unwrap(this.0).map_err(Self)
    }

    #[inline(always)]
    pub fn unwrap_or_clone(this: Self) -> A
    where
        A: PoolClone,
    {
        Self::try_unwrap(this).unwrap_or_else(|this| this.deref().clone())
    }

    #[inline(always)]
    pub fn ptr_eq(left: &Self, right: &Self) -> bool {
        Rc::ptr_eq(&left.0, &right.0)
    }

    #[inline(always)]
    pub fn strong_count(this: &Self) -> usize {
        Rc::strong_count(&this.0)
    }

    #[inline(always)]
    pub fn into_raw(this: PoolRef<A>) -> *const A {
        Rc::into_raw(this.0)
    }

    #[inline(always)]
    pub unsafe fn from_raw(ptr: *mut A) -> Self {
        Self(Rc::from_raw(ptr))
    }
}

impl<A> Clone for PoolRef<A> {
    #[inline(always)]
    fn clone(&self) -> Self {
        PoolRef(self.0.clone())
    }
}

impl<A> Deref for PoolRef<A> {
    type Target = A;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
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
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
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

/// A fake `PoolBox` which wraps a `Box`.
pub struct PoolBox<A>(Box<A>);

impl<A> PoolBox<A> {
    #[inline(always)]
    pub fn default(_pool: &Pool<A>) -> Self
    where
        A: PoolDefault,
    {
        Self(Box::new(A::default()))
    }

    #[inline(always)]
    pub fn new(_pool: &Pool<A>, value: A) -> Self {
        Self(Box::new(value))
    }

    #[inline(always)]
    pub fn clone_from(_pool: &Pool<A>, value: &A) -> Self
    where
        A: PoolClone,
    {
        Self(Box::new(value.clone()))
    }

    #[inline(always)]
    pub fn pin_default(_pool: &Pool<A>) -> Pin<Self>
    where
        A: PoolDefault,
    {
        unsafe { Pin::new_unchecked(Self(Box::new(A::default()))) }
    }

    #[inline(always)]
    pub fn pin(_pool: &Pool<A>, value: A) -> Pin<Self> {
        unsafe { Pin::new_unchecked(Self(Box::new(value))) }
    }

    #[inline(always)]
    pub fn ptr_eq(left: &Self, right: &Self) -> bool {
        std::ptr::eq(left.0.deref(), right.0.deref())
    }

    #[inline(always)]
    pub fn into_raw(this: Self) -> *mut A {
        Box::into_raw(this.0)
    }

    #[inline(always)]
    pub unsafe fn from_raw(ptr: *mut A) -> Self {
        Self(Box::from_raw(ptr))
    }
}

impl<A> Deref for PoolBox<A> {
    type Target = A;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<A> DerefMut for PoolBox<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
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
