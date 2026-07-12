use std::{
    pin::{pin},
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread::{self, Thread},
};

struct ThreadWaker(Thread);
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Block the current thread, polling the future until it's completed.
pub fn block_on_future<Fut>(future: Fut) -> <Fut as Future>::Output
where
    Fut: Future,
{
    let handle = std::thread::current();
    let mut future = pin!(future);

    let waker: Waker = Arc::new(ThreadWaker(handle)).into();

    loop {
        match future.as_mut().poll(&mut Context::from_waker(&waker)) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park(),
        }
    }
}
