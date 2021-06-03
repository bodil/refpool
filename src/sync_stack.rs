use std::{
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering::*},
};

use static_assertions::assert_impl_all;

pub const STACK_SIZE: usize = 1024;

/// A fixed capacity array based Treiber stack.
pub struct SyncStack<A> {
    size: AtomicUsize,
    data: MaybeUninit<[A; STACK_SIZE]>,
}

assert_impl_all!(SyncStack<u8>: Send, Sync);

impl<A> SyncStack<A> {
    pub const fn new() -> Self {
        Self {
            size: AtomicUsize::new(0),
            data: MaybeUninit::uninit(),
        }
    }

    fn data_ptr(&self) -> *mut A {
        self.data.as_ptr().cast::<A>() as *mut A
    }

    pub fn pop(&self) -> Option<A> {
        let mut size = self.size.load(Acquire);
        loop {
            if size == 0 {
                return None;
            }
            let next = size - 1;
            match self.size.compare_exchange_weak(size, next, AcqRel, Acquire) {
                Ok(old_size) => {
                    return Some(unsafe { self.data_ptr().add(old_size - 1).read() });
                }
                Err(old_size) => {
                    size = old_size;
                }
            }
            std::hint::spin_loop();
        }
    }

    pub fn push(&self, value: A) -> Result<(), A> {
        let mut size = self.size.load(Relaxed);
        loop {
            if size == STACK_SIZE {
                return Err(value);
            }
            let next = size + 1;
            match self
                .size
                .compare_exchange_weak(size, next, Release, Relaxed)
            {
                Ok(old_size) => {
                    unsafe { self.data_ptr().add(old_size).write(value) };
                    return Ok(());
                }
                Err(old_size) => {
                    size = old_size;
                }
            }
            std::hint::spin_loop();
        }
    }
}

impl<A> Drop for SyncStack<A> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<A>() {
            let size = *self.size.get_mut();
            let data = self.data_ptr();
            unsafe {
                std::ptr::drop_in_place(std::slice::from_raw_parts_mut(data, size));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push_and_pop_a_bit_on_one_thread() {
        let stack: SyncStack<usize> = SyncStack::new();
        for i in 0..STACK_SIZE {
            assert_eq!(Ok(()), stack.push(i));
        }
        assert_eq!(Err(STACK_SIZE), stack.push(STACK_SIZE));
        for i in 0..STACK_SIZE {
            assert_eq!(Some((STACK_SIZE - 1) - i), stack.pop());
        }
        assert_eq!(None, stack.pop());
    }
}
