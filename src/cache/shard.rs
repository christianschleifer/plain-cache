use crate::cache::RandomState;
use crate::cache::entry::{Entry, EntryPointer};
use crate::cache::fixed_size_hash_table::FixedSizeHashTable;
use crate::cache::ring_buffer::RingBuffer;
use std::borrow::Borrow;
use std::cmp;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

#[derive(Debug)]
pub(crate) struct Shard<K, V, S = RandomState> {
    entry_pointers: HashMap<K, EntryPointer, S>,
    small_queue: RingBuffer<Entry<K, V>>,
    main_queue: RingBuffer<Entry<K, V>>,
    ghost_queue: FixedSizeHashTable<K, S>,
}

impl<K, V, S> Shard<K, V, S>
where
    S: BuildHasher + Clone,
{
    pub(crate) fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        let small_fifo_queue_size = cmp::max(capacity / 10, 1);
        let main_fifo_queue_size = cmp::max(capacity - small_fifo_queue_size, 1);

        Self {
            entry_pointers: HashMap::<K, EntryPointer, S>::with_capacity_and_hasher(
                capacity,
                hash_builder.clone(),
            ),
            small_queue: RingBuffer::with_capacity(small_fifo_queue_size),
            main_queue: RingBuffer::with_capacity(main_fifo_queue_size),
            ghost_queue: FixedSizeHashTable::with_capacity_and_hasher(
                main_fifo_queue_size,
                hash_builder,
            ),
        }
    }
}

impl<K, V, S> Shard<K, V, S>
where
    K: Clone + Eq + Hash,
    S: BuildHasher,
    V: Clone,
{
    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        let previous_item = if self.entry_pointers.contains_key(&key) {
            match self.entry_pointers.get(&key).expect("just checked") {
                EntryPointer::MainQueue(index) => {
                    self.main_queue.remove(*index).map(|entry| entry.value)
                }
                EntryPointer::SmallQueue(index) => {
                    self.small_queue.remove(*index).map(|entry| entry.value)
                }
            }
        } else {
            None
        };

        let entry = Entry::new(key.clone(), value);

        if self.ghost_queue.contains(&key) {
            self.insert_into_main_queue(entry);
        } else {
            self.insert_into_small_queue(entry);
        }

        previous_item
    }

    fn insert_into_main_queue(&mut self, entry: Entry<K, V>) -> Option<V> {
        if self.main_queue.is_full() {
            self.evict_main_queue();
        }

        let key = entry.key.clone();

        let index = self
            .main_queue
            .push_back(entry)
            .expect("expecting space after eviction");

        self.entry_pointers
            .insert(key, EntryPointer::MainQueue(index));

        None
    }

    fn evict_main_queue(&mut self) {
        loop {
            if let Some(entry) = self.main_queue.pop_front() {
                let num_accessed = entry.get_num_accessed();
                if num_accessed > 0 {
                    let decremented_by_one = cmp::max(0, num_accessed - 1);
                    self.reinsert_into_main_queue(entry, decremented_by_one);
                    continue;
                } else {
                    self.entry_pointers.remove(&entry.key);
                    return;
                }
            }

            return;
        }
    }

    fn insert_into_small_queue(&mut self, entry: Entry<K, V>) -> Option<V> {
        if self.small_queue.is_full() {
            self.evict_small_queue();
        }

        let key = entry.key.clone();

        let index = self
            .small_queue
            .push_back(entry)
            .expect("there must be space after eviction");

        self.entry_pointers
            .insert(key, EntryPointer::SmallQueue(index));

        None
    }

    fn evict_small_queue(&mut self) {
        if let Some(entry) = self.small_queue.pop_front() {
            if entry.get_num_accessed() > 1 {
                // add the entry to the main queue, reset the access counter, and update the pointer

                if self.main_queue.is_full() {
                    self.evict_main_queue();
                }

                let pointer = self.entry_pointers.get_mut(&entry.key).expect(
                    "an entry popped from the small queue must be present in the entry pointers",
                );

                entry.set_num_accessed(0);

                let index = self
                    .main_queue
                    .push_back(entry)
                    .expect("there must be space after eviction");

                *pointer = EntryPointer::MainQueue(index);
            } else {
                // remove the entry and add the key to the ghost queue

                self.entry_pointers.remove(&entry.key);
                self.ghost_queue.insert(entry.key);
            };
        }
    }

    fn reinsert_into_main_queue(&mut self, entry: Entry<K, V>, num_accessed: u8) {
        let pointer = self
            .entry_pointers
            .get_mut(&entry.key)
            .expect("an entry popped from the main queue must be present in the entry pointers");

        entry.set_num_accessed(num_accessed);

        let index = self
            .main_queue
            .push_back(entry)
            .expect("there must be space after eviction");

        *pointer = EntryPointer::MainQueue(index);
    }

    pub(crate) fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        match self.entry_pointers.get(key)? {
            EntryPointer::MainQueue(index) => {
                let entry = self
                    .main_queue
                    .get(*index)
                    .expect("an entry must exist for an entry pointer");
                Self::update_access_count(entry);
                Some(entry.value.clone())
            }
            EntryPointer::SmallQueue(index) => {
                let entry = self
                    .small_queue
                    .get(*index)
                    .expect("an entry must exist for an entry pointer");
                Self::update_access_count(entry);
                Some(entry.value.clone())
            }
        }
    }

    fn update_access_count(entry: &Entry<K, V>) {
        let current_val = entry.get_num_accessed();

        if current_val >= 3 {
            return;
        }

        entry.increment_num_accessed(current_val);
    }
}
