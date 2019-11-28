// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(test)]

extern crate test;

use refpool::{Pool, PoolRef};
use std::rc::Rc;
use test::Bencher;

type TestType = usize;

#[bench]
fn alloc_131072_with_pool(b: &mut Bencher) {
    let pool = Pool::<TestType>::new(131_072);
    let mut vec: Vec<PoolRef<TestType>> = Vec::with_capacity(131_072);
    for _ in 0..131_072 {
        vec.push(PoolRef::default(&pool));
    }
    vec.clear();
    b.iter(|| {
        for _ in 0..131_072 {
            vec.push(PoolRef::default(&pool));
        }
        vec.clear();
    })
}

#[bench]
fn alloc_131072_without_pool(b: &mut Bencher) {
    let mut vec: Vec<Rc<TestType>> = Vec::with_capacity(131_072);
    b.iter(|| {
        for _ in 0..131_072 {
            vec.push(Default::default());
        }
        vec.clear();
    })
}

#[bench]
fn alloc_dealloc_131072_with_pool(b: &mut Bencher) {
    let pool = Pool::<TestType>::new(131_072);
    {
        let chunk: PoolRef<TestType> = PoolRef::default(&pool);
        test::black_box(chunk);
    }
    b.iter(|| {
        for _ in 0..131_072 {
            let chunk: PoolRef<TestType> = PoolRef::default(&pool);
            test::black_box(chunk);
        }
    })
}

#[bench]
fn alloc_dealloc_131072_without_pool(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..131_072 {
            let chunk: Rc<TestType> = Default::default();
            test::black_box(chunk);
        }
    })
}
