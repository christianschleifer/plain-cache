use crate::cache::RandomState;
use std::borrow::Borrow;
use std::cmp;
use std::hash::{BuildHasher, Hash};

pub(crate) struct FixedSizeHashTable<T, S = RandomState> {
    hash_builder: S,
    buckets: Vec<Option<T>>,
}

impl<T, S> FixedSizeHashTable<T, S>
where
    S: BuildHasher + Clone,
{
    pub(crate) fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        let mut buckets = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            buckets.push(None);
        }

        Self {
            hash_builder,
            buckets,
        }
    }
}

impl<T, S> FixedSizeHashTable<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    pub(crate) fn insert(&mut self, value: T) {
        if self.buckets.capacity() == 0 {
            return;
        }

        let hash = self.hash_builder.hash_one(&value);
        let bucket_idx = self.get_bucket_index(hash);

        self.buckets[bucket_idx] = Some(value);
    }

    pub(crate) fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        if self.buckets.capacity() == 0 {
            return false;
        }

        let hash = self.hash_builder.hash_one(value);
        let bucket_idx = self.get_bucket_index(hash);

        self.buckets
            .get(bucket_idx)
            .and_then(|opt| opt.as_ref())
            .filter(|item| Borrow::borrow(*item) == value)
            .is_some()
    }

    fn get_bucket_index(&self, hash: u64) -> usize {
        hash as usize % cmp::max(self.buckets.capacity().wrapping_sub(1), 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_initializes_buckets_with_none() {
        // when
        let hash_table: FixedSizeHashTable<String, RandomState> =
            FixedSizeHashTable::with_capacity_and_hasher(5, Default::default());

        // then
        assert_eq!(hash_table.buckets.len(), 5);
        hash_table
            .buckets
            .iter()
            .for_each(|bucket| assert_eq!(*bucket, None))
    }

    #[test]
    fn it_returns_true_for_contained_items() {
        // given
        let mut hash_table: FixedSizeHashTable<String, RandomState> =
            FixedSizeHashTable::with_capacity_and_hasher(5, Default::default());
        hash_table.insert(String::from("hello world"));

        // when
        let contained = hash_table.contains("hello world");

        // then
        assert!(contained)
    }

    #[test]
    fn it_returns_false_for_not_contained_items() {
        // given
        let mut hash_table: FixedSizeHashTable<String, RandomState> =
            FixedSizeHashTable::with_capacity_and_hasher(5, Default::default());
        hash_table.insert(String::from("hello world"));

        // when
        let contained = hash_table.contains("hello rust");

        // then
        assert!(!contained)
    }

    #[test]
    fn it_returns_false_for_overwritten_items() {
        // given
        let mut hash_table: FixedSizeHashTable<String, RandomState> =
            FixedSizeHashTable::with_capacity_and_hasher(1, Default::default());
        hash_table.insert(String::from("hello world"));
        hash_table.insert(String::from("hello rust"));

        // when
        let contained = hash_table.contains("hello world");

        // then
        assert!(!contained)
    }

    #[test]
    fn it_can_handle_zero_capacity() {
        // given
        let mut hash_table: FixedSizeHashTable<String, RandomState> =
            FixedSizeHashTable::with_capacity_and_hasher(0, Default::default());
        hash_table.insert(String::from("hello world"));

        // when
        let contained = hash_table.contains("hello world");

        // then
        assert!(!contained)
    }
}
