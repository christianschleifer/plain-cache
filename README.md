# plain-cache

[![Crates.io Version](https://img.shields.io/crates/v/plain-cache)](https://crates.io/crates/plain-cache)
[![docs.rs](https://img.shields.io/docsrs/plain-cache)](https://docs.rs/plain-cache/latest/plain_cache/)
[![Continuous Integration](https://github.com/christianschleifer/plain-cache/actions/workflows/ci.yml/badge.svg)](https://github.com/christianschleifer/plain-cache/actions/workflows/ci.yml)

## Overview

`plain-cache` is a high-performance, thread-safe cache implementation that makes no use of unsafe
Rust code. It implements the S3-FIFO eviction
algorithm [[see](https://dl.acm.org/doi/pdf/10.1145/3600006.3613147)]. `plain-cache` allocates
it's capacity at cache instantiation time.

## Use if you need

* High performance
* Thread safety
* No usage of unsafe code
* No background threads
* Cache metrics
* Small dependency tree
* Easy-to-reason cache eviction (S3-FIFO)
* Ability to provide custom hasher

## Do not use if you need

* Zero-sized types
* Lifecycle hooks
* Item weighing
* Time-based eviction
* Explicit cache deletions
