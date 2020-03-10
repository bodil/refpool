// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub(crate) trait Counter: Default {
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
