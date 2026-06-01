pub mod deadlock;
pub mod race;
pub mod reporter;
pub mod sentinel_field;
pub mod sentinel_mutex;
pub mod sentinel_rwlock;
pub mod tls;
pub mod tracker;

#[cfg(test)]
mod tests;

pub use deadlock::DeadlockDetector;
pub use race::{AccessType, MemoryAccess, RaceDetector, RaceReport, GLOBAL_RACE_DETECTOR};
pub use reporter::{print_report, report_deadlock, report_race};
pub use sentinel_field::{SentryField, SentryFieldGuard, SentryFieldMutGuard};
pub use sentinel_mutex::SentinelMutex;
pub use sentinel_rwlock::SentinelRwLock;
pub use tracker::GlobalTracker;

#[cfg(feature = "derive")]
pub use thread_sentry_derive::sentry_track;

pub type Mutex<T> = sentinel_mutex::SentinelMutex<T>;
pub type RwLock<T> = sentinel_rwlock::SentinelRwLock<T>;

pub fn init() {
    once_cell::sync::Lazy::force(&tracker::GLOBAL_TRACKER);
    once_cell::sync::Lazy::force(&race::GLOBAL_RACE_DETECTOR);
}

pub fn report_issues() {
    reporter::print_report();
}
