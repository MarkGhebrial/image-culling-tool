use std::ptr::NonNull;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use std::alloc::{Layout, alloc, dealloc};

unsafe fn my_alloc<T>() -> NonNull<T> {
    let layout = Layout::new::<T>();
    let raw_ptr = unsafe { alloc(layout) } as *mut T;
    NonNull::new(raw_ptr).unwrap()
}

struct TwoCellInner<T> {
    value: T,

    /// Indicates if the other cell in the TwoCell pair has been dropped
    other_was_dropped: AtomicBool,
}

/// A heap allocated value with exactly two owners. The value is deallocated when
impl<T> Drop for TwoCell<T> {
    fn drop(&mut self) {
        // If the inner value has already been dropped once, that means that this is the second drop
        // and we must deallocate.
        unsafe {
            // If the value has already been dropped by the WriteOneCell
            if self.inner.as_ref().other_was_dropped.load(Ordering::SeqCst) {
                dealloc(
                    self.inner.as_ptr() as *mut u8,
                    Layout::new::<TwoCellInner<T>>(),
                );
            }
            // This is the first drop of the inner value, so indicate to the other cell that it should take care of the deallocation
            else {
                self.inner
                    .as_mut()
                    .other_was_dropped
                    .store(true, Ordering::SeqCst);
            }
        }
    }
}

/// A smart pointer that has exactly one owned reference and one borrowed reference
struct TwoCell<T> {
    inner: NonNull<TwoCellInner<T>>,
}
impl<T> TwoCell<T> {
    pub fn new(value: T) -> (Self, Self) {
        let mut inner = unsafe { my_alloc::<TwoCellInner<T>>() };
        unsafe {
            *inner.as_mut() = TwoCellInner {
                value,
                other_was_dropped: AtomicBool::new(false),
            };
        }

        let a = Self {
            inner: inner.clone(),
        };

        let b = Self { inner };

        (a, b)
    }

    pub fn get(&self) -> &T {
        // Safety: since we still have self, the value has not been deallocated yet.
        unsafe { &self.inner.as_ref().value }
    }
}

unsafe impl<T> Send for TwoCell<T> {}

#[test]
fn one_cell_test() {
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Duration;

    let (a, b) = TwoCell::<OnceLock<usize>>::new(OnceLock::new());

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        b.get().set(42usize).unwrap();
    });

    loop {
        let value = a.get().get();
        println!("Reading: {:?}", value);
        if value.is_some() {
            assert_eq!(value, Some(&42usize));
            break;
        }
    }

    println!("Done: {:?}", a.get());
}
