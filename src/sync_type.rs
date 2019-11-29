// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::counter::Counter;
use crate::handle::RefBox;
use crate::pointer::Pointer;
use crate::pool::PoolInner;
use crate::stack::Stack;

#[cfg(feature = "sync")]
use crossbeam_queue::ArrayQueue;
#[cfg(feature = "sync")]
use std::sync::atomic::{AtomicPtr, AtomicUsize};

#[doc(hidden)]
pub trait PoolSyncType<A>: Sized {
    type Counter: Counter;
    type Stack: Stack<Self::ElementPointer>;
    type ElementPointer: Pointer<RefBox<A, Self>>;
    type PoolPointer: Pointer<PoolInner<A, Self>>;
}

/// Marker type for thread safe pools.
///
/// This is only available if you've enabled the `sync` feature flag.
#[cfg(feature = "sync")]
pub struct PoolSync;

/// Marker type for non-thread safe pools.
pub struct PoolUnsync;

#[cfg(feature = "sync")]
impl<A> PoolSyncType<A> for PoolSync {
    type Counter = AtomicUsize;
    type Stack = ArrayQueue<Self::ElementPointer>;
    type ElementPointer = AtomicPtr<RefBox<A, Self>>;
    type PoolPointer = AtomicPtr<PoolInner<A, Self>>;
}

impl<A> PoolSyncType<A> for PoolUnsync {
    type Counter = usize;
    type Stack = Vec<Self::ElementPointer>;
    type ElementPointer = *mut RefBox<A, Self>;
    type PoolPointer = *mut PoolInner<A, Self>;
}