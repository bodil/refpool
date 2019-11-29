// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::mem::MaybeUninit;

use crate::counter::Counter;
use crate::handle::RefBox;
use crate::pointer::Pointer;
use crate::stack::Stack;
use crate::sync_type::{PoolSyncType, PoolUnsync};

/// A pool of preallocated memory sized to match type `A`.
///
/// In order to use it to allocate objects, pass it to
/// [`PoolRef::new()`][PoolRef::new] or [`PoolRef::default()`][PoolRef::default].

/// # Example
///
/// ```rust
/// # use refpool::{Pool, PoolRef};
/// let mut pool: Pool<usize> = Pool::new(1024);
/// let pool_ref = PoolRef::new(&mut pool, 31337);
/// assert_eq!(31337, *pool_ref);
/// ```

/// [PoolRef::new]: struct.PoolRef.html#method.new
/// [PoolRef::default]: struct.PoolRef.html#method.default
pub struct Pool<A, S = PoolUnsync>
where
    S: PoolSyncType<A>,
{
    inner: S::PoolPointer,
}

impl<A, S> Pool<A, S>
where
    S: PoolSyncType<A>,
{
    /// Construct a new pool with a given max size.
    pub fn new(max_size: usize) -> Self {
        Box::new(PoolInner::new(max_size)).into_ref()
    }

    pub(crate) fn push(&self, value: S::ElementPointer) {
        unsafe { (*self.inner.get_ptr()).push(value) }
    }

    pub(crate) fn pop(&self) -> Box<MaybeUninit<RefBox<A, S>>> {
        unsafe { (*self.inner.get_ptr()).pop() }
    }

    fn deref(&self) -> &PoolInner<A, S> {
        unsafe { &*self.inner.get_ptr() }
    }

    /// Get the maximum size of the pool.
    pub fn get_max_size(&self) -> usize {
        self.deref().get_max_size()
    }

    /// Get the current size of the pool.
    pub fn get_pool_size(&self) -> usize {
        self.deref().get_pool_size()
    }

    /// Test if the pool is currently full.
    pub fn is_full(&self) -> bool {
        self.get_pool_size() >= self.get_max_size()
    }

    /// Fill the pool with empty allocations.
    ///
    /// This operation will pre-allocate `self.get_max_size() -
    /// self.get_pool_size()` memory chunks, without initialisation, and put
    /// them in the pool.
    pub fn fill(&self) {
        while self.get_max_size() > self.get_pool_size() {
            let chunk = unsafe {
                std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
                    std::mem::size_of::<RefBox<A, S>>(),
                    std::mem::align_of::<RefBox<A, S>>(),
                ))
            };
            self.push(S::ElementPointer::wrap(chunk.cast()));
        }
    }
}

impl<A, S> Clone for Pool<A, S>
where
    S: PoolSyncType<A>,
{
    fn clone(&self) -> Self {
        unsafe { (*self.inner.get_ptr()).make_ref() }
    }
}

impl<A, S> Drop for Pool<A, S>
where
    S: PoolSyncType<A>,
{
    fn drop(&mut self) {
        let ptr = self.inner.get_ptr();
        if unsafe { (*ptr).dec() } == 1 {
            std::mem::drop(unsafe { Box::from_raw(ptr) });
        }
    }
}

#[doc(hidden)]
pub struct PoolInner<A, S>
where
    S: PoolSyncType<A>,
{
    count: S::Counter,
    max_size: usize,
    stack: S::Stack,
}

impl<A, S> PoolInner<A, S>
where
    S: PoolSyncType<A>,
{
    fn new(max_size: usize) -> Self {
        Self {
            count: Default::default(),
            max_size,
            stack: Stack::stack_new(max_size),
        }
    }

    fn into_ref(mut self: Box<Self>) -> Pool<A, S> {
        self.inc();
        Pool {
            inner: S::PoolPointer::wrap(Box::into_raw(self)),
        }
    }

    fn make_ref(&mut self) -> Pool<A, S> {
        self.inc();
        Pool {
            inner: S::PoolPointer::wrap(self),
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

    unsafe fn init_box(ref_box: *mut RefBox<A, S>, pool: Pool<A, S>) {
        let count_ptr: *mut _ = &mut (*(ref_box)).count;
        let pool_ptr: *mut _ = &mut (*(ref_box)).pool;
        count_ptr.write(Default::default());
        pool_ptr.write(pool);
    }

    fn pop(&mut self) -> Box<MaybeUninit<RefBox<A, S>>> {
        if let Some(value_ptr) = self.stack.stack_pop() {
            let box_ptr = value_ptr.cast::<MaybeUninit<RefBox<A, S>>>();
            unsafe {
                let obj = box_ptr.as_mut().unwrap().as_mut_ptr();
                Self::init_box(obj, self.make_ref());
                Box::from_raw(box_ptr)
            }
        } else {
            let mut obj: Box<MaybeUninit<RefBox<A, S>>> = Box::new(MaybeUninit::uninit());
            unsafe { Self::init_box(obj.as_mut_ptr(), self.make_ref()) };
            obj
        }
    }

    fn push(&mut self, handle: S::ElementPointer) {
        self.stack.stack_push(handle);
    }
}
