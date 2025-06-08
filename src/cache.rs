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

pub struct Cache<K, V, S = RandomState> {
    hash_builder: S,
    shards: Vec<RwLock<Shard<K, V, S>>>,
}

impl<K, V, S> Cache<K, V, S>
where
    K: Clone + Eq + Hash,
    V: Clone,
    S: BuildHasher,
{
    pub fn with_capacity(capacity: usize) -> Cache<K, V> {
        Cache::with_capacity_and_hasher(capacity, RandomState::new())
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let hash = self.hash_builder.hash_one(&key);
        let shard_lock = self.get_shard(hash);

        let mut shard = shard_lock.write();
        shard.insert(key, value)
    }

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
