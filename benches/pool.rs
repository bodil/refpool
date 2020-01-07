// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::rc::Rc;
use std::sync::Arc;

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};

use refpool::{Pool, PoolDefault, PoolRef, PoolSync, PoolSyncType, PoolUnsync};

const SIZES: &[usize] = &[1024, 2048, 4096, 8192, 16384, 32768, 65536, 131_072];

pub fn alloc<A: PoolDefault, S: PoolSyncType<A>, P: Default>(name: &str, c: &mut Criterion) {
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
                || (Pool::<A, S>::new(size), Vec::with_capacity(size)),
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
                    let pool = Pool::<A, S>::new(size);
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

pub fn realloc<A: PoolDefault, S: PoolSyncType<A>, P: Default>(name: &str, c: &mut Criterion) {
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
                || Pool::<A, S>::new(size),
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

fn alloc_unsync_usize(c: &mut Criterion) {
    alloc::<usize, PoolUnsync, Rc<usize>>("alloc/unsync/usize", c)
}

fn alloc_sync_usize(c: &mut Criterion) {
    alloc::<usize, PoolSync, Arc<usize>>("alloc/sync/usize", c)
}

fn realloc_unsync_usize(c: &mut Criterion) {
    realloc::<usize, PoolUnsync, Rc<usize>>("realloc/unsync/usize", c)
}

fn realloc_sync_usize(c: &mut Criterion) {
    realloc::<usize, PoolSync, Arc<usize>>("realloc/sync/usize", c)
}

criterion_group!(
    refpool,
    alloc_unsync_usize,
    alloc_sync_usize,
    realloc_unsync_usize,
    realloc_sync_usize
);
criterion_main!(refpool);
