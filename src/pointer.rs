// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ptr::NonNull;

pub trait Pointer<A> {
    fn wrap(ptr: *mut A) -> Self;
    fn get_ptr(&self) -> *mut A;
    fn cast<B>(self) -> *mut B;
}

pub trait NullablePointer<A>: Pointer<A> {
    fn null() -> Self;
    fn is_null(&self) -> bool;
}

impl<A> Pointer<A> for *mut A {
    #[inline(always)]
    fn wrap(ptr: Self) -> Self {
        ptr
    }

    #[inline(always)]
    fn get_ptr(&self) -> Self {
        *self
    }

    #[inline(always)]
    fn cast<B>(self) -> *mut B {
        self.cast()
    }
}

impl<A> NullablePointer<A> for *mut A {
    #[inline(always)]
    fn null() -> Self {
        std::ptr::null_mut()
    }

    #[inline(always)]
    fn is_null(&self) -> bool {
        (*self).is_null()
    }
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
        self.get_ptr().cast()
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
        self.0.get_ptr().cast()
    }
}

#[cfg(feature = "sync")]
impl<A> Pointer<A> for std::sync::atomic::AtomicPtr<A> {
    #[inline(always)]
    fn wrap(ptr: *mut A) -> Self {
        Self::new(ptr)
    }

    #[inline(always)]
    fn get_ptr(&self) -> *mut A {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[inline(always)]
    fn cast<B>(self) -> *mut B {
        self.get_ptr().cast()
    }
}

#[cfg(feature = "sync")]
impl<A> NullablePointer<A> for std::sync::atomic::AtomicPtr<A> {
    #[inline(always)]
    fn null() -> Self {
        Self::new(std::ptr::null_mut())
    }

    #[inline(always)]
    fn is_null(&self) -> bool {
        self.get_ptr().is_null()
    }
}
