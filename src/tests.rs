#[cfg(test)]
mod tests {
    use crate::{init, report_issues, Mutex, RwLock};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_mutex_basic() {
        init();

        let mutex = Mutex::new(0);
        let mut guard = mutex.lock();
        *guard += 1;
        drop(guard);

        assert_eq!(*mutex.lock(), 1);
    }

    #[test]
    fn test_rwlock_basic() {
        init();

        let rwlock = RwLock::new(42);

        {
            let guard = rwlock.read();
            assert_eq!(*guard, 42);
        }

        {
            let mut guard = rwlock.write();
            *guard = 100;
        }

        assert_eq!(*rwlock.read(), 100);
    }

    #[test]
    fn test_mutex_multithreaded() {
        init();

        let mutex = Arc::new(Mutex::new(0u64));
        let mut handles = vec![];

        for _ in 0..10 {
            let mutex_clone = Arc::clone(&mutex);
            let h = thread::spawn(move || {
                for _ in 0..1000 {
                    let mut guard = mutex_clone.lock();
                    *guard += 1;
                }
            });
            handles.push(h);
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(*mutex.lock(), 10_000);
    }

    #[test]
    fn test_rwlock_multithreaded() {
        init();

        let rwlock = Arc::new(RwLock::new(0u64));
        let mut handles = vec![];

        for i in 0..5 {
            let rwlock_clone = Arc::clone(&rwlock);
            let h = thread::spawn(move || {
                for _ in 0..100 {
                    let mut guard = rwlock_clone.write();
                    *guard += 1;
                }
            });
            handles.push(h);
        }

        for _ in 0..5 {
            let rwlock_clone = Arc::clone(&rwlock);
            let h = thread::spawn(move || {
                for _ in 0..100 {
                    let guard = rwlock_clone.read();
                    let _ = *guard;
                }
            });
            handles.push(h);
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(*rwlock.read(), 500);
    }
}
