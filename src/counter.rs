// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[cfg(feature = "sync")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[doc(hidden)]
pub trait Counter: Default {
    fn inc(&mut self);
    fn dec(&mut self) -> usize;
    fn count(&self) -> usize;
}

impl Counter for usize {
    #[inline(always)]
    fn inc(&mut self) {
        *self += 1;
    }

    #[inline(always)]
    fn dec(&mut self) -> usize {
        let prev = *self;
        *self -= 1;
        prev
    }

    #[inline(always)]
    fn count(&self) -> usize {
        *self
    }
}

#[cfg(feature = "sync")]
impl Counter for AtomicUsize {
    #[inline(always)]
    fn inc(&mut self) {
        self.fetch_add(1, Ordering::Relaxed);
    }

    #[inline(always)]
    fn dec(&mut self) -> usize {
        self.fetch_sub(1, Ordering::Release)
    }

    #[inline(always)]
    fn count(&self) -> usize {
        self.load(Ordering::SeqCst)
    }
}
