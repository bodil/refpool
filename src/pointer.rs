// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ptr::NonNull;

pub trait Pointer<A> {
    fn wrap(ptr: *mut A) -> Self;
    fn get_ptr(&self) -> *mut A;
    fn cast<B>(self) -> *mut B;
    fn get_ptr_checked(&self) -> Option<*mut A>;
    fn null() -> Self;
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
        self.as_ptr().cast()
    }

    #[inline(always)]
    fn get_ptr_checked(&self) -> Option<*mut A> {
        if *self == NonNull::dangling() {
            None
        } else {
            Some(self.as_ptr())
        }
    }

    #[inline(always)]
    fn null() -> Self {
        NonNull::dangling()
    }
}

#[cfg(feature = "sync")]
pub struct NonNullAtomicPtr<A>(NonNull<A>);

#[cfg(feature = "sync")]
unsafe impl<A> Send for NonNullAtomicPtr<A> {}
#[cfg(feature = "sync")]
unsafe impl<A> Sync for NonNullAtomicPtr<A> {}

#[cfg(feature = "sync")]
impl<A> Pointer<A> for NonNullAtomicPtr<A> {
    #[inline(always)]
    fn wrap(ptr: *mut A) -> Self {
        debug_assert_eq!(false, ptr.is_null());
        Self(unsafe { NonNull::new_unchecked(ptr) })
    }

    #[inline(always)]
    fn get_ptr(&self) -> *mut A {
        use std::sync::atomic::{AtomicPtr, Ordering};
        let atomic = unsafe { &*(&self.0 as *const NonNull<A> as *const AtomicPtr<A>) };
        atomic.load(Ordering::Relaxed)
    }

    #[inline(always)]
    fn cast<B>(self) -> *mut B {
        self.get_ptr().cast()
    }

    #[inline(always)]
    fn get_ptr_checked(&self) -> Option<*mut A> {
        use std::sync::atomic::{AtomicPtr, Ordering};
        let atomic = unsafe { &*(&self.0 as *const NonNull<A> as *const AtomicPtr<A>) };
        let ptr = atomic.load(Ordering::Relaxed);
        if ptr == NonNull::dangling().as_ptr() {
            None
        } else {
            Some(ptr)
        }
    }

    #[inline(always)]
    fn null() -> Self {
        Self(NonNull::dangling())
    }
}
