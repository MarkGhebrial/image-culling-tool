use eframe::egui::ahash::HashMap;
use lru::LruCache;
use std::{
    hash::Hash,
    num::NonZeroUsize,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake},
    thread::{self, JoinHandle, Thread},
};

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

enum FutureStatus<Fut>
where
    Fut: Future,
{
    /// The future is still pending (i.e. the last call to `.poll()` returned `Poll::Pending`)
    Pending(Fut),
    /// The future is done being executed (the last call to `.poll()` returned `Poll::Ready`)
    Done(Fut::Output),
}
impl<Fut> FutureStatus<Fut>
where
    Fut: Future,
{
    fn from_future(future: Fut) -> Self {
        Self::Pending(future)
    }

    fn is_done(&self) -> bool {
        matches!(self, FutureStatus::Done(_))
    }

    fn unwrap_done(&self) -> &Fut::Output {
        match self {
            Self::Done(output) => output,
            _ => panic!(),
        }
    }
}
impl<T, Fut> FutureStatus<Pin<T>>
where
    T: std::ops::DerefMut<Target = Fut>,
    Fut: Future + ?Sized,
{
    fn poll(&mut self, cx: &mut std::task::Context<'_>) -> Option<&Fut::Output> {
        match self {
            FutureStatus::Pending(future) => {
                // let poll_
                let poll_result = future.as_mut().poll(cx);
                match poll_result {
                    Poll::Ready(output) => {
                        *self = Self::Done(output);
                        Some(self.unwrap_done())
                    }
                    Poll::Pending => None,
                }
            }
            FutureStatus::Done(output) => Some(output),
        }
    }
}

struct ThreadWaker(Thread);
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// An LRU cache that loads items asynchronously.
///
/// Generic parameters:
/// - `K`: The key of the LRU cache
/// - `V`: The value of the LRU cache
/// - `C`: The async closure that's used to load items into the cache
/// - `Fut`: The type of the future returned by `C`
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
    cache:
        Arc<Mutex<LruCache<L::Key, FutureStatus<Pin<Box<dyn Future<Output = L::Value> + Send>>>>>>,

    /// A handle to the worker thread. Used for resuming the thread when a new future is added to the cache.
    thread_handle: JoinHandle<()>,
    /// Set this to true to kill the worker thread.
    worker_thread_kill_mutex: Arc<Mutex<bool>>,
}

impl<L> AsyncLruCache<L>
where
    L: AsyncLoader + 'static,
    L::Key: Hash + Eq + Clone + Send,
    L::Value: Send + Clone,
{
    pub fn new(cache_size: NonZeroUsize) -> Self {
        let cache: Arc<
            Mutex<LruCache<L::Key, FutureStatus<Pin<Box<dyn Future<Output = L::Value> + Send>>>>>,
        > = Arc::new(Mutex::new(LruCache::new(cache_size)));

        let worker_thread_kill_mutex = Arc::new(Mutex::new(false));

        // Create a weak pointer to the cache structure. The worker thread doesn't need to outlive the
        // AsyncLruCache object, so
        let cache_copy = Arc::downgrade(&cache);
        let kill_mutex_copy = worker_thread_kill_mutex.clone();

        // Spawn the thread that'll service all the outstanding futures
        let thread_handle = std::thread::spawn(move || {
            // Wakes up this thread when a future finishes doing its thing
            let waker = Arc::new(ThreadWaker(thread::current()));
            loop {
                // If the kill mutex is true, break out of the loop and kill the thread.
                if *kill_mutex_copy.lock().unwrap() {
                    break;
                }

                for (_, future_status) in cache_copy.upgrade().unwrap().lock().unwrap().iter_mut() {
                    future_status.poll(&mut Context::from_waker(&waker.clone().into()));
                }

                // Park the thread until the waker wakes it up. TODO: What happens to this thread when
                // the AsyncLruCache object is deallocated? Does this thread stay parked forever?
                thread::park();
            }
        });

        Self {
            cache,
            thread_handle,
            worker_thread_kill_mutex,
        }
    }

    /// Get an item from the cache. If the item isn't in the cache, or the cache isn't
    /// done loading the item, return None.
    pub fn load(&mut self, key: L::Key) -> Option<L::Value> {
        // Lock the mutex guarding the cache
        let cache = &mut self.cache.lock().unwrap();

        match cache.get(&key) {
            Some(future_status) => match future_status {
                FutureStatus::Pending(_) => None,
                // We can't return a reference here because the cache entry might become invalidated
                // at any time while the reference is still held
                FutureStatus::Done(output) => Some(output.clone()),
            },
            None => {
                // We don't have a pending future for this item, so add one
                let key_copy = key.clone();
                let future = Box::pin(L::load(key_copy));
                cache.put(key, FutureStatus::from_future(future));

                // Resume the thread that handles the futures
                self.thread_handle.thread().unpark();

                None
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
            .is_some_and(|future_status| future_status.is_done())
    }
}

impl<L> Drop for AsyncLruCache<L>
where
    L: AsyncLoader,
{
    fn drop(&mut self) {
        // Tell the worker thread to stop running
        *self.worker_thread_kill_mutex.lock().unwrap() = true;
        // Resume the worker thread so that it'll see the updated value of the worker_thread_kill_mutex
        self.thread_handle.thread().unpark();
    }
}

// TODO: Add tests

#[test]
fn foo() {
    struct DummyWaker;
    impl std::task::Wake for DummyWaker {
        fn wake(self: Arc<Self>) {
            // Do nothing
        }
    }

    let fut = std::pin::pin!(async { 1 });
    // let fut = async { 1 };
    let mut fs = FutureStatus::from_future(fut);

    fs.poll(&mut Context::from_waker(&Arc::new(DummyWaker).into()));
}
