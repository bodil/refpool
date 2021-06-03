// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::{Debug, Error, Formatter};
use std::mem::MaybeUninit;

use crate::counter::Counter;
use crate::pointer::Pointer;
use crate::refbox::RefBox;
use crate::stack::Stack;
use crate::types::{ElementPointer, PoolPointer};

unsafe fn init_box<A>(ref_box: *mut RefBox<A>, pool: Pool<A>) {
    let count_ptr: *mut _ = &mut (*(ref_box)).count;
    let pool_ptr: *mut _ = &mut (*(ref_box)).pool;
    count_ptr.write(Default::default());
    pool_ptr.write(pool);
}

/// A pool of preallocated memory sized to match type `A`.
///
/// In order to use it to allocate objects, pass it to
/// [`PoolRef::new()`][PoolRef::new] or [`PoolRef::default()`][PoolRef::default].
///
/// # Example
///
/// ```rust
/// # use refpool::{Pool, PoolRef};
/// let mut pool: Pool<usize> = Pool::new(1024);
/// let pool_ref = PoolRef::new(&mut pool, 31337);
/// assert_eq!(31337, *pool_ref);
/// ```
///
/// [PoolRef::new]: struct.PoolRef.html#method.new
/// [PoolRef::default]: struct.PoolRef.html#method.default

pub struct Pool<A> {
    inner: PoolPointer<A>,
}

impl<A> Pool<A> {
    /// Construct a new pool with a given max size and return a handle to it.
    ///
    /// Values constructed via the pool will be returned to the pool when
    /// dropped, up to `max_size`. When the pool is full, values will be dropped
    /// in the regular way.
    ///
    /// If `max_size` is `0`, meaning the pool can never hold any dropped
    /// values, this method will give you back a null handle without allocating
    /// a pool. You can still use this to construct `PoolRef` values, they'll
    /// just allocate in the old fashioned way without using a pool. It is
    /// therefore advisable to use a zero size pool as a null value instead of
    /// `Option<Pool>`, which eliminates the need for unwrapping the `Option`
    /// value.
    pub fn new(max_size: usize) -> Self {
        if max_size == 0 {
            Self {
                inner: PoolPointer::null(),
            }
        } else {
            Box::new(PoolInner::new(max_size)).into_ref()
        }
    }

    pub(crate) fn push(&self, value: ElementPointer<A>) {
        debug_assert!(self.inner.get_ptr_checked().is_some());
        unsafe { (*self.inner.get_ptr()).push(value) }
    }

    pub(crate) fn pop(&self) -> Box<MaybeUninit<RefBox<A>>> {
        let mut obj = if let Some(inner) = self.inner.get_ptr_checked() {
            unsafe { (*inner).pop() }
        } else {
            None
        }
        .unwrap_or_else(|| Box::new(MaybeUninit::uninit()));
        unsafe { init_box(obj.as_mut_ptr(), self.clone()) };
        obj
    }

    fn deref(&self) -> Option<&PoolInner<A>> {
        self.inner.get_ptr_checked().map(|p| unsafe { &*p })
    }

    /// Get the maximum size of the pool.
    pub fn get_max_size(&self) -> usize {
        self.deref().map(|p| p.get_max_size()).unwrap_or(0)
    }

    /// Get the current size of the pool.
    pub fn get_pool_size(&self) -> usize {
        self.deref().map(|p| p.get_pool_size()).unwrap_or(0)
    }

    /// Test if the pool is currently full.
    pub fn is_full(&self) -> bool {
        self.deref()
            .map(|p| p.get_pool_size() >= p.get_max_size())
            .unwrap_or(true)
    }

    /// Fill the pool with empty allocations.
    ///
    /// This operation will pre-allocate `self.get_max_size() -
    /// self.get_pool_size()` memory chunks, without initialisation, and put
    /// them in the pool.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// let pool: Pool<usize> = Pool::new(1024);
    /// assert_eq!(0, pool.get_pool_size());
    /// pool.fill();
    /// assert_eq!(1024, pool.get_pool_size());
    /// ```
    pub fn fill(&self) {
        if let Some(inner) = self.deref() {
            while inner.get_max_size() > inner.get_pool_size() {
                let chunk = unsafe {
                    std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
                        std::mem::size_of::<RefBox<A>>(),
                        std::mem::align_of::<RefBox<A>>(),
                    ))
                };
                self.push(ElementPointer::wrap(chunk.cast()));
            }
        }
    }

    /// Fill the pool and return it.
    ///
    /// This is a convenience function that calls [`fill()`][fill] on
    /// the pool, so that you can construct a pool with a one liner:
    ///
    /// ```rust
    /// # use refpool::Pool;
    /// let pool: Pool<u64> = Pool::new(1024).filled();
    /// assert!(pool.is_full());
    /// ```
    ///
    /// This is functionally equivalent to, but terser than:
    ///
    /// ```rust
    /// # use refpool::Pool;
    /// let mut pool: Pool<u64> = Pool::new(1024);
    /// pool.fill();
    /// assert!(pool.is_full());
    /// ```
    ///
    /// [fill]: #method.fill
    pub fn filled(self) -> Self {
        self.fill();
        self
    }

    /// Convert a pool handle for type `A` into a handle for type `B`.
    ///
    /// The types `A` and `B` must have the same size and alignment, as
    /// per [`std::mem::size_of`][size_of] and
    /// [`std::mem::align_of`][align_of], or this method will panic.
    ///
    /// This lets you use the same pool to construct values of different
    /// types, as long as they are of the same size and alignment, so
    /// they can reuse each others' memory allocations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::{Pool, PoolRef};
    /// # use std::convert::TryInto;
    /// let u64_pool: Pool<u64> = Pool::new(1024);
    /// let u64_number = PoolRef::new(&u64_pool, 1337);
    ///
    /// let i64_pool: Pool<i64> = u64_pool.cast();
    /// let i64_number = PoolRef::new(&i64_pool, -1337);
    /// # assert_eq!(i64_number.abs().try_into(), Ok(*u64_number));
    /// ```
    ///
    /// [size_of]: https://doc.rust-lang.org/std/mem/fn.size_of.html
    /// [align_of]: https://doc.rust-lang.org/std/mem/fn.align_of.html
    pub fn cast<B>(&self) -> Pool<B> {
        assert!(std::mem::size_of::<A>() == std::mem::size_of::<B>());
        assert!(std::mem::align_of::<A>() >= std::mem::align_of::<B>());

        if let Some(ptr) = self.inner.get_ptr_checked() {
            let inner: *mut PoolInner<B> = ptr.cast();
            unsafe { (*inner).make_ref() }
        } else {
            Pool::new(0)
        }
    }
}

impl<A> Clone for Pool<A> {
    fn clone(&self) -> Self {
        if let Some(inner) = self.inner.get_ptr_checked() {
            unsafe { (*inner).make_ref() }
        } else {
            Self::new(0)
        }
    }
}

impl<A> Drop for Pool<A> {
    fn drop(&mut self) {
        if let Some(ptr) = self.inner.get_ptr_checked() {
            if unsafe { (*ptr).dec() } == 1 {
                std::mem::drop(unsafe { Box::from_raw(ptr) });
            }
        }
    }
}

impl<A> Debug for Pool<A> {
    /// Debug implementation for `Pool`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use refpool::Pool;
    /// let mut pool: Pool<usize> = Pool::new(256);
    /// assert!(format!("{:?}", pool).starts_with("Pool[0/256]:0x"));
    /// pool.fill();
    /// assert!(format!("{:?}", pool).starts_with("Pool[256/256]:0x"));
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Pool[{}/{}]:{:p}",
            self.get_pool_size(),
            self.get_max_size(),
            self.inner
        )
    }
}

pub(crate) struct PoolInner<A> {
    count: usize,
    max_size: usize,
    stack: Vec<ElementPointer<A>>,
}

impl<A> PoolInner<A> {
    fn new(max_size: usize) -> Self {
        Self {
            count: Default::default(),
            max_size,
            stack: Stack::stack_new(max_size),
        }
    }

    fn into_ref(mut self: Box<Self>) -> Pool<A> {
        self.inc();
        Pool {
            inner: PoolPointer::wrap(Box::into_raw(self)),
        }
    }

    fn make_ref(&mut self) -> Pool<A> {
        self.inc();
        Pool {
            inner: PoolPointer::wrap(self),
        }
    }

    /// Get the maximum size of the pool.
    fn get_max_size(&self) -> usize {
        self.max_size
    }

    /// Get the current size of the pool.
    fn get_pool_size(&self) -> usize {
        self.stack.stack_len()
    }

    #[inline(always)]
    fn inc(&mut self) {
        self.count.inc();
    }

    #[inline(always)]
    fn dec(&mut self) -> usize {
        self.count.dec()
    }

    fn pop(&mut self) -> Option<Box<MaybeUninit<RefBox<A>>>> {
        self.stack.stack_pop().map(|value_ptr| {
            let box_ptr = value_ptr.cast::<MaybeUninit<RefBox<A>>>();
            unsafe { Box::from_raw(box_ptr.as_ptr()) }
        })
    }

    fn push(&mut self, handle: ElementPointer<A>) {
        self.stack.stack_push(handle);
    }
}

impl<A> Drop for PoolInner<A> {
    fn drop(&mut self) {
        while let Some(chunk) = self.stack.stack_pop() {
            unsafe {
                std::alloc::dealloc(
                    chunk.as_ptr().cast(),
                    std::alloc::Layout::from_size_align_unchecked(
                        std::mem::size_of::<RefBox<A>>(),
                        std::mem::align_of::<RefBox<A>>(),
                    ),
                );
            }
        }
    }
}
