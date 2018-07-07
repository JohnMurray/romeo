use super::actor::{Actor, ActorConstructable, Props};
use super::address::Address;
use super::cell::{ACell, Cell};

use std::sync::{Arc, Mutex};
use std::{time, thread};

// ---
// Runtime/System
// ---
pub struct Runtime {
    cells: Mutex<Vec<Arc<ACell>>>,
}
impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            cells: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn add_cell(&self, cell: Arc<ACell>) {
        let mut cells = self.cells.lock().unwrap();
        cells.push(cell);
    }

    pub fn new_actor<A, P>(&self, props: P) -> Address<A>
        where A: Actor + ActorConstructable<P> + 'static,
              P: Props + 'static,
    {
        let producer = Box::new(move || {
            A::new(&props)
        });
        let cell: Arc<Cell<A>> = Cell::new(producer);
        self.add_cell(cell.clone());
        Cell::address(cell)
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
