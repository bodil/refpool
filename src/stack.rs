// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[cfg(feature = "sync")]
use crossbeam_queue::ArrayQueue;

#[doc(hidden)]
pub trait Stack<A> {
    fn stack_new(max_size: usize) -> Self;
    fn stack_push(&mut self, value: A);
    fn stack_pop(&mut self) -> Option<A>;
    fn stack_len(&self) -> usize;
}

impl<A> Stack<A> for Vec<A> {
    fn stack_new(max_size: usize) -> Self {
        Self::with_capacity(max_size)
    }

    #[inline(always)]
    fn stack_push(&mut self, value: A) {
        self.push(value);
    }

    #[inline(always)]
    fn stack_pop(&mut self) -> Option<A> {
        self.pop()
    }

    #[inline(always)]
    fn stack_len(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "sync")]
impl<A> Stack<A> for ArrayQueue<A> {
    fn stack_new(max_size: usize) -> Self {
        Self::new(max_size)
    }

    #[inline(always)]
    fn stack_push(&mut self, value: A) {
        self.push(value).ok();
    }

    #[inline(always)]
    fn stack_pop(&mut self) -> Option<A> {
        self.pop().ok()
    }

    #[inline(always)]
    fn stack_len(&self) -> usize {
        self.len()
    }
}
