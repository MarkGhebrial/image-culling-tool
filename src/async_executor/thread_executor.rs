use std::{
    sync::{Arc, Mutex, OnceLock},
    thread::{self},
};

use crate::async_executor::{Eventual, Executor, block_on_future::block_on_future};

pub struct ThreadExecutor;

impl ThreadExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Executor for ThreadExecutor {
    type EventualId = Arc<Mutex<bool>>;

    fn schedule<Fut>(&self, task: Fut) -> Eventual<Self, <Fut as Future>::Output>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send + Sync,
    {
        let id = Arc::new(Mutex::new(false));

        let eventual = Eventual {
            id: id.clone(),
            executor: Self::new(),
            value: Arc::new(OnceLock::new()),
        };

        // Spawn a thread that'll poll the future and give its result to the eventual. One thread per eventual
        // let weak_eventual = Arc::downgrade(&eventual);
        let cell = Arc::downgrade(&eventual.value);
        thread::spawn(move || {
            let output = block_on_future(task);
            // let _ = cell.set(output);
            if let Some(cell) = cell.upgrade() {
                // Set the OnceLock. If the value is already set, we ignore the error
                let _ = cell.set(output);
            }
        });

        eventual
    }

    fn cancel(&self, eventual_id: &Self::EventualId) {
        // Set the flag to true, to indicate to the eventual's thread that it should stop running
        if let Ok(mut eventual_id) = eventual_id.lock() {
            *eventual_id = true;
        }
    }
}

#[test]
fn foo_bar() {
    let executor = ThreadExecutor::new();

    let eventual = executor.schedule(async { 50usize });

    loop {
        if eventual.get().is_some() {
            assert_eq!(*eventual.get().unwrap(), 50usize);
        };
    }
}
