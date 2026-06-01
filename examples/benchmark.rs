use thread_sentry::{Mutex, init};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn main() {
    init();
    
    println!("Thread Sentry Performance Benchmark\n");
    println!("====================================\n");

    let iterations = 1_000_000;
    let num_threads = 8;

    println!("Test: {} iterations across {} threads\n", iterations, num_threads);

    let mutex = Arc::new(Mutex::new(0u64));
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..num_threads {
        let mutex_clone = Arc::clone(&mutex);
        let h = thread::spawn(move || {
            for _ in 0..iterations / num_threads {
                let mut guard = mutex_clone.lock();
                *guard += 1;
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    let duration = start.elapsed();
    let total_ops = iterations as f64;
    let ops_per_sec = total_ops / duration.as_secs_f64();
    let overhead_ns = duration.as_nanos() as f64 / total_ops;

    println!("Results:");
    println!("  Total time: {:?}", duration);
    println!("  Operations/sec: {:.0}", ops_per_sec);
    println!("  Overhead per lock: {:.2} ns", overhead_ns);
    println!("  Final value: {}", *mutex.lock());
    println!("\n✓ Benchmark completed successfully");
}