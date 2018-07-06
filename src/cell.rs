use super::actor::Actor;
use super::address::Address;

use std::boxed::FnBox;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// ---
// Cell
// ---
pub(crate) struct Cell<A: Actor> {
    actor: Arc<Mutex<A>>,
    actor_producer: Box<Fn() -> A>,
    pub msg_queue: Mutex<VecDeque<Box<FnBox()>>>,
}

impl<A: Actor + 'static> Cell<A> {
    pub(crate) fn new(actor_producer: Box<Fn() -> A>) -> Arc<Self> {
        Arc::new(Cell{
            actor: Arc::new(Mutex::new(actor_producer())),
            actor_producer,
            msg_queue: Mutex::new(VecDeque::new()),
        })
    }

    pub(crate) fn actor_ref(&self) -> Arc<Mutex<A>> {
        self.actor.clone()
    }

    pub(crate) fn address(cell: Arc<Self>) -> Address<A> {
        Address::new(Arc::downgrade(&cell))
    }

    pub(crate) fn receive(&self, f: Box<FnBox()>) {
        let mut msg_queue = self.msg_queue.lock().unwrap();
        msg_queue.push_back(f);
    }
}

unsafe impl<A: Actor> Send for Cell<A> {}
unsafe impl<A: Actor> Sync for Cell<A> {}

// ---
// ACell, a parameter-type-less cell for the runtime
// ---
pub(crate) trait ACell: Send + Sync {
    fn process(&self) -> bool;
}
impl<A: Actor + 'static> ACell for Cell<A> {

    fn process(&self) -> bool {
        let mut msg_queue = self.msg_queue.lock().unwrap();
        if let Some(f) = msg_queue.pop_front() {
            f();
            return true
        }
        false
    }
}
