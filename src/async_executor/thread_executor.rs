use std::{
    sync::{Arc, Mutex, OnceLock},
    thread::{self},
};

use crate::async_executor::{Eventual, block_on_future::block_on_future};

pub fn schedule<Fut>(task: Fut) -> Eventual<Fut::Output>
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + Sync,
{
    let cancel_handle = Arc::new(Mutex::new(false));

    let eventual = Eventual {
        // cancel_handle,//: cancel_handle.clone(),
        value: Arc::new(OnceLock::new()),
        // cancel_handle: todo!(),
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

// fn cancel(&self, eventual_id: &Self::EventualId) {
//     // Set the flag to true, to indicate to the eventual's thread that it should stop running
//     if let Ok(mut eventual_id) = eventual_id.lock() {
//         *eventual_id = true;
//     }
// }


