pub mod async_lru_cache;
pub mod block_on_future;
#[allow(unused)] // We're not currently using anything from this module
pub mod rw_cell;
pub mod thread_executor;
pub mod future_status;

use std::sync::{Arc, OnceLock};

/// Represents a value of type T that'll be eventually be loaded by an executor of type E
pub struct Eventual<T> {
    // cancel_handle: Arc<Mutex<bool>>,
    value: Arc<OnceLock<T>>,
}

impl<T> Eventual<T>
{
    pub fn get(&self) -> Option<&T> {
        self.value.get()
    }

    pub fn is_loaded(&self) -> bool {
        self.get().is_some()
    }
}
impl<T> Drop for Eventual<T> {
    fn drop(&mut self) {
        // *self.cancel_handle.lock() = true;
    }
}

// pub trait Executor: Sized {
//     /// Used to uniquely identify the `Eventual`s that are handed out
//     type EventualId;

//     fn schedule<'a, Fut>(&'a self, task: Fut) -> Eventual<<Fut as Future>::Output>
//     where
//         Fut: Future + Send + 'static,
//         Fut::Output: Send + Sync;

//     fn cancel(&self, eventual: &Self::EventualId);
// }
