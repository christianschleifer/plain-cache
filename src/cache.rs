use crate::Stats;
use parking_lot::{Mutex, RwLock};
use shard::Shard;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};
use std::num::NonZero;
use std::time::Instant;
use std::{cmp, thread};

mod entry;
mod fixed_size_hash_table;
mod ring_buffer;
mod shard;
pub(crate) mod stats;

pub(crate) type RandomState = ahash::RandomState;

/// Highly performant, thread-safe cache with a focus on simplicity.
///
/// It implements the S3-FIFO eviction algorithm as specified in
/// [FIFO Queues are All You Need for Cache Eviction](https://dl.acm.org/doi/pdf/10.1145/3600006.3613147).
/// The cache is divided into multiple shards to reduce contention during concurrent access. This
/// crate does not use any unsafe code.
///
/// Wrap the cache in a [`std::sync::Arc`] to share it between threads. Both reads and writes only
/// require shared references to the cache.
#[derive(Debug)]
pub struct Cache<K, V, S = RandomState> {
    hash_builder: S,
    shards: Vec<RwLock<Shard<K, V, S>>>,
    metrics_last_accessed: Mutex<Instant>,
}

impl<K, V> Cache<K, V, RandomState>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Creates a new cache with at least the specified capacity.
    ///
    /// The actual capacity may be slightly higher due to sharding and rounding.
    pub fn with_capacity(capacity: usize) -> Cache<K, V, RandomState> {
        Cache::with_capacity_and_hasher(capacity, Default::default())
    }
}

impl<K, V, S> Cache<K, V, S>
where
    K: Clone + Eq + Hash,
    V: Clone,
    S: BuildHasher,
{
    /// Inserts a key-value pair into the cache.
    ///
    /// If the cache did not have this key present, [`None`] is returned.
    ///
    /// If the cache did have this key present, the value is updated, and the old value is returned.
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let hash = self.hash_builder.hash_one(&key);
        let shard_lock = self.get_shard(hash)?;

        let mut shard = shard_lock.write();
        shard.insert(key, value)
    }

    /// Returns the value corresponding to the key.
    ///
    /// This method clones the value when returning the item. Consider wrapping your values in
    /// [`std::sync::Arc`] if cloning is too expensive for you use-case.
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        let hash = self.hash_builder.hash_one(key);
        let shard_lock = self.get_shard(hash)?;

        let shard = shard_lock.read();
        shard.get(key)
    }

    fn get_shard(&self, hash: u64) -> Option<&RwLock<Shard<K, V, S>>> {
        let shard_idx = hash as usize % (cmp::max(self.shards.len(), 2) - 1);
        self.shards.get(shard_idx)
    }
}

impl<K, V, S> Cache<K, V, S>
where
    K: Clone + Eq + Hash,
    V: Clone,
    S: Clone + BuildHasher,
{
    /// Creates a new cache with the at least the specified capacity, using `hasher` to hash the
    /// keys.
    ///
    /// The actual capacity may be slightly higher due to sharding and rounding.
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Cache<K, V, S> {
        let available_parallelism = thread::available_parallelism()
            .map(NonZero::get)
            .unwrap_or(1);

        let number_of_shards = cmp::min(available_parallelism * 4, capacity);

        let mut shards = Vec::with_capacity(number_of_shards);

        let metrics_last_accessed = Mutex::new(Instant::now());

        if number_of_shards == 0 {
            return Self {
                hash_builder,
                shards,
                metrics_last_accessed,
            };
        }

        let capacity_per_shard = capacity.div_ceil(number_of_shards);

        for _ in 0..number_of_shards {
            let shard = Shard::with_capacity_and_hasher(capacity_per_shard, hash_builder.clone());
            shards.push(RwLock::new(shard))
        }

        Self {
            hash_builder,
            shards,
            metrics_last_accessed,
        }
    }
}

impl<K, V, S> Cache<K, V, S> {
    pub fn stats(&self) -> Stats {
        let mut stats = Stats::default();

        let millis_elapsed = {
            let mut guard = self.metrics_last_accessed.lock();
            let millis_elapsed = guard.elapsed().as_millis();
            *guard = Instant::now();
            millis_elapsed
        };

        stats.millis_elapsed = millis_elapsed;

        for shard in &self.shards {
            let shard = shard.read();
            stats.hit_count += shard.hit_count();
            stats.miss_count += shard.miss_count();
            stats.eviction_count += shard.eviction_count();
            shard.reset_counters();
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn it_inserts_and_gets_basic_values() {
        // given
        let cache = Cache::with_capacity(100);

        // when
        cache.insert("key1", "value1");

        // then
        assert_eq!(cache.get("key1"), Some("value1"));
        assert_eq!(cache.get("key2"), None);
    }

    #[test]
    fn it_updates_existing_value() {
        // given
        let cache = Cache::with_capacity(100);
        cache.insert("key1", "value1");

        // when
        let old_value = cache.insert("key1", "new_value");

        // then
        assert_eq!(old_value, Some("value1"));
        assert_eq!(cache.get("key1"), Some("new_value"));
    }

    #[test]
    fn it_handles_zero_capacity() {
        // given
        let cache = Cache::with_capacity(0);

        // when
        cache.insert("key1", "value1");

        // then
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn it_handles_one_capacity() {
        // given
        let cache = Cache::with_capacity(1);

        // when
        cache.insert("key1", "value1");

        // then
        assert_eq!(cache.get("key1"), Some("value1"));
        assert_eq!(cache.get("key2"), None);
    }

    #[test]
    fn it_works_with_custom_hasher() {
        // given
        use std::collections::hash_map::RandomState;
        let cache = Cache::with_capacity_and_hasher(100, RandomState::new());

        // when
        cache.insert("key1", "value1");

        // then
        assert_eq!(cache.get("key1"), Some("value1"));
    }

    #[test]
    fn it_is_thread_safe() {
        // given
        let cache: Arc<Cache<String, String>> = Arc::new(Cache::with_capacity(1_000));
        let mut handles = vec![];

        // when
        for i in 0..5 {
            let cache_clone = Arc::clone(&cache);
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            let handle = thread::spawn(move || {
                // Insert value
                cache_clone.insert(key.clone(), value.clone());
                // Read value
                assert_eq!(cache_clone.get(&key), Some(value));
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // then
        for i in 0..5 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            assert_eq!(cache.get(&key), Some(value));
        }
    }

    #[test]
    fn it_respects_capacity_limits() {
        // given
        let cache = Cache::with_capacity(2);

        // when
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");
        cache.insert("key3", "value3");
        cache.insert("key4", "value4");

        // then
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn it_returns_and_resets_stats() {
        // given
        let cache = Cache::with_capacity(1_000);

        // when
        for i in 0..10 {
            cache.insert(i, i);
        }

        // 5 hits
        for i in 0..5 {
            cache.get(&i);
        }

        // 5 misses
        for i in 10..15 {
            cache.get(&i);
        }

        // then
        let stats = cache.stats();
        assert_eq!(stats.hit_count, 5);
        assert_eq!(stats.miss_count, 5);

        let stats = cache.stats();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 0);
    }
}
