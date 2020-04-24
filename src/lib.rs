// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! A reimplementation of [`std::boxed::Box`][Box] and [`std::rc::Rc`][Rc]
//! which uses a pool of reusable memory to speed up reallocation.
//!
//! # Prerequisites
//!
//! In order to initialise a type to its default value from the memory pool
//! using [`PoolBox::default()`][PoolBox::default] or
//! [`PoolRef::default()`][PoolRef::default], it needs to implement
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
//! # Differences from [`Box`][Box] and [`Rc`][Rc]
//!
//! [`PoolBox`][PoolBox] is API compatible with [`Box`][Box] and [`PoolRef`][PoolRef]
//! with [`Rc`][Rc], with the following exceptions:
//!
//!   * Types handled by the pool must be [`Sized`][Sized]. This means the pool
//!     won't accept trait objects, ie. no `Pool<dyn A>`.
//!   * Constructors need a [`Pool`][Pool] argument, so they're necessarily
//!     different: instead of [`Rc::new(value)`][Rc::new], you have
//!     [`PoolRef::default(pool)`][PoolRef::default] to construct a default
//!     value and [`PoolRef::new(pool, value)`][PoolRef::new] as the equivalent
//!     of [`Rc::new(value)`][Rc::new]. Likewise for [`PoolBox`][PoolBox].
//!   * [`PoolBox`][PoolBox] and [`PoolRef`][PoolRef] do not implement
//!     [`Default`][Default], because you need a
//!     [`Pool`][Pool] argument to construct an instance. Use
//!     [`PoolRef::default(pool)`][PoolRef::default].
//!   * There's currently no equivalent to [`Weak`][Weak] for [`PoolRef`][PoolRef].
//!   * Experimental APIs are not implemented.
//!
//! # Thread Safety
//!
//! [`Pool`][Pool] is strictly thread local, ie. it does not
//! implement [`Sync`][Sync] and it will fail in appalling ways if you still
//! somehow manage to access it from two different threads. There is no
//! equivalent of [`Arc`][Arc] because adding thread safety to the pool turns
//! out to degrade performance sufficiently that the pool is no longer providing
//! a significant performance benefit even with the slowest system allocators
//! you're likely to come across in the wild (by which I mean Windows).
//!
//! # Performance
//!
//! You can expect [`Pool`][Pool] to always outperform the system allocator,
//! though the performance gains will vary between platforms. Preliminary
//! benchmarks show it's approximately twice as fast on Linux, and 5-6 times as
//! fast on Windows. Custom allocators like jemalloc may yield even less
//! benefit, but it's very unlikely you'll find an allocator that can outperform
//! the pool.
//!
//! You can expect bigger performance gains from data types with beneficial
//! [`PoolDefault`][PoolDefault] and [`PoolClone`][PoolClone] implementations,
//! "beneficial" in this case meaning cases where you can leave most of the
//! allocated memory uninitialised. [`sized_chunks::Chunk`][Chunk], which
//! allocates 528 bytes on 64-bit platforms but only needs to initialise 16
//! of them for [`PoolDefault`][PoolDefault], would be a good example of this.

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
//!
//! # Feature Flags
//!
//! There's one feature flag available, `default_impl`, which requires a nightly
//! rustc because it leans on the `min_specialization` language feature, which
//! removes the `PoolDefaultImpl` trait and instead provides a `default`
//! overridable implementation for `PoolClone` and `PoolDefault` for any type
//! that implements `Clone` and `Default`. `PoolDefaultImpl` is an unfortunate
//! hack to get around the current absence of specialisation in stable rustc.
//!
//! [Pool]: struct.Pool.html
//! [PoolBox]: struct.PoolBox.html
//! [PoolBox::default]: struct.PoolBox.html#method.default
//! [PoolRef]: struct.PoolRef.html
//! [PoolRef::new]: struct.PoolRef.html#method.new
//! [PoolRef::default]: struct.PoolRef.html#method.default
//! [PoolRef::make_mut]: struct.PoolRef.html#method.make_mut
//! [PoolDefault]: trait.PoolDefault.html
//! [PoolClone]: trait.PoolClone.html
//! [PoolDefaultImpl]: trait.PoolDefaultImpl.html
//! [PoolSync]: struct.PoolSync.html
//! [Box]: https://doc.rust-lang.org/stable/std/boxed/struct.Box.html
//! [Box::from_raw]: https://doc.rust-lang.org/stable/std/boxed/struct.Box.html#method.from_raw
//! [Box::into_raw]: https://doc.rust-lang.org/stable/std/boxed/struct.Box.html#method.into_raw
//! [Default]: https://doc.rust-lang.org/std/default/trait.Default.html
//! [Clone]: https://doc.rust-lang.org/std/clone/trait.Clone.html
//! [Arc]: https://doc.rust-lang.org/std/sync/struct.Arc.html
//! [Rc]: https://doc.rust-lang.org/std/rc/struct.Rc.html
//! [Rc::new]: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.new
//! [Weak]: https://doc.rust-lang.org/std/rc/struct.Weak.html
//! [Sized]: https://doc.rust-lang.org/std/marker/trait.Sized.html
//! [Sync]: https://doc.rust-lang.org/std/marker/trait.Sync.html
//! [Chunk]: https://docs.rs/sized-chunks/*/sized_chunks/sized_chunk/struct.Chunk.html

#![forbid(rust_2018_idioms)]
#![deny(nonstandard_style)]
#![warn(
    unreachable_pub,
    missing_docs,
    missing_debug_implementations,
    missing_doc_code_examples
)]
#![cfg_attr(feature = "default_impl", feature(min_specialization))]

use std::mem::MaybeUninit;

mod box_handle;
mod counter;
mod pointer;
mod pool;
mod ref_handle;
mod refbox;
mod stack;
mod types;

pub mod fakepool;

pub use self::box_handle::PoolBox;
pub use self::pool::Pool;
pub use self::ref_handle::PoolRef;

#[cfg(not(feature = "default_impl"))]
mod std_types;
#[cfg(not(feature = "default_impl"))]
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

#[cfg(feature = "default_impl")]
impl<A> PoolDefault for A
where
    A: Default,
{
    default unsafe fn default_uninit(target: &mut MaybeUninit<Self>) {
        target.as_mut_ptr().write(Default::default());
    }
}

#[cfg(feature = "default_impl")]
impl<A> PoolClone for A
where
    A: Clone + Default,
{
    default unsafe fn clone_uninit(&self, target: &mut MaybeUninit<Self>) {
        target.as_mut_ptr().write(self.clone());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct DropTest<'a> {
        counter: &'a AtomicUsize,
    }

    impl<'a> DropTest<'a> {
        fn new(counter: &'a AtomicUsize) -> Self {
            counter.fetch_add(1, Ordering::Relaxed);
            DropTest { counter }
        }
    }

    impl<'a> Drop for DropTest<'a> {
        fn drop(&mut self) {
            self.counter.fetch_sub(1, Ordering::Relaxed);
        }
    }

    fn fill_drop(pool_size: usize, alloc_size: usize) {
        let counter = AtomicUsize::new(0);
        let pool: Pool<DropTest<'_>> = Pool::new(pool_size);
        {
            let mut vec = Vec::new();
            for _ in 0..alloc_size {
                vec.push(PoolRef::new(&pool, DropTest::new(&counter)));
            }
            assert_eq!(alloc_size, counter.load(Ordering::SeqCst));
        }
        assert_eq!(0, counter.load(Ordering::SeqCst));
    }

    #[test]
    fn dropping_sized() {
        fill_drop(1024, 2048);
    }

    #[test]
    fn dropping_null() {
        fill_drop(0, 128);
    }

    #[test]
    fn allocate_and_deallocate_a_bit() {
        let pool: Pool<usize> = Pool::new(1024);
        assert_eq!(0, pool.get_pool_size());
        let mut refs: Vec<_> = Vec::new();
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

    #[test]
    fn null_pool_antics() {
        let pool: Pool<usize> = Pool::new(0);
        assert_eq!(0, pool.get_pool_size());
        let mut refs: Vec<_> = Vec::new();
        for _ in 0..10000 {
            refs.push(PoolRef::default(&pool));
        }
        assert_eq!(0, pool.get_pool_size());
        refs.clear();
        assert_eq!(0, pool.get_pool_size());
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
        assert_eq!(0, pool.get_pool_size());
    }

    #[test]
    fn unwrap_or_clone() {
        let pool: Pool<usize> = Pool::new(1024);
        let val = PoolRef::new(&pool, 1337);
        // This would crash if unwrap_or_clone tries to drop the consumed PoolRef.
        let unwrapped = PoolRef::unwrap_or_clone(val);
        assert_eq!(1337, unwrapped);
    }

    #[test]
    fn option_of_ref_size_equals_ref_size() {
        use std::mem::size_of;
        assert_eq!(
            size_of::<PoolRef<usize>>(),
            size_of::<Option<PoolRef<usize>>>()
        );
        assert_eq!(size_of::<Pool<usize>>(), size_of::<Option<Pool<usize>>>());
    }
}
