#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use refpool::{Pool, PoolRef};

#[derive(Arbitrary, Debug)]
enum Action {
    Allocate(String),
    AllocateDefault,
    Deallocate(usize),
    CloneNew(usize),
    Unwrap(usize),
}

use self::Action::*;

fn alloc<A>(allocs: &[A], index: usize) -> Option<usize> {
    if allocs.is_empty() {
        None
    } else {
        Some(index % allocs.len())
    }
}

fuzz_target!(|input: (u16, Vec<Action>)| {
    let pool = Pool::new(input.0 as usize);
    let actions = input.1;
    let mut allocs = Vec::new();
    for action in actions {
        match action {
            Allocate(data) => {
                allocs.push(PoolRef::new(&pool, data));
            }
            AllocateDefault => {
                allocs.push(PoolRef::<String>::default(&pool));
            }
            Deallocate(index) => {
                if let Some(index) = alloc(&allocs, index) {
                    allocs.remove(index);
                }
            }
            CloneNew(index) => {
                if let Some(index) = alloc(&allocs, index) {
                    allocs.push(PoolRef::cloned(&pool, &allocs[index]));
                }
            }
            Unwrap(index) => {
                if let Some(index) = alloc(&allocs, index) {
                    let string = PoolRef::unwrap_or_clone(allocs.remove(index));
                    allocs.push(PoolRef::new(&pool, string));
                }
            }
        }
    }
});
