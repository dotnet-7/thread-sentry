use crate::tracker::{GlobalTracker, LockType};
use crate::{DeadlockDetector, RaceDetector};
use parking_lot::{RwLock as ParkRwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct SentinelRwLock<T> {
    inner: ParkRwLock<T>,
    lock_id: usize,
    tracker: Arc<GlobalTracker>,
    detector: Arc<DeadlockDetector>,
}

impl<T> SentinelRwLock<T> {
    pub fn new(data: T) -> Self {
        let tracker = Arc::clone(&*crate::tracker::GLOBAL_TRACKER);
        let lock_id = tracker.allocate_lock_id();

        Self {
            inner: ParkRwLock::new(data),
            lock_id,
            tracker,
            detector: Arc::new(DeadlockDetector::new()),
        }
    }

    pub fn read(&self) -> SentinelRwLockReadGuard<'_, T> {
        let thread_id = self.tracker.get_or_allocate_thread_id();

        self.tracker
            .record_lock_wait(self.lock_id, LockType::RwLockRead, thread_id);

        let reports = self.detector.check_deadlock();
        for report in reports {
            crate::reporter::report_deadlock(&report);
        }

        let guard = self.inner.read();

        self.tracker
            .record_lock_acquire(self.lock_id, LockType::RwLockRead, thread_id);

        crate::tls::set_current_lock(self.lock_id);
        crate::tls::set_current_thread_id(thread_id);

        SentinelRwLockReadGuard {
            guard,
            lock_id: self.lock_id,
            thread_id,
            tracker: Arc::clone(&self.tracker),
            race_detector: Arc::clone(&*crate::race::GLOBAL_RACE_DETECTOR),
        }
    }

    pub fn write(&self) -> SentinelRwLockWriteGuard<'_, T> {
        let thread_id = self.tracker.get_or_allocate_thread_id();

        self.tracker
            .record_lock_wait(self.lock_id, LockType::RwLockWrite, thread_id);

        let reports = self.detector.check_deadlock();
        for report in reports {
            crate::reporter::report_deadlock(&report);
        }

        let guard = self.inner.write();

        self.tracker
            .record_lock_acquire(self.lock_id, LockType::RwLockWrite, thread_id);

        crate::tls::set_current_lock(self.lock_id);
        crate::tls::set_current_thread_id(thread_id);

        SentinelRwLockWriteGuard {
            guard,
            lock_id: self.lock_id,
            thread_id,
            tracker: Arc::clone(&self.tracker),
            race_detector: Arc::clone(&*crate::race::GLOBAL_RACE_DETECTOR),
        }
    }

    pub fn try_read(&self) -> Option<SentinelRwLockReadGuard<'_, T>> {
        let thread_id = self.tracker.get_or_allocate_thread_id();

        if let Some(guard) = self.inner.try_read() {
            self.tracker
                .record_lock_acquire(self.lock_id, LockType::RwLockRead, thread_id);

            crate::tls::set_current_lock(self.lock_id);
            crate::tls::set_current_thread_id(thread_id);

            Some(SentinelRwLockReadGuard {
                guard,
                lock_id: self.lock_id,
                thread_id,
                tracker: Arc::clone(&self.tracker),
                race_detector: Arc::clone(&*crate::race::GLOBAL_RACE_DETECTOR),
            })
        } else {
            None
        }
    }

    pub fn try_write(&self) -> Option<SentinelRwLockWriteGuard<'_, T>> {
        let thread_id = self.tracker.get_or_allocate_thread_id();

        if let Some(guard) = self.inner.try_write() {
            self.tracker
                .record_lock_acquire(self.lock_id, LockType::RwLockWrite, thread_id);

            crate::tls::set_current_lock(self.lock_id);
            crate::tls::set_current_thread_id(thread_id);

            Some(SentinelRwLockWriteGuard {
                guard,
                lock_id: self.lock_id,
                thread_id,
                tracker: Arc::clone(&self.tracker),
                race_detector: Arc::clone(&*crate::race::GLOBAL_RACE_DETECTOR),
            })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

pub struct SentinelRwLockReadGuard<'a, T> {
    guard: RwLockReadGuard<'a, T>,
    pub lock_id: usize,
    pub thread_id: usize,
    tracker: Arc<GlobalTracker>,
    race_detector: Arc<RaceDetector>,
}

impl<'a, T> Deref for SentinelRwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = &*self.guard as *const T as usize;
        let data_size = std::mem::size_of::<T>();

        if let Some(report) = self.race_detector.record_access(
            ptr,
            self.thread_id,
            crate::race::AccessType::Read,
            Some(self.lock_id),
            data_size,
        ) {
            crate::reporter::report_race(&report);
        }

        &self.guard
    }
}

impl<'a, T> Drop for SentinelRwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        crate::tls::clear_current_lock();
        crate::tls::clear_current_thread_id();
        self.tracker
            .record_lock_release(self.lock_id, self.thread_id);
    }
}

pub struct SentinelRwLockWriteGuard<'a, T> {
    guard: RwLockWriteGuard<'a, T>,
    pub lock_id: usize,
    pub thread_id: usize,
    tracker: Arc<GlobalTracker>,
    race_detector: Arc<RaceDetector>,
}

impl<'a, T> Deref for SentinelRwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = &*self.guard as *const T as usize;
        let data_size = std::mem::size_of::<T>();

        if let Some(report) = self.race_detector.record_access(
            ptr,
            self.thread_id,
            crate::race::AccessType::Write,
            Some(self.lock_id),
            data_size,
        ) {
            crate::reporter::report_race(&report);
        }

        &self.guard
    }
}

impl<'a, T> DerefMut for SentinelRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = &mut *self.guard as *mut T as usize;
        let data_size = std::mem::size_of::<T>();

        if let Some(report) = self.race_detector.record_access(
            ptr,
            self.thread_id,
            crate::race::AccessType::Write,
            Some(self.lock_id),
            data_size,
        ) {
            crate::reporter::report_race(&report);
        }

        &mut self.guard
    }
}

impl<'a, T> Drop for SentinelRwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        crate::tls::clear_current_lock();
        crate::tls::clear_current_thread_id();
        self.tracker
            .record_lock_release(self.lock_id, self.thread_id);
    }
}
