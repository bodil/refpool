// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[cfg(feature = "sync")]
use std::sync::atomic::AtomicPtr;

pub trait Pointer<A> {
    fn wrap(ptr: *mut A) -> Self;
    fn get_ptr(&self) -> *mut A;
    fn cast<B>(self) -> *mut B;
}

impl<A> Pointer<A> for *mut A {
    #[inline(always)]
    fn wrap(ptr: Self) -> Self {
        ptr
    }

    #[inline(always)]
    fn get_ptr(&self) -> *mut A {
        *self
    }

    #[inline(always)]
    fn cast<B>(self) -> *mut B {
        self as _
    }
}

#[cfg(feature = "sync")]
impl<A> Pointer<A> for AtomicPtr<A> {
    #[inline(always)]
    fn wrap(ptr: *mut A) -> Self {
        AtomicPtr::new(ptr)
    }

    #[inline(always)]
    fn get_ptr(&self) -> *mut A {
        self.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[inline(always)]
    fn cast<B>(self) -> *mut B {
        self.get_ptr().cast()
    }
}
