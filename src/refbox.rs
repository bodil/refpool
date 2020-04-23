// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::mem::MaybeUninit;

use crate::{
    box_handle::PoolBox, counter::Counter, pointer::Pointer, pool::Pool, ref_handle::PoolRef,
    types::ElementPointer,
};

pub(crate) unsafe fn assume_init<A>(maybe_boxed: Box<MaybeUninit<A>>) -> Box<A> {
    Box::from_raw(Box::into_raw(maybe_boxed).cast())
    // TODO: Change this to `maybe_boxed.assume_init()` when the `new_uninit`
    // feature stabilises.
}

pub(crate) unsafe fn data_ptr<A>(this: &mut MaybeUninit<RefBox<A>>) -> &mut MaybeUninit<A> {
    (*this.as_mut_ptr())
        .value_as_mut_ptr()
        .cast::<MaybeUninit<A>>()
        .as_mut()
        .unwrap()
}

pub(crate) struct RefBox<A> {
    pub(crate) count: usize,
    pub(crate) pool: Pool<A>,
    pub(crate) value: A,
}

impl<A> RefBox<A> {
    pub(crate) fn into_ref(mut self: Box<Self>) -> PoolRef<A> {
        let ref_handle = self.new_ref();
        Box::leak(self);
        ref_handle
    }

    pub(crate) fn into_box(mut self: Box<Self>) -> PoolBox<A> {
        let box_handle = self.new_box();
        Box::leak(self);
        box_handle
    }

    pub(crate) fn new_ref(&mut self) -> PoolRef<A> {
        self.inc();
        PoolRef {
            handle: ElementPointer::wrap(self),
        }
    }

    pub(crate) fn new_box(&mut self) -> PoolBox<A> {
        self.inc();
        PoolBox {
            handle: ElementPointer::wrap(self),
        }
    }

    pub(crate) fn return_to_pool(self: Box<Self>) {
        if !self.pool.is_full() {
            let pool = self.pool.clone();
            let ptr = Box::into_raw(self);
            unsafe {
                ptr.drop_in_place();
                pool.push(ElementPointer::wrap(ptr));
            };
        }
    }

    pub(crate) fn value_as_ref(&self) -> &A {
        &self.value
    }

    pub(crate) fn value_as_mut(&mut self) -> &mut A {
        &mut self.value
    }

    pub(crate) unsafe fn value_as_mut_ptr(&mut self) -> *mut A {
        &mut self.value
    }

    #[inline(always)]
    pub(crate) fn inc(&mut self) {
        self.count.inc()
    }

    #[inline(always)]
    pub(crate) fn dec(&mut self) -> usize {
        self.count.dec()
    }

    #[inline(always)]
    pub(crate) fn is_shared(&self) -> bool {
        self.count.count() > 1
    }
}
