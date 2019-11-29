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
//! # Differences from [`Rc`][Rc] and [`Arc`][Arc]
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
//!
//! # Thread Safety
//!
//! [`Pool`][Pool] defaults to being thread local by default, ie. it does not
//! implement [`Sync`][Sync] and it will fail in appalling ways if you still
//! somehow manage to access it from two different threads. There's a marker
//! type [`PoolSync`][PoolSync], available behind the `sync` feature flag, which
//! you can pass as a second type argument to [`Pool`][Pool] and
//! [`PoolRef`][PoolRef], for a thread safe version. However, this will be much
//! less performant, on some platforms even failing to outperform the system
//! allocator by a significant margin. It's not recommended that you use pools
//! for thread safe code unless your benchmarks actually show that you gain from
//! doing so.
//!
//! There are also type aliases for the thread safe version available in the
//! `refpool::sync` namespace, if you have the `sync` feature flag enabled:
//! `refpool::sync::Pool<A>` and `refpool::sync::PoolRef<A>`.
//!
//! # Performance
//!
//! You can expect [`Pool`][Pool] to always outperform the system allocator,
//! though the performance gains will vary between platforms. Preliminary
//! benchmarks show it's approximately twice as fast on Linux, and 5-6 times as
//! fast on Windows. The [`PoolSync`][PoolSync] version is marginally faster on
//! Windows, but about 3 times slower on Linux, hence the recommendation above
//! that you don't use it without benchmarks to back your use case.
//!
//! You can expect bigger performance gains from data types with beneficial
//! [`PoolDefault`][PoolDefault] and [`PoolClone`][PoolClone] implementations,
//! "beneficial" in this case meaning cases where you can leave most of the
//! allocated memory unitialised. [`sized_chunks::Chunk`][Chunk], which
//! allocates 528 bytes on 64-bit platforms but only needs to initialise 16
//! bytes for [`PoolDefault`][PoolDefault], would be a good example of this.

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
//! [PoolSync]: struct.PoolSync.html
//! [Default]: https://doc.rust-lang.org/std/default/trait.Default.html
//! [Clone]: https://doc.rust-lang.org/std/clone/trait.Clone.html
//! [Arc]: https://doc.rust-lang.org/std/sync/struct.Arc.html
//! [Rc]: https://doc.rust-lang.org/std/rc/struct.Rc.html
//! [Rc::new]: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.new
//! [Weak]: https://doc.rust-lang.org/std/rc/struct.Weak.html
//! [Sized]: https://doc.rust-lang.org/std/marker/trait.Sized.html
//! [Sync]: https://doc.rust-lang.org/std/marker/trait.Sync.html
//! [Chunk]: https://docs.rs/sized-chunks/*/sized_chunks/sized_chunk/struct.Chunk.html

use std::mem::MaybeUninit;

mod counter;
mod handle;
mod pointer;
mod pool;
mod stack;
mod std_types;
mod sync_type;

pub use self::handle::PoolRef;
pub use self::pool::Pool;
pub use self::std_types::PoolDefaultImpl;
pub use self::sync_type::{PoolSyncType, PoolUnsync};

#[cfg(feature = "sync")]
pub use self::sync_type::PoolSync;

/// Type aliases for thread safe pools.
#[cfg(feature = "sync")]
pub mod sync {
    use crate::sync_type::PoolSync;

    /// A thread safe pool type.
    pub type Pool<A> = crate::Pool<A, PoolSync>;
    /// A thread safe reference counter type.
    ///
    /// This is the pooled equivalent to [`std::sync::Arc`][Arc].
    ///
    /// [Arc]: https://doc.rust-lang.org/std/sync/struct.Arc.html
    pub type PoolRef<A> = crate::PoolRef<A, PoolSync>;
}

/// Type aliases for non-thread safe pools.
pub mod unsync {
    use crate::sync_type::PoolUnsync;

    /// A non-thread safe pool type.
    pub type Pool<A> = crate::Pool<A, PoolUnsync>;
    /// A non-thread safe reference counter type.
    ///
    /// This is the pooled equivalent to [`std::rc::Rc`][Rc].
    ///
    /// [Rc]: https://doc.rust-lang.org/std/rc/struct.Rc.html
    pub type PoolRef<A> = crate::PoolRef<A, PoolUnsync>;
}

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

#[cfg(test)]
mod test {
    use super::*;

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
}
