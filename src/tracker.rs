use dashmap::DashMap;
use parking_lot::Mutex as ParkMutex;
use smallvec::SmallVec;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

pub type ThreadId = usize;
pub type LockId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LockInfo {
    pub lock_id: LockId,
    pub lock_type: LockType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockType {
    Mutex,
    RwLockRead,
    RwLockWrite,
}

#[derive(Debug, Clone)]
pub struct LockEvent {
    pub lock_id: LockId,
    pub lock_type: LockType,
    pub thread_id: ThreadId,
    pub acquired_at: Instant,
    pub backtrace: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ThreadLockState {
    pub held_locks: SmallVec<[(LockId, LockType); 4]>,
    pub waiting_for: Option<(LockId, LockType)>,
}

pub struct GlobalTracker {
    pub lock_events: DashMap<LockId, LockEvent>,
    pub thread_states: DashMap<ThreadId, ThreadLockState>,
    pub lock_graph: DashMap<(LockId, LockId), usize>,
    next_lock_id: ParkMutex<usize>,
    next_thread_id: ParkMutex<usize>,
}

impl GlobalTracker {
    pub fn new() -> Self {
        Self {
            lock_events: DashMap::new(),
            thread_states: DashMap::new(),
            lock_graph: DashMap::new(),
            next_lock_id: ParkMutex::new(1),
            next_thread_id: ParkMutex::new(1),
        }
    }

    pub fn allocate_lock_id(&self) -> LockId {
        let mut id = self.next_lock_id.lock();
        let lock_id = *id;
        *id += 1;
        lock_id
    }

    pub fn get_or_allocate_thread_id(&self) -> ThreadId {
        let _thread_id_str = format!("{:?}", thread::current().id());
        let mut id = self.next_thread_id.lock();
        let tid = *id;
        *id += 1;
        tid
    }

    pub fn record_lock_acquire(&self, lock_id: LockId, lock_type: LockType, thread_id: ThreadId) {
        let bt: Vec<String> = backtrace::Backtrace::new()
            .frames()
            .iter()
            .skip(3)
            .take(10)
            .map(|f| format!("{:?}", f))
            .collect();

        let event = LockEvent {
            lock_id,
            lock_type,
            thread_id,
            acquired_at: Instant::now(),
            backtrace: bt,
        };

        self.lock_events.insert(lock_id, event);

        self.thread_states
            .entry(thread_id)
            .and_modify(|state| {
                state.held_locks.push((lock_id, lock_type));
                state.waiting_for = None;
            })
            .or_insert(ThreadLockState {
                held_locks: smallvec::smallvec![(lock_id, lock_type)],
                waiting_for: None,
            });
    }

    pub fn record_lock_release(&self, lock_id: LockId, thread_id: ThreadId) {
        self.lock_events.remove(&lock_id);

        if let Some(mut state) = self.thread_states.get_mut(&thread_id) {
            state.held_locks.retain(|(id, _)| *id != lock_id);
        }
    }

    pub fn record_lock_wait(&self, lock_id: LockId, lock_type: LockType, thread_id: ThreadId) {
        self.thread_states
            .entry(thread_id)
            .and_modify(|state| {
                state.waiting_for = Some((lock_id, lock_type));
            })
            .or_insert(ThreadLockState {
                held_locks: SmallVec::new(),
                waiting_for: Some((lock_id, lock_type)),
            });

        if let Some(state) = self.thread_states.get(&thread_id) {
            for (held_id, _held_type) in &state.held_locks {
                self.lock_graph
                    .entry((*held_id, lock_id))
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }
    }
}

pub static GLOBAL_TRACKER: once_cell::sync::Lazy<Arc<GlobalTracker>> =
    once_cell::sync::Lazy::new(|| Arc::new(GlobalTracker::new()));
