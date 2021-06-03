// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::rc::Rc;

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};

use refpool::{Pool, PoolDefault, PoolDefaultImpl, PoolRef};

const SIZES: &[usize] = &[1024, 2048, 4096, 8192, 16384, 32768, 65536, 131_072];

struct BigLumpOfUsize([usize; 1024]);

impl Default for BigLumpOfUsize {
    fn default() -> Self {
        Self([0; 1024])
    }
}

impl PoolDefaultImpl for BigLumpOfUsize {}

pub fn alloc<A: PoolDefault, P: Default>(name: &str, c: &mut Criterion) {
    let mut group = c.benchmark_group(name);
    for size in SIZES {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("sysalloc", size), size, |b, &size| {
            b.iter_batched_ref(
                || Vec::with_capacity(size),
                |vec| {
                    for _ in 0..size {
                        vec.push(P::default());
                    }
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("empty pool", size), size, |b, &size| {
            b.iter_batched_ref(
                || (Pool::<A>::new(size), Vec::with_capacity(size)),
                |&mut (ref pool, ref mut vec)| {
                    for _ in 0..size {
                        vec.push(PoolRef::default(pool));
                    }
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("full pool", size), size, |b, &size| {
            b.iter_batched_ref(
                || {
                    let pool = Pool::<A>::new(size);
                    pool.fill();
                    (pool, Vec::with_capacity(size))
                },
                |&mut (ref pool, ref mut vec)| {
                    for _ in 0..size {
                        vec.push(PoolRef::default(pool));
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

pub fn realloc<A: PoolDefault, P: Default>(name: &str, c: &mut Criterion) {
    let mut group = c.benchmark_group(name);
    for size in SIZES {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("sysalloc", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    black_box(P::default());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("pool", size), size, |b, &size| {
            b.iter_batched_ref(
                || Pool::<A>::new(size),
                |pool| {
                    for _ in 0..size {
                        black_box(PoolRef::default(pool));
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn alloc_usize(c: &mut Criterion) {
    alloc::<usize, Rc<usize>>("alloc/usize", c)
}

fn realloc_usize(c: &mut Criterion) {
    realloc::<usize, Rc<usize>>("realloc/usize", c)
}

fn alloc_1024x_usize(c: &mut Criterion) {
    alloc::<BigLumpOfUsize, Rc<BigLumpOfUsize>>("alloc/1024xusize", c)
}

fn realloc_1024x_usize(c: &mut Criterion) {
    realloc::<BigLumpOfUsize, Rc<BigLumpOfUsize>>("realloc/1024xsize", c)
}

criterion_group!(
    refpool,
    alloc_usize,
    realloc_usize,
    alloc_1024x_usize,
    realloc_1024x_usize
);
criterion_main!(refpool);
