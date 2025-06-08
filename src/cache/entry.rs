use std::sync::atomic::{AtomicU8, Ordering};

pub(crate) enum EntryPointer {
    MainQueue(usize),
    SmallQueue(usize),
}

pub(crate) struct Entry<K, V> {
    key: K,
    value: V,
    num_accessed: AtomicU8,
}

impl<K, V> Entry<K, V> {}

impl<K, V> Entry<K, V> {
    pub(crate) fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            num_accessed: AtomicU8::new(0),
        }
    }

    pub fn into_key(self) -> K {
        self.key
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub(crate) fn set_num_accessed(&self, val: u8) {
        self.num_accessed.store(val, Ordering::Release);
    }

    pub(crate) fn get_num_accessed(&self) -> u8 {
        self.num_accessed.load(Ordering::Acquire)
    }

    pub(crate) fn increment_num_accessed(&self, mut current_val: u8) {
        loop {
            match self.num_accessed.compare_exchange(
                current_val,
                current_val + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(new_val) => {
                    current_val = new_val;
                }
            }
        }
    }
}
