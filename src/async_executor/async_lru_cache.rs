use eframe::egui::ahash::HashMap;
use lru::LruCache;
use std::{
    hash::Hash,
    num::NonZeroUsize,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Poll, Wake},
    thread::Thread,
};

use crate::async_executor::{Eventual, Executor};

pub trait AsyncLoader {
    type Key;
    type Value;

    fn load(key: Self::Key) -> impl Future<Output = Self::Value> + Send;
}

/// TODO: Refactor AsyncLruCache to be generic over this
pub trait AsyncCacheStore {
    type Key;
    type Value;

    fn get(&mut self, key: &Self::Key) -> Option<&Self::Value>;
}
impl<K, V> AsyncCacheStore for LruCache<K, V>
where
    K: Hash + Eq,
{
    type Key = K;
    type Value = V;

    fn get(&mut self, key: &K) -> Option<&V> {
        LruCache::get(self, key)
    }
}
impl<K, V> AsyncCacheStore for HashMap<K, V>
where
    K: Hash + Eq,
{
    type Key = K;
    type Value = V;

    fn get(&mut self, key: &K) -> Option<&V> {
        HashMap::get(self, key)
    }
}


/// An LRU cache that loads items asynchronously.
///
/// Generic parameters:
/// - `K`: The key of the LRU cache
/// - `V`: The value of the LRU cache
/// - `C`: The async closure that's used to load items into the cache
/// - `Fut`: The type of the future returned by `C`
pub struct AsyncLruCache<L, E>
where
    L: AsyncLoader,
    E: Executor,
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
    cache: Arc<Mutex<LruCache<L::Key, Eventual<E, L::Value>>>>,

    /// The executor used to execute the futures returned by the loader
    executor: E,
}

impl<L, E> AsyncLruCache<L, E>
where
    E: Executor,
    L: AsyncLoader + 'static,
    L::Key: Hash + Eq + Clone + Send,
    L::Value: Send + Sync + Clone,
{
    pub fn new(cache_size: NonZeroUsize, executor: E) -> Self {
        // let cache: Arc<
        //     Mutex<LruCache<L::Key, FutureStatus<Pin<Box<dyn Future<Output = L::Value> + Send>>>>>,
        // > = Arc::new(Mutex::new(LruCache::new(cache_size)));

        let cache: Arc<Mutex<LruCache<L::Key, Eventual<E, L::Value>>>> =
            Arc::new(Mutex::new(LruCache::new(cache_size)));

        // let worker_thread_kill_mutex = Arc::new(Mutex::new(false));

        // Create a weak pointer to the cache structure. The worker thread doesn't need to outlive the
        // AsyncLruCache object, so
        // let cache_copy = Arc::downgrade(&cache);
        // let kill_mutex_copy = worker_thread_kill_mutex.clone();

        // Spawn the thread that'll service all the outstanding futures
        // let thread_handle = std::thread::spawn(move || {
        //     // Wakes up this thread when a future finishes doing its thing
        //     let waker = Arc::new(ThreadWaker(thread::current()));
        //     loop {
        //         // If the kill mutex is true, break out of the loop and kill the thread.
        //         if *kill_mutex_copy.lock().unwrap() {
        //             break;
        //         }

        //         for (_, future_status) in cache_copy.upgrade().unwrap().lock().unwrap().iter_mut() {
        //             future_status.poll(&mut Context::from_waker(&waker.clone().into()));
        //         }

        //         // Park the thread until the waker wakes it up. TODO: What happens to this thread when
        //         // the AsyncLruCache object is deallocated? Does this thread stay parked forever?
        //         thread::park();
        //     }
        // });

        Self {
            cache,
            executor,
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
                let eventual = self.executor.schedule(L::load(key.clone()));
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
