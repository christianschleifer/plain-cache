//! A highly performant, thread-safe cache implementation.
//!
//! This crate provides a cache that implements the S3-FIFO eviction algorithm as specified in
//! [FIFO Queues are All You Need for Cache Eviction](https://dl.acm.org/doi/pdf/10.1145/3600006.3613147).
//!
//! # Features
//!
//! - Thread-safe by default - no need for explicit synchronization
//! - S3-FIFO eviction algorithm for optimal cache performance
//! - Sharded design to reduce contention during concurrent access
//! - No unsafe code
//!
//! # Safety
//!
//! This crate is designed to be safe and easy to use:
//!
//! - No unsafe code is used
//! - Thread-safe by default when wrapped in `Arc`
//! - All operations are atomic
//!
//! # Examples
//!
//! Basic usage with string keys and values:
//!
//! ```rust
//! use plain_cache::Cache;
//!
//! // Create a new cache with a capacity of 1000 items
//! let cache = Cache::with_capacity(1000);
//!
//! // Insert and retrieve a value
//! cache.insert("key1", "value1");
//! assert_eq!(cache.get("key1"), Some("value1"));
//! ```
//!
//! Updating existing values:
//!
//! ```rust
//! use plain_cache::Cache;
//!
//! let cache = Cache::with_capacity(100);
//!
//! // Insert initial value
//! cache.insert("key1", "value1");
//!
//! // Update the value and get the old one
//! let old_value = cache.insert("key1", "new_value");
//! assert_eq!(old_value, Some("value1"));
//! assert_eq!(cache.get("key1"), Some("new_value"));
//! ```
//!
//! Thread-safe usage across multiple threads:
//!
//! ```rust
//! use plain_cache::Cache;
//! use std::sync::Arc;
//! use std::thread;
//!
//! let cache = Arc::new(Cache::with_capacity(100));
//! cache.insert("key1", "value1");
//!
//! // Spawn a thread that inserts a value
//! let cache_in_arc = Arc::clone(&cache);
//! let handle = thread::spawn(move || {
//!     cache_in_arc.insert("key2", "value2");
//! });
//!
//! handle.join().unwrap();
//!
//! assert_eq!(cache.get("key1"), Some("value1"));
//! assert_eq!(cache.get("key2"), Some("value2"));
//! ```

#![forbid(unsafe_code)]
pub mod cache;

pub use cache::Cache;
pub use cache::stats::Stats;
