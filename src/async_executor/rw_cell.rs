use std::ptr::NonNull;
use std::sync::atomic::Ordering;
use std::{cell::UnsafeCell, sync::atomic::AtomicBool};

use std::alloc::{Layout, alloc, dealloc};

/// A heap allocated value with exactly two owners. One owner has exclusive write access to the inner
/// value.
struct OneInner<T> {
    value: T,
    /// Indicates if either of the two owners have been dropped.
    owner_was_dropped: AtomicBool,
}
impl<T> Drop for OneCell<T> {
    fn drop(&mut self) {
        // If the inner value has already been dropped once, that means that this is the second drop
        // and we must deallocate the inner value
        unsafe {
            // If the value has already been dropped by the WriteOneCell
            if self.inner.as_ref().owner_was_dropped.load(Ordering::SeqCst) {
                dealloc(self.inner.as_ptr() as *mut u8, Layout::new::<OneInner<T>>());
            }
            // This is the first drop of the inner value, so indicate to the WriteOneCell that it should take care of the deallocation
            else {
                self.inner.as_mut().owner_was_dropped.store(true, Ordering::SeqCst);
            }
        }
    }
}

/// A smart pointer that has exactly one owned reference and one borrowed reference
struct OneCell<T> {
    inner: NonNull<OneInner<T>>,
}
impl<T> OneCell<T> {
    pub fn new(value: T) -> (Self, WriteOneCell<T>) {
        let layout = Layout::new::<OneInner<T>>();
        let inner_ptr = unsafe {
            let raw_ptr = alloc(layout) as *mut OneInner<T>;
            let mut ptr = NonNull::new(raw_ptr).unwrap();
            *ptr.as_mut() = OneInner {
                value,
                owner_was_dropped: AtomicBool::new(false),
            };
            ptr
        };

        let one = Self { inner: inner_ptr };

        let w = WriteOneCell { inner: inner_ptr };

        (one, w)
    }

    pub fn get(&self) -> &T {
        // Safety: since we still have self, the inner value has not been deallocated yet, so its pointer remains valid.
        unsafe { &self.inner.as_ref().value }
    }
}

struct WriteOneCell<T> {
    // The pointer is not null, but it may point to freed memory!
    inner: NonNull<OneInner<T>>,
}
impl<T> WriteOneCell<T> {
    pub fn set(&mut self, value: T) {
        // Safety: we're the only ones that can write to this value
        unsafe {
            self.inner.as_mut().value = value;
        }
    }
}

unsafe impl<T> Send for WriteOneCell<T> {}
unsafe impl<T> Send for OneCell<T> {}

impl<T> Drop for WriteOneCell<T> {
    fn drop(&mut self) {
        // If the inner value has already been dropped once, that means that this is the second drop
        // and we must deallocate the inner value
        unsafe {
            // If the value has already been dropped by the OneCell
            if self.inner.as_ref().owner_was_dropped.load(Ordering::SeqCst) {
                dealloc(self.inner.as_ptr() as *mut u8, Layout::new::<OneInner<T>>());
            }
            // This is the first drop of the inner value, so indicate to the OneCell that it should take care of the deallocation
            else {
                self.inner.as_mut().owner_was_dropped.store(true, Ordering::SeqCst);
            }
        }
    }
}

#[test]
fn one_cell_test() {
    use std::thread;
    use std::time::Duration;
    
    let (reader, mut writer) = OneCell::<Option<usize>>::new(None);

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        writer.set(Some(42));
    });

    let value = reader.get();
    loop {
        println!("Reading: {:?}", value.is_some());
        if value.is_some() {
            assert_eq!(value, &Some(42));
            break;
        }
    }

    println!("Done: {:?}", reader.get());
}