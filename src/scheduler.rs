use super::cell::ACell;

use std::sync::{Arc, Mutex};
use std::{thread, time};

/// The Scheduler is responsible for scheduling execution of messages over actors for
/// a single thread. It also keeps references to underlying cells in order to proccess
/// messages and handle lifecycle events.
pub struct Scheduler {
    cells: Mutex<Vec<Arc<ACell>>>,
}
impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            cells: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn add_cell(&self, cell: Arc<ACell>) {
        let mut cells = self.cells.lock().unwrap();
        cells.push(cell);
    }

    // simple, single-threaded, blocking event-loop runtime
    pub fn start(&self) {
        loop {
            let cells = self.cells.lock().unwrap();
            cells.iter().for_each(|cell| {
                cell.process();
            });
            drop(cells);
            thread::sleep(time::Duration::from_millis(1000));
            println!("completed iteration");
        }
    }
}
