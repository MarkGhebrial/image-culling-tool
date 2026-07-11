use std::ptr::NonNull;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use std::alloc::{Layout, alloc, dealloc};

unsafe fn my_alloc<T>() -> NonNull<T> {
    let layout = Layout::new::<T>();
    let raw_ptr = unsafe { alloc(layout) } as *mut T;
    NonNull::new(raw_ptr).unwrap()
}

// #[derive(Clone)]
// struct OneInner {
//     /// Indicates if either of the two owners have been dropped.
//     other_was_dropped: NonNull<AtomicBool>,
// }
// impl OneInner {
//     fn new() -> Self {
//         let layout = Layout::new::<AtomicBool>();
//         let owner_was_dropped = unsafe {
//             let raw_ptr = alloc(layout) as *mut AtomicBool;
//             let mut ptr = NonNull::new(raw_ptr).unwrap();
//             *ptr.as_mut() = AtomicBool::new(false);
//             ptr
//         };

//         Self { owner_was_dropped }
//     }
// }

// impl Drop for OneInner {
//     fn drop(&mut self) {
//         // If the inner value has already been dropped once, that means that this is the second drop
//         // and we must deallocate the inner value
//         unsafe {
//             // If the value has already been dropped by the OneCell
//             if self.owner_was_dropped.as_ref().load(Ordering::SeqCst) {
//                 println!("Second inner drop (deallocating the atomic bool)");
//                 dealloc(
//                     self.owner_was_dropped.as_ptr() as *mut u8,
//                     Layout::new::<AtomicBool>(),
//                 );
//             }
//             // This is the first drop of the inner value, so indicate to the OneCell that it should take care of the deallocation
//             else {
//                 println!("First inner drop (storing true into the atomic bool)");
//                 self.owner_was_dropped
//                     .as_mut()
//                     .store(true, Ordering::SeqCst);
//             }
//         }
//     }
// }

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

// struct WeakCell<T> {
//     /// It may be non-null, but it may be dangling!
//     value_ptr: NonNull<T>,

//     // The pointer is not null, but it may point to freed memory!
//     inner: OneInner,
// }
// impl<T> WeakCell<T> {
//     pub fn is_owner_alive(&self) -> bool {
//         unsafe { !self.inner.owner_was_dropped.as_ref().load(Ordering::SeqCst) }
//     }

//     pub fn promote<'a>(&'a self) -> Option<PromotedWeakCell<T>> {
//         if self.is_owner_alive() {
//             unsafe {
//                 Some(PromotedWeakCell {
//                     value_ref: self.value_ptr.as_ref(),
//                     inner: self.inner.clone(),
//                 })
//             }
//         } else {
//             None
//         }
//     }

//     pub fn get(&self) -> Option<&T> {
//         if self.is_owner_alive() {
//             // This is unsound if the OwnedCell deallocates the value while this reference is held! BIG NO NO!!! DO NOT DO THIS!!!
//             unsafe { Some(self.value_ptr.as_ref()) }
//         } else {
//             None
//         }
//     }
// }

// struct PromotedWeakCell<'a, T> {
//     value_ref: &'a T,
//     inner: OneInner,
// }
// impl<'a, T> Deref for PromotedWeakCell<'a, T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         self.value_ref
//     }
// }

// unsafe impl<T> Send for WeakCell<T> {}
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
