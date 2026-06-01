use crate::tracker::ThreadId;
use dashmap::DashMap;
use smallvec::SmallVec;
use std::sync::Arc;

pub static GLOBAL_RACE_DETECTOR: once_cell::sync::Lazy<Arc<RaceDetector>> =
    once_cell::sync::Lazy::new(|| Arc::new(RaceDetector::new()));

pub struct RaceDetector {
    memory_access: DashMap<usize, MemoryAccessRecord>,
    reported: DashMap<(usize, usize), bool>,
}

#[derive(Debug, Clone)]
pub struct MemoryAccessRecord {
    pub address: usize,
    pub accesses: SmallVec<[MemoryAccess; 8]>,
}

#[derive(Debug, Clone)]
pub struct MemoryAccess {
    pub thread_id: ThreadId,
    pub access_type: AccessType,
    pub lock_held: Option<usize>,
    pub backtrace: Vec<String>,
    pub timestamp: std::time::Instant, // When did this access occur
    pub data_size: usize,              // Size of accessed data (for overlapping detection)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
}

impl RaceDetector {
    pub fn new() -> Self {
        Self {
            memory_access: DashMap::new(),
            reported: DashMap::new(),
        }
    }

    pub fn record_access_manual(
        address: usize,
        thread_id: ThreadId,
        access_type: AccessType,
        lock_held: Option<usize>,
        data_size: usize,
    ) -> Option<RaceReport> {
        GLOBAL_RACE_DETECTOR.record_access(address, thread_id, access_type, lock_held, data_size)
    }

    pub fn record_access(
        &self,
        address: usize,
        thread_id: ThreadId,
        access_type: AccessType,
        lock_held: Option<usize>,
        data_size: usize,
    ) -> Option<RaceReport> {
        let bt: Vec<String> = backtrace::Backtrace::new()
            .frames()
            .iter()
            .skip(3)
            .take(8)
            .map(|f| format!("{:?}", f))
            .collect();

        let new_access = MemoryAccess {
            thread_id,
            access_type,
            lock_held,
            backtrace: bt,
            timestamp: std::time::Instant::now(),
            data_size,
        };

        let mut race_detected = None;

        self.memory_access
            .entry(address)
            .and_modify(|record| {
                for existing in &record.accesses {
                    if self.is_race(&existing, &new_access) {
                        let key = (existing.thread_id * 1000 + thread_id, address);
                        if !self.reported.contains_key(&key) {
                            self.reported.insert(key, true);
                            race_detected = Some(RaceReport {
                                address,
                                access1: existing.clone(),
                                access2: new_access.clone(),
                            });
                        }
                    }
                }
                record.accesses.push(new_access.clone());
            })
            .or_insert_with(|| MemoryAccessRecord {
                address,
                accesses: smallvec::smallvec![new_access],
            });

        race_detected
    }

    fn is_race(&self, a1: &MemoryAccess, a2: &MemoryAccess) -> bool {
        // Rule 1: Same thread accessing same address is safe (sequential execution)
        if a1.thread_id == a2.thread_id {
            return false;
        }

        // Rule 2: Both are reads is safe (read-read concurrency)
        if a1.access_type == AccessType::Read && a2.access_type == AccessType::Read {
            return false;
        }

        // Rule 3: Protected by the same lock is safe (lock ensures mutual exclusion)
        // Only if BOTH have locks AND they are the SAME lock
        if a1.lock_held.is_some() && a2.lock_held.is_some() && a1.lock_held == a2.lock_held {
            return false;
        }

        // Rule 4: All other cases are races
        // - Different threads (checked)
        // - Same address (implicit, as we're checking within same address record)
        // - At least one write (checked, not both reads)
        // - No common lock protection (checked)
        true
    }
}

#[derive(Debug, Clone)]
pub struct RaceReport {
    pub address: usize,
    pub access1: MemoryAccess,
    pub access2: MemoryAccess,
}
