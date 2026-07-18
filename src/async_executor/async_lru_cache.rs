use lru::LruCache;
use std::{
    hash::Hash,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use crate::async_executor::Eventual;

pub trait AsyncLoader {
    type Key;
    type Value;

    fn load(key: Self::Key) -> impl Future<Output = Self::Value> + Send;
}

/// An LRU cache that loads items asynchronously.
///
/// ## Generic parameters:
/// - `L`: The `AsyncLoader` that'll generate futures for loading items
/// - `E`: The `Executor` that'll be used to execute those futures
pub struct AsyncLruCache<L>
where
    L: AsyncLoader,
{
    /// Wow, that's quite the type. Let me break it down for you:
    /// - The combination of `Arc` and `Mutex` allows us to sync and modify the `LruCache` between threads.
    ///   We need this because the worker thread accesses and updates the `LruCache` at the same time the
    ///   user might be accessing the cache via `.load()`.
    /// - `LruCache` is the underlying data structure behind this struct. It's key type is a generic
    ///   parameter (`K`), and its value type is a `FutureStatus` that either contains a pending future
    ///   or the result of the future.
    /// - `Pin<Box<dyn Future<Output = V> + Send>>` is just the boilerplate for the type that means "a
    ///   future that's heap allocated, safe to send between threads, and returns something of type V".
    ///   It's what we poll to load items into the cache.
    /// 
    /// TODO: Update the above comment
    cache: Arc<Mutex<LruCache<L::Key, Eventual<L::Value>>>>,
}

impl<L> AsyncLruCache<L>
where
    L: AsyncLoader + 'static,
    L::Key: Hash + Eq + Clone + Send,
    L::Value: Send + Sync + Clone,
{
    pub fn new(cache_size: NonZeroUsize) -> Self {
        let cache = Arc::new(Mutex::new(LruCache::new(cache_size)));

        Self {
            cache,
        }
    }

    /// Get an item from the cache. If the item isn't in the cache, or the cache isn't
    /// done loading the item, return None.
    pub fn load(&mut self, key: L::Key) -> Option<L::Value> {
        // Lock the mutex guarding the cache
        let cache = &mut self.cache.lock().unwrap();

        match cache.get(&key) {
            Some(eventual) => eventual.get().map(|v| v.clone()),
            None => {
                let eventual = super::thread_executor::schedule(L::load(key.clone()));
                let out = eventual.get().map(|v| v.clone());
                cache.put(key, eventual);
                out
            }
        }
    }

    /// Is the given key hot in the cache? Returns false if the key is not in the cache or its value
    /// is still pending.
    pub fn is_loaded(&self, key: &L::Key) -> bool {
        self.cache
            .lock()
            .unwrap()
            .peek(key)
            .is_some_and(|eventual| eventual.is_loaded())
    }
}

// TODO: Add tests
