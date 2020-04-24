# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project
adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ADDED

-   There's now a feature flag `default_impl` which removes the `PoolDefaultImpl` trait and instead
    uses specialisation (specifically the `min_specialization` language feature) to provide default
    implementations for `PoolClone` and `PoolDefault` for any type implementing `Clone` and
    `Default`. As this needs an unstable language feature to be enabled, it will only work on
    nightly rustc.
-   `PoolBox` and `PoolRef` now have `into_raw`, `into_raw_non_null` and `from_raw` functions, which
    work similarly to their `Box` and `Rc` counterparts. To accommodate this, the memory layout of
    the internal `RefBox` structure has changed, so that the pointer stored in a `PoolBox` or
    `PoolRef` is now guaranteed to point at the boxed value.

## [0.3.1] - 2020-04-23

### ADDED

-   There is now a `PoolBox` which works like `Box` but uses the pool to allocate.

## [0.3.0] - 2020-03-10

### REMOVED

-   The `PoolSync` mode has been removed entirely, along with the option to choose which mode to
    use, as `PoolUnsync` is now the only one remaining. `PoolSync` pools were too slow to be
    worthwhile, and I didn't trust the correctness of the threadsafe `PoolRef` implementation.

## [0.2.3] - 2020-01-07

### ADDED

-   `Pool` now implements `Debug`.

### FIXED

-   `Option<Pool>` and `Pool` are now once again the same size. Zero sized pools still don't cause
    any allocations.

## [0.2.2] - 2019-12-16

### ADDED

-   You can now `Pool::cast()` a pool handle into a pool handle for a different type, allowing you
    to construct values of multiple types from the same pool, provided they are of the exact same
    size and alignment.
-   `Pool`s of size 0 are now represented by null pointers, meaning they allocate nothing. It also
    means `Option<Pool>` is no longer identical in size to `Pool`, but `PoolRef` still retains that
    property. A `Pool` of size 0 is also conceptually identical to the `None` value of an
    `Option<Pool>`, except you can use it to construct values without having to unwrap it first, so
    there's no good reason you should ever need `Option<Pool>`.

## [0.2.1] - 2019-12-12

### FIXED

-   `Pool` and `PoolRef` now use `NonNull` instead of raw pointers, so that they can be wrapped in
    `Option` without growing in size.
-   Fixed a race condition where the last `PoolRef` referencing a pool might try to drop it before
    returning its allocation to it, causing a memory fault.

## [0.2.0] - 2019-11-29

### CHANGED

-   The pool is now reference counted, which means your `PoolRef`s won't suddenly become dangerously
    invalid when the pool goes out of scope. This also means that you can now clone a `Pool` and get
    another reference to the same pool.

### ADDED

-   There are now both `Sync` and `!Sync` versions of the pool. The latter, in
    `refpool::unsync::Pool`, is as performant as previously, while the thread safe version in
    `refpool::sync::Pool` is roughly 5-10x slower, but still manages to be about 25% faster than the
    Windows system allocator. You should prefer not to use it on platforms with faster system
    allocators, such as Linux. To enable the thread safe version, use the `sync` feature flag.
-   A method `Pool::fill()` has been added, which preallocates memory chunks to the capacity of the
    pool.

## [0.1.0] - 2019-11-26

Initial release.
