// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::{BuildHasher, Hash};
use std::mem::MaybeUninit;
use std::path::PathBuf;

use crate::{PoolClone, PoolDefault};

/// A marker trait for types which should be fully initialised.
///
/// Implementing this trait for a type provides a [`PoolDefault`][PoolDefault]
/// implementation which writes the result of
/// [`Default::default()`][Default::default] to its memory location.
///
/// For types which implement [`Clone`][Clone], this will also provide an
/// implementation of [`PoolClone`][PoolClone] that writes the result of
/// [`Clone::clone()`][Clone::clone] to its memory location.
///
/// This makes sense for most types, and these implementations are always
/// correct, but you may wish to provide your own implementations for types
/// which don't have to fully initialise their allocated memory regions, which
/// is why we don't implement [`PoolDefault`][PoolDefault] for anything that
/// implements [`Default`][Default] and [`PoolClone`][PoolClone] for anything
/// that implements [`Clone`][Clone], given the absence of [trait
/// specialisation](https://github.com/rust-lang/rust/issues/31844).
///
/// [PoolDefault]: trait.PoolDefault.html
/// [PoolClone]: trait.PoolClone.html
/// [Default]: https://doc.rust-lang.org/std/default/trait.Default.html
/// [Default::default]: https://doc.rust-lang.org/std/default/trait.Default.html#tymethod.default
/// [Clone]: https://doc.rust-lang.org/std/clone/trait.Clone.html
/// [Clone::clone]: https://doc.rust-lang.org/std/clone/trait.Clone.html#tymethod.clone
pub trait PoolDefaultImpl: Default {}

impl<A> PoolDefault for A
where
    A: PoolDefaultImpl,
{
    unsafe fn default_uninit(target: &mut MaybeUninit<Self>) {
        target.as_mut_ptr().write(Default::default());
    }
}

impl<A> PoolClone for A
where
    A: PoolDefaultImpl + Clone,
{
    unsafe fn clone_uninit(&self, target: &mut MaybeUninit<Self>) {
        target.as_mut_ptr().write(self.clone());
    }
}

impl PoolDefaultImpl for bool {}

impl PoolDefaultImpl for u8 {}
impl PoolDefaultImpl for u16 {}
impl PoolDefaultImpl for u32 {}
impl PoolDefaultImpl for u64 {}
impl PoolDefaultImpl for u128 {}
impl PoolDefaultImpl for usize {}

impl PoolDefaultImpl for i8 {}
impl PoolDefaultImpl for i16 {}
impl PoolDefaultImpl for i32 {}
impl PoolDefaultImpl for i64 {}
impl PoolDefaultImpl for i128 {}
impl PoolDefaultImpl for isize {}

impl<A> PoolDefaultImpl for Option<A> {}

impl PoolDefaultImpl for String {}
impl PoolDefaultImpl for PathBuf {}

impl<A> PoolDefaultImpl for Vec<A> {}
impl<A> PoolDefaultImpl for VecDeque<A> {}
impl<A: Hash + Eq, S: BuildHasher + Default> PoolDefaultImpl for HashSet<A, S> {}
impl<A: Hash + Eq, B, S: BuildHasher + Default> PoolDefaultImpl for HashMap<A, B, S> {}
impl<A: Ord, B> PoolDefaultImpl for BTreeMap<A, B> {}
impl<A: Ord> PoolDefaultImpl for BTreeSet<A> {}
impl<A: Ord> PoolDefaultImpl for BinaryHeap<A> {}
impl<A> PoolDefaultImpl for LinkedList<A> {}

impl<A, B> PoolDefaultImpl for (A, B)
where
    A: PoolDefaultImpl,
    B: PoolDefaultImpl,
{
}

impl<A, B, C> PoolDefaultImpl for (A, B, C)
where
    A: PoolDefaultImpl,
    B: PoolDefaultImpl,
    C: PoolDefaultImpl,
{
}

impl<A, B, C, D> PoolDefaultImpl for (A, B, C, D)
where
    A: PoolDefaultImpl,
    B: PoolDefaultImpl,
    C: PoolDefaultImpl,
    D: PoolDefaultImpl,
{
}

impl<A, B, C, D, E> PoolDefaultImpl for (A, B, C, D, E)
where
    A: PoolDefaultImpl,
    B: PoolDefaultImpl,
    C: PoolDefaultImpl,
    D: PoolDefaultImpl,
    E: PoolDefaultImpl,
{
}

impl<A, B, C, D, E, F> PoolDefaultImpl for (A, B, C, D, E, F)
where
    A: PoolDefaultImpl,
    B: PoolDefaultImpl,
    C: PoolDefaultImpl,
    D: PoolDefaultImpl,
    E: PoolDefaultImpl,
    F: PoolDefaultImpl,
{
}

impl<A, B, C, D, E, F, G> PoolDefaultImpl for (A, B, C, D, E, F, G)
where
    A: PoolDefaultImpl,
    B: PoolDefaultImpl,
    C: PoolDefaultImpl,
    D: PoolDefaultImpl,
    E: PoolDefaultImpl,
    F: PoolDefaultImpl,
    G: PoolDefaultImpl,
{
}

impl<A, B, C, D, E, F, G, H> PoolDefaultImpl for (A, B, C, D, E, F, G, H)
where
    A: PoolDefaultImpl,
    B: PoolDefaultImpl,
    C: PoolDefaultImpl,
    D: PoolDefaultImpl,
    E: PoolDefaultImpl,
    F: PoolDefaultImpl,
    G: PoolDefaultImpl,
    H: PoolDefaultImpl,
{
}
