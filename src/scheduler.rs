use super::cell::ACell;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::{thread, time};

use uuid::Uuid;

/// The Scheduler is responsible for scheduling execution of messages over actors for
/// a single thread. It also keeps references to underlying cells in order to process
/// messages and handle lifecycle events.
///
/// `Scheduler` is meant to be used on a single thread and is itself not meant to be
/// moved around threads. `Scheduler::start` will spawn a simple event-loop to process
/// actor messages and lifecycle events in it's care.
pub(crate) struct Scheduler {
    id: usize,
    cells: RwLock<Vec<Arc<ACell>>>,

    // queues for various actions to be performed in the event-loop
    actor_starts: Mutex<VecDeque<Arc<ACell>>>,
    actor_stops:  Mutex<VecDeque<Arc<ACell>>>,
}
impl Scheduler {
    pub(crate) fn new(id: usize) -> Scheduler {
        Scheduler {
            id,
            cells: RwLock::new(Vec::new()),

            actor_starts: Mutex::new(VecDeque::new()),
            actor_stops:  Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn register_new_cell(&self, cell: Arc<ACell>) {
        let mut cells = self.cells.write().unwrap();
        cells.push(cell.clone());

        let mut starts = self.actor_starts.lock().unwrap();
        starts.push_back(cell);
    }

    /// Start executes a simple event-loop on the current thread. The event loop is blocking and
    /// will not exit on it's own. All actor messages and lifecycle events are processed within
    /// this event loop.
    ///
    /// ## Notes for Future Self
    /// This event-loop implementation was not meant to be the end-all be-all of event-loops. It is
    /// simply a placeholder to get things working so that the gross mechanics of romeo could be
    /// built. It likely makes sense to replace this with a more fully-featured event loop in the
    /// future to avoid spending too many cycles on this code. But alas, that was more to learn than
    /// I could handle when I implemented this.
    pub(crate) fn start(&self) {
        let base_backoff_us = 2;
        let max_backoff_us = 1_000_000;
        let exp = 2;

        let mut backoff_us = base_backoff_us;
        loop {
            trace!("[Tick] Scheduler {}", self.id);
            let mut zero_work_loop = true;
            {
                // start a new actor if one is available
                {
                    let mut starts = self.actor_starts.lock().unwrap();
                    if let Some(actor) = starts.pop_front() {
                        actor.start();
                        zero_work_loop = false;
                    }
                }

                // stop an existing actor if one is available
                {
                    let mut stops = self.actor_stops.lock().unwrap();
                    let mut cells = self.cells.write().unwrap();
                    if let Some(actor) = stops.pop_front() {
                        // call shutdown handle
                        actor.shutdown();

                        // remove the cell from the scheduler
                        let cell_lookup = cells.iter().enumerate()
                            .find(|(_, c)| c.uuid() == actor.uuid())
                            .map(|(i, c)| (i, c.uuid()));
                        if let Some((id, uuid)) = cell_lookup {
                            cells.remove(id);
                            debug!("Removed cell from scheduler: {}", uuid);
                        }

                        zero_work_loop = false;
                    }
                }

                // handle all actors, processing a message if one is available
                {
                    let cells = self.cells.read().unwrap();
                    cells.iter().for_each(|cell| {
                        // TODO: Should be checking for a result type here and possibly triggering lifecycle
                        // TODO: methods (restart, shutdown, etc)
                        if cell.process() {
                            zero_work_loop = false;
                        }
                    });
                }
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

    /// Stop an actor by removing it from the list of actors to process messages for and
    /// adding it to the queue of actors to shutdown. This ensures that the actor shutdown
    /// is processed on a scheduler as opposed to whatever context this function might
    /// be called from.
    ///
    /// Note that removal of the actor from the scheduler happens within the schedulers
    /// event loops (to avoid dead-locks).
    pub(crate) fn stop_actor(&self, uuid: Uuid) {
        let cells = self.cells.read().unwrap();
        let cell = cells.iter().find(|c| c.uuid() == uuid);
        cell.map(|c| self.actor_stops.lock().unwrap().push_back(c.clone()));
    }
}
