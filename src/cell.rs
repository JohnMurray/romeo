use super::actor::Actor;
use super::address::Address;

use std::boxed::FnBox;
use std::sync::{Arc, Mutex};

use crossbeam_channel as channel;

// ---
// Cell
// ---
pub(crate) struct Cell<A: Actor> {
    actor: Arc<Mutex<A>>,
    actor_producer: Box<Fn() -> A>,
    mailbox: channel::Receiver<Box<FnBox()>>,
    postman: channel::Sender<Box<FnBox()>>,
}

impl<A: Actor + 'static> Cell<A> {
    pub(crate) fn new(actor_producer: Box<Fn() -> A>) -> Arc<Self> {
        let (tx, rx) = channel::unbounded::<Box<FnBox()>>();
        Arc::new(Cell {
            actor: Arc::new(Mutex::new(actor_producer())),
            actor_producer,
            mailbox: rx,
            postman: tx,
        })
    }

    pub(crate) fn actor_ref(&self) -> Arc<Mutex<A>> {
        self.actor.clone()
    }

    pub(crate) fn address(cell: Arc<Self>) -> Address<A> {
        Address::new(Arc::downgrade(&cell), (&cell).postman.clone())
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
        if let Some(f) = self.mailbox.try_recv() {
            f();
            return true;
        }
        false
    }
}
