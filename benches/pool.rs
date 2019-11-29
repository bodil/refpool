// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(test)]

extern crate test;

use refpool::{Pool, PoolRef, PoolSyncType, PoolUnsync};
use std::rc::Rc;
use test::Bencher;

#[cfg(feature = "sync")]
use std::sync::Arc;

#[cfg(feature = "sync")]
use refpool::PoolSync;

type TestType = usize;

fn alloc_131072_with_pool<S: PoolSyncType<TestType>>(b: &mut Bencher) {
    let pool = Pool::<TestType, S>::new(131_072);
    pool.fill();
    let mut vec: Vec<_> = Vec::with_capacity(131_072);
    b.iter(|| {
        for _ in 0..131_072 {
            vec.push(PoolRef::default(&pool));
        }
        vec.clear();
    })
}

fn alloc_131072_without_pool<Ref: Default>(b: &mut Bencher) {
    let mut vec: Vec<Ref> = Vec::with_capacity(131_072);
    b.iter(|| {
        for _ in 0..131_072 {
            vec.push(Default::default());
        }
        vec.clear();
    })
}

fn alloc_dealloc_131072_with_pool<S: PoolSyncType<TestType>>(b: &mut Bencher) {
    let pool = Pool::<TestType, S>::new(131_072);
    {
        let chunk = PoolRef::default(&pool);
        test::black_box(chunk);
    }
    b.iter(|| {
        for _ in 0..131_072 {
            let chunk = PoolRef::default(&pool);
            test::black_box(chunk);
        }
    })
}

fn alloc_dealloc_131072_without_pool<Ref: Default>(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..131_072 {
            let chunk: Ref = Default::default();
            test::black_box(chunk);
        }
    })
}

#[bench]
fn alloc_131072_with_unsync_pool(b: &mut Bencher) {
    alloc_131072_with_pool::<PoolUnsync>(b)
}

#[bench]
fn alloc_131072_without_unsync_pool(b: &mut Bencher) {
    alloc_131072_without_pool::<Rc<TestType>>(b)
}

#[bench]
fn alloc_dealloc_131072_with_unsync_pool(b: &mut Bencher) {
    alloc_dealloc_131072_with_pool::<PoolUnsync>(b)
}

#[bench]
fn alloc_dealloc_131072_without_unsync_pool(b: &mut Bencher) {
    alloc_dealloc_131072_without_pool::<Rc<TestType>>(b)
}

#[cfg(feature = "sync")]
#[bench]
fn alloc_131072_with_sync_pool(b: &mut Bencher) {
    alloc_131072_with_pool::<PoolSync>(b)
}

#[cfg(feature = "sync")]
#[bench]
fn alloc_131072_without_sync_pool(b: &mut Bencher) {
    alloc_131072_without_pool::<Arc<TestType>>(b)
}

#[cfg(feature = "sync")]
#[bench]
fn alloc_dealloc_131072_with_sync_pool(b: &mut Bencher) {
    alloc_dealloc_131072_with_pool::<PoolSync>(b)
}

#[cfg(feature = "sync")]
#[bench]
fn alloc_dealloc_131072_without_sync_pool(b: &mut Bencher) {
    alloc_dealloc_131072_without_pool::<Arc<TestType>>(b)
}
