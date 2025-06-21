use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Stats {
    pub miss_count: u64,
    pub hit_count: u64,
    pub eviction_count: u64,
    pub millis_elapsed: u128,
}

#[derive(Debug, Default)]
pub(crate) struct Counters {
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    eviction_count: AtomicU64,
}

impl Counters {
    pub(crate) fn hit_count(&self) -> u64 {
        self.hit_count.load(Ordering::Acquire)
    }

    pub(crate) fn miss_count(&self) -> u64 {
        self.miss_count.load(Ordering::Acquire)
    }

    pub(crate) fn eviction_count(&self) -> u64 {
        self.eviction_count.load(Ordering::Acquire)
    }

    pub(crate) fn increment_hit_count(&self) {
        self.hit_count.fetch_add(1, Ordering::AcqRel);
    }

    pub(crate) fn increment_miss_count(&self) {
        self.miss_count.fetch_add(1, Ordering::AcqRel);
    }

    pub(crate) fn increment_eviction_count(&self) {
        self.eviction_count.fetch_add(1, Ordering::AcqRel);
    }

    pub(crate) fn reset(&self) {
        self.hit_count.store(0, Ordering::Release);
        self.miss_count.store(0, Ordering::Release);
        self.eviction_count.store(0, Ordering::Release);
    }
}
