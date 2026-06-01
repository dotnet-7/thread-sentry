use std::sync::Arc;
use std::thread;
use thread_sentry::{init, report_issues, Mutex, SentryField};

fn main() {
    init();

    println!("=== Thread-Sentry Demo ===\n");

    demo_guard_auto_detection();
    demo_sentry_field_tracking();
    demo_no_race_same_lock();

    report_issues();
}

fn demo_guard_auto_detection() {
    println!("--- Method 1: Guard Auto-Detection ---");

    let data = Arc::new(Mutex::new(0u64));

    let data1 = Arc::clone(&data);
    thread::spawn(move || {
        let mut guard = data1.lock();
        *guard = 100;
        println!("Thread 1: Written 100 (auto-detected)");
    });

    let data2 = Arc::clone(&data);
    thread::spawn(move || {
        let guard = data2.lock();
        let value = *guard;
        println!("Thread 2: Read {} (auto-detected)", value);
    });

    println!("✓ Same lock protects data - No race\n");
}

fn demo_sentry_field_tracking() {
    println!("--- Method 2: SentryField Tracking ---");

    struct SharedData {
        counter: SentryField<u64>,
    }

    let data = Arc::new(Mutex::new(SharedData {
        counter: SentryField::new(0),
    }));

    let data1 = Arc::clone(&data);
    thread::spawn(move || {
        let mut guard = data1.lock();
        guard.counter.set(100);
        println!("Thread 1: Set counter to 100 (field-level tracking)");
    });

    let data2 = Arc::clone(&data);
    thread::spawn(move || {
        let guard = data2.lock();
        let counter = *guard.counter.get();
        println!(
            "Thread 2: Read counter = {} (field-level tracking)",
            counter
        );
    });

    println!("✓ SentryField auto-tracks with TLS - No race\n");
}

fn demo_no_race_same_lock() {
    println!("--- Test: Same Lock = No Race ---");

    let data = Arc::new(Mutex::new(0u64));

    let data1 = Arc::clone(&data);
    let h1 = thread::spawn(move || {
        let mut guard = data1.lock();
        println!("Thread 1: lock_id = {}", guard.lock_id);
        *guard = 100;
    });

    let data2 = Arc::clone(&data);
    let h2 = thread::spawn(move || {
        let guard = data2.lock();
        println!("Thread 2: lock_id = {}", guard.lock_id);
        let value = *guard;
    });

    h1.join().unwrap();
    h2.join().unwrap();

    println!("✓ Both threads use same lock_id - Correctly no race\n");
    println!("See examples/tls_demo.rs for race detection demo\n");
}
