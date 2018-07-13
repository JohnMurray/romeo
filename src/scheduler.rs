use super::cell::ACell;

use std::sync::{Arc, Mutex};
use std::{thread, time};

use uuid::Uuid;

/// The Scheduler is responsible for scheduling execution of messages over actors for
/// a single thread. It also keeps references to underlying cells in order to process
/// messages and handle lifecycle events.
///
/// `Scheduler` is meant to be used on a single thread and is itself not meant to be
/// moved around threads. `Scheduler::start` will spawn a simple event-loop to process
/// actor messages and lifecycle events in it's care.
pub struct Scheduler {
    id: usize,
    cells: Mutex<Vec<Arc<ACell>>>,
}
impl Scheduler {
    pub fn new(id: usize) -> Scheduler {
        Scheduler {
            id,
            cells: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn add_cell(&self, cell: Arc<ACell>) {
        let mut cells = self.cells.lock().unwrap();
        cells.push(cell);
    }

    // simple, single-threaded, blocking event-loop runtime
    pub fn start(&self) {
        let base_backoff_us = 2;
        let max_backoff_us = 1_000_000;
        let exp = 2;

        let mut backoff_us = base_backoff_us;
        loop {
            let mut zero_work_loop = true;
            {
                let cells = self.cells.lock().unwrap();
                cells.iter().for_each(|cell| {
                    // TODO: Should be checking for a result type here and possibly triggering lifecycle
                    // TODO: methods (restart, shutdown, etc)
                    if cell.process() {
                        zero_work_loop = false;
                    }
                });
            }

            if zero_work_loop {
                thread::sleep(time::Duration::from_micros(backoff_us));
                trace!("Sleeping scheduler {} for {}us", self.id, backoff_us);
                backoff_us = (backoff_us * exp).min(max_backoff_us);
            } else {
                backoff_us = base_backoff_us;
            }
        }
    }

    /// Stop an actor by removing it from the scheduler and calling all lifecycle
    /// events related to shutdown.
    pub(crate) fn stop_actor(&self, uuid: Uuid) {
        let mut cells = self.cells.lock().unwrap();
        let mut remove_cell: Option<usize> = None;
        for (i, cell) in cells.iter().enumerate() {
            if cell.uuid() == uuid {
                remove_cell = Some(i);
                break;
            }
        }
        if let Some(i) = remove_cell {
            let cell = cells.remove(i);
            // TODO: call lifecycle shutdown hook
        }
    }
}
