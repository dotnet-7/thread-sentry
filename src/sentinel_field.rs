use std::ops::{Deref, DerefMut};

pub struct SentryField<T> {
    inner: T,
}

pub struct SentryFieldGuard<'a, T> {
    inner: &'a T,
    address: usize,
    thread_id: usize,
    lock_id: Option<usize>,
}

impl<T> SentryField<T> {
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }

    pub fn get(&self) -> SentryFieldGuard<'_, T> {
        let address = &self.inner as *const T as usize;
        let thread_id = crate::tls::get_current_thread_id()
            .unwrap_or_else(|| crate::tracker::GLOBAL_TRACKER.get_or_allocate_thread_id());
        let lock_id = crate::tls::get_current_lock();

        if let Some(report) = crate::race::GLOBAL_RACE_DETECTOR.record_access(
            address,
            thread_id,
            crate::race::AccessType::Read,
            lock_id,
            std::mem::size_of::<T>(),
        ) {
            crate::reporter::report_race(&report);
        }

        SentryFieldGuard {
            inner: &self.inner,
            address,
            thread_id,
            lock_id,
        }
    }

    pub fn set(&mut self, value: T) {
        let address = &mut self.inner as *mut T as usize;
        let thread_id = crate::tls::get_current_thread_id()
            .unwrap_or_else(|| crate::tracker::GLOBAL_TRACKER.get_or_allocate_thread_id());
        let lock_id = crate::tls::get_current_lock();

        if let Some(report) = crate::race::GLOBAL_RACE_DETECTOR.record_access(
            address,
            thread_id,
            crate::race::AccessType::Write,
            lock_id,
            std::mem::size_of::<T>(),
        ) {
            crate::reporter::report_race(&report);
        }

        self.inner = value;
    }

    pub fn get_mut(&mut self) -> SentryFieldMutGuard<'_, T> {
        let address = &mut self.inner as *mut T as usize;
        let thread_id = crate::tls::get_current_thread_id()
            .unwrap_or_else(|| crate::tracker::GLOBAL_TRACKER.get_or_allocate_thread_id());
        let lock_id = crate::tls::get_current_lock();

        if let Some(report) = crate::race::GLOBAL_RACE_DETECTOR.record_access(
            address,
            thread_id,
            crate::race::AccessType::Write,
            lock_id,
            std::mem::size_of::<T>(),
        ) {
            crate::reporter::report_race(&report);
        }

        SentryFieldMutGuard {
            inner: &mut self.inner,
            address,
            thread_id,
            lock_id,
        }
    }
}

impl<'a, T> Deref for SentryFieldGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

pub struct SentryFieldMutGuard<'a, T> {
    inner: &'a mut T,
    address: usize,
    thread_id: usize,
    lock_id: Option<usize>,
}

impl<'a, T> Deref for SentryFieldMutGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for SentryFieldMutGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<T: Default> Default for SentryField<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
