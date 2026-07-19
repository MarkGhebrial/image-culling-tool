use std::{pin::Pin, task::Poll};

pub enum FutureStatus<Fut>
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
    pub fn from_future(future: Fut) -> Self {
        Self::Pending(future)
    }

    pub fn is_done(&self) -> bool {
        matches!(self, FutureStatus::Done(_))
    }

    pub fn unwrap_done(&self) -> &Fut::Output {
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