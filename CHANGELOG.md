# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project
adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### FIXED

-   The unsync `Pool` and `PoolRef` now use `NonNull` instead of raw pointers, so that they can be
    wrapped in `Option` without growing in size.
-   Fixed a race condition where the last `PoolRef` referencing a pool might try to drop it before
    returning its allocation to it.

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
