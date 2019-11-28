# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic
Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### CHANGED

- The pool is now reference counted, which means your `PoolRef`s won't suddenly
  become dangerously invalid when the pool goes out of scope. This also means
  that when you clone a `Pool`, you get another reference to the same pool.

## [0.1.0] - 2019-11-26

Initial release.
