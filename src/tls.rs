use std::cell::RefCell;

thread_local! {
    static CURRENT_LOCK: RefCell<Option<usize>> = RefCell::new(None);
    static CURRENT_THREAD_ID: RefCell<Option<usize>> = RefCell::new(None);
}

pub fn set_current_lock(lock_id: usize) {
    CURRENT_LOCK.with(|l| *l.borrow_mut() = Some(lock_id));
}

pub fn clear_current_lock() {
    CURRENT_LOCK.with(|l| *l.borrow_mut() = None);
}

pub fn get_current_lock() -> Option<usize> {
    CURRENT_LOCK.with(|l| *l.borrow())
}

pub fn set_current_thread_id(thread_id: usize) {
    CURRENT_THREAD_ID.with(|t| *t.borrow_mut() = Some(thread_id));
}

pub fn clear_current_thread_id() {
    CURRENT_THREAD_ID.with(|t| *t.borrow_mut() = None);
}

pub fn get_current_thread_id() -> Option<usize> {
    CURRENT_THREAD_ID.with(|t| *t.borrow())
}
