// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![forbid(rust_2018_idioms)]
#![deny(nonstandard_style)]
#![warn(unreachable_pub, missing_docs)]

//! A reimplementation of [`std::rc::Rc`][Rc] which uses a pool of reusable
//! memory to speed up reallocation.
//!
//! # Prerequisites
//!
//! In order to initialise a type to its default value from the memory pool
//! using [`PoolRef::default()`][PoolRef::default], it needs to implement
//! [`PoolDefault`][PoolDefault].
//!
//! If you want to be able to use [`PoolRef::make_mut()`][PoolRef::make_mut], it
//! also needs to implement [`PoolClone`][PoolClone].
//!
//! For constructing values using [`PoolRef::new()`][PoolRef::new], there's no
//! requirement.
//!
//! There are implementations for [`PoolDefault`][PoolDefault] and
//! [`PoolClone`][PoolClone] for most primitive types and a good selection of
//! `std`'s data types, and you can easily provide default implementations for
//! your own types by implementing the marker trait
//! [`PoolDefaultImpl`][PoolDefaultImpl]. You can also implement your own if you
//! have data structures whose memory doesn't need to be fully intitialised at
//! construction time, which can give you a slight performance boost. (This
//! optimisation is why [`PoolDefault`][PoolDefault] and
//! [`PoolClone`][PoolClone] exist as distinct traits, otherwise
//! [`Default`][Default] and [`Clone`][Clone] would have sufficed.)
//!
//! # Usage
//!
//! You create new values by calling
//! [`PoolRef::default(pool)`][PoolRef::default] or [`PoolRef::new(pool,
//! value)`][PoolRef::new]. This will use memory from the pool if available,
//! falling back to a normal heap allocation if the pool is empty.  When the
//! last [`PoolRef`][PoolRef] referencing the value is dropped, its allocated
//! memory is returned to the pool.
//!
//! # Differences from [`Rc`][Rc]
//!
//! [`PoolRef`][PoolRef] is API compatible with [`Rc`][Rc], with the following
//! exceptions:
//!
//!   * Types handled by the pool must be [`Sized`][Sized]. This means the pool
//!     won't accept trait objects, ie. no `Pool<dyn A>`.
//!   * Constructors need a [`Pool`][Pool] argument, so they're necessarily
//!     different: instead of [`Rc::new(value)`][Rc::new], you have
//!     [`PoolRef::default(pool)`][PoolRef::default] to construct a default
//!     value and [`PoolRef::new(pool, value)`][PoolRef::new] as the equivalent
//!     of [`Rc::new(value)`][Rc::new].
//!   * It does not implement [`Default`][Default], because you need a
//!     [`Pool`][Pool] argument to construct an instance. Use
//!     [`PoolRef::default(pool)`][PoolRef::default].
//!   * There's currently no equivalent to [`Weak`][Weak].
//!   * Experimental APIs are not implemented.

//! # Example
//!
//! ```rust
//! # use refpool::{Pool, PoolRef};
//! // Create a pool of `usize` with a max size of 1 (for argument's sake).
//! let mut pool: Pool<usize> = Pool::new(1);
//!
//! {
//!     // Create a reference handle to a usize initialised to 0.
//!     // The pool starts out empty, so this triggers a normal heap alloc.
//!     let value_ref = PoolRef::default(&mut pool);
//!     assert_eq!(0, *value_ref); // You can deref it just like `Rc`.
//! } // `value_ref` is dropped here, and its heap memory goes on the pool.
//!
//! // Check that we do indeed have one allocation in the pool now.
//! assert_eq!(1, pool.get_pool_size());
//!
//! // Create another reference and initialise it to 31337, a good round number.
//! // This will reuse `value_ref`'s memory.
//! let another_value_ref = PoolRef::new(&mut pool, 31337);
//! assert_eq!(31337, *another_value_ref);
//!
//! // Check that the pool is again empty after we reused the previous memory.
//! assert_eq!(0, pool.get_pool_size());
//! ```

//! [Pool]: struct.Pool.html
//! [PoolRef]: struct.PoolRef.html
//! [PoolRef::new]: struct.PoolRef.html#method.new
//! [PoolRef::default]: struct.PoolRef.html#method.default
//! [PoolRef::make_mut]: struct.PoolRef.html#method.make_mut
//! [PoolDefault]: trait.PoolDefault.html
//! [PoolClone]: trait.PoolClone.html
//! [PoolDefaultImpl]: trait.PoolDefaultImpl.html
//! [Default]: https://doc.rust-lang.org/std/default/trait.Default.html
//! [Clone]: https://doc.rust-lang.org/std/clone/trait.Clone.html
//! [Rc]: https://doc.rust-lang.org/std/rc/struct.Rc.html
//! [Rc::new]: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.new
//! [Weak]: https://doc.rust-lang.org/std/rc/struct.Weak.html
//! [Sized]: https://doc.rust-lang.org/std/marker/trait.Sized.html

use std::mem::MaybeUninit;

mod handle;
pub use self::handle::PoolRef;

use self::handle::RefBox;

mod std_types;
pub use self::std_types::PoolDefaultImpl;

/// A trait for initialising a `MaybeUninit<Self>` to a default value.
pub trait PoolDefault: Default {
    /// Initialise an instance of `Self` to its default state.
    ///
    /// Specifically, after calling `self.default_uninit()`, the object's state
    /// should be equal to what `<Self as Default>::default()` would produce.
    ///
    /// # Safety
    ///
    /// You should assume that the object as passed to you contains
    /// uninitialised memory, and you must leave it in a fully initialised
    /// state, as expected by `MaybeUninit::assume_init()`.
    unsafe fn default_uninit(target: &mut MaybeUninit<Self>);
}

/// A trait for cloning a value into a `MaybeUninit<Self>`.
pub trait PoolClone: PoolDefault + Clone {
    /// Clone an instance of `Self` into an uninitialised instance of `Self`.
    ///
    /// # Safety
    ///
    /// You should assume that the object as passed to you contains
    /// uninitialised memory, and you must leave it in a fully initialised
    /// state, as expected by `MaybeUninit::assume_init()`.
    unsafe fn clone_uninit(&self, target: &mut MaybeUninit<Self>);
}

struct Stack<A>(Vec<*mut A>);

impl<A> Stack<A> {
    fn new(max_size: usize) -> Self {
        Stack(Vec::with_capacity(max_size))
    }

    fn push(&mut self, value: *mut A) {
        self.0.push(value);
    }

    fn pop(&mut self) -> Option<*mut A> {
        self.0.pop()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

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
pub struct Pool<A> {
    inner: *mut PoolInner<A>,
}

impl<A> Pool<A> {
    /// Construct a new pool with a given max size.
    pub fn new(max_size: usize) -> Self {
        Box::new(PoolInner::new(max_size)).into_ref()
    }

    fn push(&self, value: *mut RefBox<A>) {
        unsafe { (*self.inner).push(value) }
    }

    fn pop(&self) -> Box<MaybeUninit<RefBox<A>>> {
        unsafe { (*self.inner).pop() }
    }

    fn deref(&self) -> &PoolInner<A> {
        unsafe { &*self.inner }
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
}

impl<A> Clone for Pool<A> {
    fn clone(&self) -> Self {
        unsafe { (*self.inner).make_ref() }
    }
}

impl<A> Drop for Pool<A> {
    fn drop(&mut self) {
        if unsafe { (*self.inner).dec() } == 0 {
            std::mem::drop(unsafe { Box::from_raw(self.inner) });
        }
    }
}

struct PoolInner<A> {
    count: usize,
    max_size: usize,
    stack: Stack<RefBox<A>>,
}

impl<A> PoolInner<A> {
    fn new(max_size: usize) -> Self {
        Self {
            count: 0,
            max_size,
            stack: Stack::new(max_size),
        }
    }

    fn into_ref(mut self: Box<Self>) -> Pool<A> {
        self.inc();
        Pool {
            inner: Box::into_raw(self),
        }
    }

    fn make_ref(&mut self) -> Pool<A> {
        self.inc();
        Pool { inner: self }
    }

    /// Get the maximum size of the pool.
    fn get_max_size(&self) -> usize {
        self.max_size
    }

    /// Get the current size of the pool.
    fn get_pool_size(&self) -> usize {
        self.stack.len()
    }

    fn inc(&mut self) {
        self.count += 1;
    }

    fn dec(&mut self) -> usize {
        self.count -= 1;
        self.count
    }

    unsafe fn init_box(ref_box: *mut RefBox<A>, pool: Pool<A>) {
        let count_ptr: *mut usize = &mut (*(ref_box)).count;
        let pool_ptr: *mut Pool<A> = &mut (*(ref_box)).pool;
        count_ptr.write(0);
        pool_ptr.write(pool);
    }

    fn pop(&mut self) -> Box<MaybeUninit<RefBox<A>>> {
        if let Some(value_ptr) = self.stack.pop() {
            let box_ptr = value_ptr.cast::<MaybeUninit<RefBox<A>>>();
            unsafe {
                let obj = box_ptr.as_mut().unwrap().as_mut_ptr();
                Self::init_box(obj, self.make_ref());
                Box::from_raw(box_ptr)
            }
        } else {
            let mut obj: Box<MaybeUninit<RefBox<A>>> = Box::new(MaybeUninit::uninit());
            unsafe { Self::init_box(obj.as_mut_ptr(), self.make_ref()) };
            obj
        }
    }

    fn push(&mut self, handle: *mut RefBox<A>) {
        self.stack.push(handle);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn allocate_and_deallocate_a_bit() {
        let pool = Pool::new(1024);
        assert_eq!(0, pool.get_pool_size());
        let mut refs: Vec<PoolRef<usize>> = Vec::new();
        for _ in 0..10000 {
            refs.push(PoolRef::default(&pool));
        }
        assert_eq!(0, pool.get_pool_size());
        refs.clear();
        assert_eq!(1024, pool.get_pool_size());
        for _ in 0..10000 {
            refs.push(PoolRef::default(&pool));
        }
        assert_eq!(0, pool.get_pool_size());
        let mut refs2 = refs.clone();
        assert_eq!(refs, refs2);
        for (left, right) in refs.iter().zip(refs2.iter()) {
            assert!(PoolRef::ptr_eq(left, right));
        }
        refs.clear();
        assert_eq!(0, pool.get_pool_size());
        refs2.clear();
        assert_eq!(1024, pool.get_pool_size());
    }
}
