// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ptr::NonNull;
#[cfg(feature = "sync")]
use std::sync::atomic::AtomicPtr;

pub trait Pointer<A> {
    fn wrap(ptr: *mut A) -> Self;
    fn get_ptr(&self) -> *mut A;
    fn cast<B>(self) -> *mut B;
}

impl<A> Pointer<A> for NonNull<A> {
    #[inline(always)]
    fn wrap(ptr: *mut A) -> Self {
        debug_assert_eq!(false, ptr.is_null());
        unsafe { NonNull::new_unchecked(ptr) }
    }

    #[inline(always)]
    fn get_ptr(&self) -> *mut A {
        self.as_ptr()
    }

    #[inline(always)]
    fn cast<B>(self) -> *mut B {
        NonNull::cast(self).as_ptr()
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
