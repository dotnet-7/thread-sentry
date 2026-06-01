use crate::tracker::{GlobalTracker, LockId, LockType};
use dashmap::DashSet;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct DeadlockDetector {
    tracker: Arc<GlobalTracker>,
    reported: DashSet<(LockId, LockId)>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            tracker: Arc::clone(&crate::tracker::GLOBAL_TRACKER),
            reported: DashSet::new(),
        }
    }

    pub fn check_deadlock(&self) -> Vec<DeadlockReport> {
        let mut reports = Vec::new();
        let graph = self.build_wait_graph();
        let cycles = self.find_cycles(&graph);

        for cycle in cycles {
            if cycle.len() >= 2 {
                let key = (cycle[0], cycle[1]);
                if !self.reported.contains(&key) {
                    self.reported.insert(key);
                    reports.push(self.create_report(cycle));
                }
            }
        }

        reports
    }

    fn build_wait_graph(&self) -> HashMap<LockId, Vec<LockId>> {
        let mut graph: HashMap<LockId, Vec<LockId>> = HashMap::new();

        for entry in self.tracker.lock_graph.iter() {
            let (from, to) = entry.key();
            graph.entry(*from).or_default().push(*to);
        }

        graph
    }

    fn find_cycles(&self, graph: &HashMap<LockId, Vec<LockId>>) -> Vec<Vec<LockId>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in graph.keys() {
            self.dfs_cycle(
                *node,
                graph,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            );
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: LockId,
        graph: &HashMap<LockId, Vec<LockId>>,
        visited: &mut HashSet<LockId>,
        rec_stack: &mut HashSet<LockId>,
        path: &mut Vec<LockId>,
        cycles: &mut Vec<Vec<LockId>>,
    ) {
        if rec_stack.contains(&node) {
            if let Some(start) = path.iter().position(|&x| x == node) {
                cycles.push(path[start..].to_vec());
            }
            return;
        }

        if visited.contains(&node) {
            return;
        }

        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(neighbors) = graph.get(&node) {
            for neighbor in neighbors {
                self.dfs_cycle(*neighbor, graph, visited, rec_stack, path, cycles);
            }
        }

        path.pop();
        rec_stack.remove(&node);
    }

    fn create_report(&self, cycle: Vec<LockId>) -> DeadlockReport {
        let mut chain = Vec::new();

        for lock_id in &cycle {
            if let Some(event) = self.tracker.lock_events.get(lock_id) {
                chain.push(LockChainEntry {
                    lock_id: *lock_id,
                    lock_type: event.lock_type,
                    thread_id: event.thread_id,
                    backtrace: event.backtrace.clone(),
                });
            }
        }

        DeadlockReport {
            cycle_length: cycle.len(),
            lock_chain: chain,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeadlockReport {
    pub cycle_length: usize,
    pub lock_chain: Vec<LockChainEntry>,
}

#[derive(Debug, Clone)]
pub struct LockChainEntry {
    pub lock_id: LockId,
    pub lock_type: LockType,
    pub thread_id: usize,
    pub backtrace: Vec<String>,
}
