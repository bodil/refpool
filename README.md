# refpool

A reimplementation of Rust's `std::rc::Rc` which uses a pool of reusable memory
to speed up reallocation.

## Is It Fast?

It's about twice as fast as the system allocator on Linux systems, and six times
as fast on Windows systems, when the pool is non-empty. For certain data types,
gains can be even higher.

## Documentation

* [API docs](https://docs.rs/refpool)

## Licence

Copyright 2019 Bodil Stokke

This software is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

## Code of Conduct

Please note that this project is released with a [Contributor Code of
Conduct][coc]. By participating in this project you agree to abide by its
terms.

[coc]: https://github.com/bodil/refpool/blob/master/CODE_OF_CONDUCT.md
