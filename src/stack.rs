// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub(crate) trait Stack<A> {
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
