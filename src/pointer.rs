// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ptr::NonNull;

pub(crate) trait Pointer<A>: std::fmt::Pointer {
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
