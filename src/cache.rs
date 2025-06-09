use parking_lot::RwLock;
use shard::Shard;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};
use std::num::NonZero;
use std::{cmp, thread};

mod entry;
mod fixed_size_hash_table;
mod ring_buffer;
mod shard;

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
pub struct Cache<K, V, S = RandomState> {
    hash_builder: S,
    shards: Vec<RwLock<Shard<K, V, S>>>,
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
        let shard_lock = self.get_shard(hash);

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
        let shard_lock = self.get_shard(hash);

        let shard = shard_lock.read();
        shard.get(key)
    }

    fn get_shard(&self, hash: u64) -> &RwLock<Shard<K, V, S>> {
        let shard_idx = hash as usize % cmp::max(self.shards.len() - 1, 1);
        self.shards
            .get(shard_idx)
            .expect("modulo op must return valid shard index")
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
        let number_of_shards = cmp::min(
            thread::available_parallelism()
                .map(NonZero::get)
                .unwrap_or(1),
            capacity,
        ) * 4;

        let mut shards = Vec::with_capacity(number_of_shards);
        let capacity_per_shard = capacity.div_ceil(number_of_shards);

        for _ in 0..number_of_shards {
            let shard = Shard::with_capacity_and_hasher(capacity_per_shard, hash_builder.clone());
            shards.push(RwLock::new(shard))
        }

        Self {
            hash_builder,
            shards,
        }
    }
}
