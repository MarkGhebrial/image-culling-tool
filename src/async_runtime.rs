use lru::LruCache;
use std::{
    hash::Hash,
    num::NonZeroUsize,
    pin::Pin,
    sync::{Arc, Mutex},
    task::Wake,
    thread::{self, JoinHandle, Thread},
};

pub trait AsyncLruCacheLoader {
    type Key;
    type Value;

    fn load(key: Self::Key) -> impl Future<Output = Self::Value> + Send;
}

enum FutureStatus<Fut>
where
    Fut: Future,
{
    Pending(Fut),
    /// The future is done being executed (the last call to `.poll()` returned `Poll::Ready`)
    Done(Fut::Output),
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
    L: AsyncLruCacheLoader,
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
    // waker: Arc<ThreadWaker<()>>,
    thread_handle: JoinHandle<()>,
}

// type Fut<V> = Future<Output = V>;
// type C<K, V> = Fn(K)

impl<L> AsyncLruCache<L>
where
    L: AsyncLruCacheLoader + 'static,
    L::Key: Hash + Eq + Clone + Send, // + 'static,
    L::Value: Send + Clone,           // + 'static,
{
    pub fn new(cache_size: NonZeroUsize) -> Self {
        let cache: Arc<
            Mutex<LruCache<L::Key, FutureStatus<Pin<Box<dyn Future<Output = L::Value> + Send>>>>>,
        > = Arc::new(Mutex::new(LruCache::new(cache_size)));

        // Create a weak
        let cache_copy = Arc::downgrade(&cache);

        // Spawn the thread that'll service all the outstanding futures
        let thread_handle = std::thread::spawn(move || {
            // Wakes up this thread when a future finishes doing its thing
            let waker = Arc::new(ThreadWaker(thread::current()));
            loop {
                for (_key, future_status) in
                    cache_copy.upgrade().unwrap().lock().unwrap().iter_mut()
                {
                    if let FutureStatus::Pending(future) = future_status {
                        match future
                            .as_mut()
                            .poll(&mut std::task::Context::from_waker(&waker.clone().into()))
                        {
                            std::task::Poll::Ready(value) => {
                                *future_status = FutureStatus::Done(value)
                            }
                            std::task::Poll::Pending => { /* Do nothing, just keep polling this future in the next iteration */
                            }
                        }
                    }
                }

                // Park the thread until the waker wakes it up
                thread::park();
            }
        });

        Self {
            cache,
            thread_handle,
        }
    }

    /// Get an item from the cache. If the item isn't in the cache, or the cache isn't
    /// done loading the item, return None.
    pub fn load<'a>(&'a mut self, key: L::Key) -> Option<L::Value>
    where
        L::Key: 'a,
    {
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
                cache.put(key, FutureStatus::Pending(future));

                // Resume the thread that handles the futures
                self.thread_handle.thread().unpark();

                None
            }
        }
    }
}

#[test]
fn test_async_lru_cache() {
    struct Loader;
    impl AsyncLruCacheLoader for Loader {
        type Key = String;
        type Value = f64;

        async fn load(key: Self::Key) -> Self::Value {
            foo().await;

            if key.starts_with("0") { 42.0 } else { 0.0 }
        }
    }

    let mut cache: AsyncLruCache<Loader> = AsyncLruCache::new(NonZeroUsize::new(2).unwrap());

    cache.load("0banana".into());
}

// struct MyWaker;
// impl std::task::Wake for MyWaker {
//     fn wake(self: Arc<Self>) {
//         println!("I'm being awoken!")
//     }
// }

async fn foo() -> bool {
    true
}

async fn bar(n: usize) -> bool {
    if foo().await { n % 2 == 0 } else { false }
}

// fn block_on_future<T, R>(mut future: Pin<&mut T>) -> R
// where
//     T: Future<Output = R>,
// {
//     loop {
//         let poll_result = future.as_mut().poll(&mut std::task::Context::from_waker(
//             &Arc::new(MyWaker).into(),
//         ));
//         match poll_result {
//             std::task::Poll::Ready(output) => return output,
//             std::task::Poll::Pending => { /* Do nothing, keep polling */ }
//         }
//     }
// }
