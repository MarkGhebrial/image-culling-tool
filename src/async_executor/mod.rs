pub mod async_lru_cache;
pub mod block_on_future;
pub mod rw_cell;
pub mod thread_executor;

use std::sync::{Arc, OnceLock};

/// Represents a value of type T that'll be eventually be loaded by an executor of type E
pub struct Eventual<E, T>
where
    E: Executor,
{
    id: E::EventualId,
    executor: E,
    // value_state: Mutex<LoadedState<T>>,
    value: Arc<OnceLock<T>>,
}
impl<E, T> Eventual<E, T>
where
    E: Executor,
{
    pub fn get(&self) -> Option<&T> {
        self.value.get()
    }
}
impl<E, T> Drop for Eventual<E, T>
where
    E: Executor,
{
    fn drop(&mut self) {
        self.executor.cancel(&self.id);
    }
}

pub trait Executor: Sized {
    /// Used to uniquely identify the `Eventual`s that are handed out
    type EventualId;

    fn schedule<'a, Fut>(&'a self, task: Fut) -> Eventual<Self, <Fut as Future>::Output>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send + Sync;

    fn cancel(&self, eventual: &Self::EventualId);
}
