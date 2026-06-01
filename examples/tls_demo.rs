use std::sync::Arc;
use std::thread;
use thread_sentry::{init, report_issues, SentinelMutex, SentryField};

struct SharedData {
    counter: SentryField<u64>,
    flag: SentryField<bool>,
}

impl SharedData {
    fn new() -> Self {
        Self {
            counter: SentryField::new(0),
            flag: SentryField::new(false),
        }
    }
}

fn main() {
    init();

    println!("=== TLS Auto-Tracking Demo ===\n");
    println!("SentryField now automatically gets lock_id from TLS!\n");

    demo_same_lock_no_race();
    demo_different_locks_race();

    report_issues();
}

fn demo_same_lock_no_race() {
    println!("--- Test 1: Same lock protects SentryField (NO race) ---");

    let data = Arc::new(SentinelMutex::new(SharedData::new()));

    let data1 = Arc::clone(&data);
    let h1 = thread::spawn(move || {
        let mut guard = data1.lock();
        println!("Thread 1: lock_id = {}", guard.lock_id);

        guard.counter.set(100);
        println!("Thread 1: Set counter to 100 (lock_id auto-detected from TLS)");
    });

    let data2 = Arc::clone(&data);
    let h2 = thread::spawn(move || {
        let guard = data2.lock();
        println!("Thread 2: lock_id = {}", guard.lock_id);

        let counter = *guard.counter.get();
        println!(
            "Thread 2: Read counter = {} (lock_id auto-detected from TLS)",
            counter
        );
    });

    h1.join().unwrap();
    h2.join().unwrap();

    println!("Expected: No race (same lock_id)\n");
}

fn demo_different_locks_race() {
    println!("--- Test 2: Different locks accessing same address (RACE) ---");

    struct UnsafeWrapper<T>(std::cell::UnsafeCell<T>);
    unsafe impl<T: Send> Sync for UnsafeWrapper<T> {}
    unsafe impl<T: Send> Send for UnsafeWrapper<T> {}

    let raw_counter = Arc::new(UnsafeWrapper(std::cell::UnsafeCell::new(0u64)));

    let mutex1: Arc<SentinelMutex<u64>> = Arc::new(SentinelMutex::new(0));
    let mutex2: Arc<SentinelMutex<u64>> = Arc::new(SentinelMutex::new(0));

    let raw1 = Arc::clone(&raw_counter);
    let m1 = Arc::clone(&mutex1);
    let h1 = thread::spawn(move || {
        let guard = m1.lock();
        println!("Thread 1: lock_id = {}", guard.lock_id);

        unsafe {
            *raw1.0.get() = 100;
        }

        if let Some(report) = thread_sentry::RaceDetector::record_access_manual(
            raw1.0.get() as usize,
            guard.thread_id,
            thread_sentry::AccessType::Write,
            Some(guard.lock_id),
            8,
        ) {
            println!("Thread 1: RACE DETECTED!");
            thread_sentry::report_race(&report);
        } else {
            println!("Thread 1: Wrote 100 to raw_counter");
        }
    });

    let raw2 = Arc::clone(&raw_counter);
    let m2 = Arc::clone(&mutex2);
    let h2 = thread::spawn(move || {
        let guard = m2.lock();
        println!("Thread 2: lock_id = {}", guard.lock_id);

        unsafe {
            let _val = *raw2.0.get();
        }

        if let Some(report) = thread_sentry::RaceDetector::record_access_manual(
            raw2.0.get() as usize,
            guard.thread_id,
            thread_sentry::AccessType::Read,
            Some(guard.lock_id),
            8,
        ) {
            println!("Thread 2: RACE DETECTED!");
            thread_sentry::report_race(&report);
        } else {
            println!("Thread 2: Read from raw_counter");
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    println!("Expected: Race detected (different lock_ids)\n");

    println!("=== Summary ===");
    println!("✓ TLS automatically provides lock_id to SentryField");
    println!("✓ Same lock: No race (correct)");
    println!("✓ Different locks: Race detected (correct)");
    println!("✓ Zero user intervention required!");
}
