//! A store shared between the bus service thread and the control thread.

/// Imports `sync::{Arc, Mutex, MutexGuard, PoisonError}`,
///  `time::Instant` from `std`.
use std::{
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Instant,
};

/// A lock-protected store plus the compositor's clock.
///
/// What:
///      `pub struct Shared<T> { inner: Arc<Mutex<T>>, start_time: Instant }`.
///  Cloning makes another owner of the same store.
/// Why:
///       The service thread records client calls while the control thread reads them,
///  and both stamp events on one clock.
#[derive(Debug)]
pub struct Shared<T> {
    /// The store.
    inner: Arc<Mutex<T>>,
    /// When the compositor started;
    ///  every timestamp counts from here.
    start_time: Instant,
}

/// Another owner of the same store.
impl<T> Clone for Shared<T> {
    /// Shares the store and copies the clock.
    fn clone(&self) -> Self {
        return Self { inner: Arc::clone(&self.inner), start_time: self.start_time };
    }
}

/// Creating,
///  locking,
///  and stamping.
impl<T> Shared<T> {
    /// Wraps a new store.
    #[must_use]
    pub fn new(store: T, start_time: Instant) -> Self {
        return Self { inner: Arc::new(Mutex::new(store)), start_time };
    }

    /// Locks the store.
    ///
    /// What:
    ///      `pub fn lock(&self) -> MutexGuard<'_, T>`.
    /// Why:
    ///       A poisoned lock still holds consistent records,
    ///  because each store call completes
    ///  before the lock is released,
    ///  so the guard is recovered rather than panicking the compositor.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        return self.inner.lock().unwrap_or_else(PoisonError::into_inner);
    }

    /// Returns the milliseconds since the compositor started.
    #[must_use]
    pub fn now_ms(&self) -> u64 {
        return u64::try_from(self.start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
    }
}
