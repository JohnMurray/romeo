use super::actor::{self, Actor, Context};
use super::address::Address;
use super::scheduler::Scheduler;

use std::boxed::FnBox;
use std::cell::RefCell;
use std::sync::{Arc, Mutex, Weak};

use crossbeam_channel as channel;
use uuid::Uuid;

// ---
// Cell
// ---
pub(crate) struct Cell<A: Actor> {
    uuid: Uuid,
    actor: Arc<Mutex<A>>,
    actor_running_state: RefCell<actor::State>,
    actor_producer: Box<Fn() -> A>,
    mailbox: channel::Receiver<Box<FnBox()>>,
    postman: channel::Sender<Box<FnBox()>>,

    parent_scheduler: Weak<Scheduler>,
}

impl<A: Actor + 'static> Cell<A> {
    pub(crate) fn new(actor_producer: Box<Fn() -> A>, scheduler: Weak<Scheduler>) -> Arc<Self> {
        let (tx, rx) = channel::unbounded::<Box<FnBox()>>();
        Arc::new(Cell {
            uuid: Uuid::new_v4(),
            actor: Arc::new(Mutex::new(actor_producer())),
            actor_running_state: RefCell::new(actor::State::Starting),
            actor_producer,
            mailbox: rx,
            postman: tx,

            parent_scheduler: scheduler,
        })
    }

    pub(crate) fn actor_ref(&self) -> Arc<Mutex<A>> {
        self.actor.clone()
    }

    pub(crate) fn context(&self) -> Context {
        Context::new(self.uuid, self.actor_running_state.borrow().clone(), self.parent_scheduler.clone())
    }

    pub(crate) fn address(cell: Arc<Self>) -> Address<A> {
        Address::new(Arc::downgrade(&cell), (&cell).postman.clone())
    }

    /// Replace the current actor state with the `actor_producer` constructor
    /// lambda. Typically used with actor restarts. See `romeo::actor::Context::restart`.
    pub(crate) fn reset_actor_state(&self) {
        let mut actor = self.actor.lock().unwrap();
        *actor = (*self.actor_producer)();
    }
}

unsafe impl<A: Actor> Send for Cell<A> {}
unsafe impl<A: Actor> Sync for Cell<A> {}

// ---
// ACell, a parameter-type-less cell for the runtime
// ---
pub(crate) trait ACell: Send + Sync {
    fn process(&self) -> bool;
    fn uuid(&self) -> Uuid;
    fn start(&self);
    fn restart(&self);
    fn shutdown(&self);
}
impl<A: Actor + 'static> ACell for Cell<A> {
    fn process(&self) -> bool {
        if let Some(f) = self.mailbox.try_recv() {
            f();
            return true;
        }
        false
    }

    fn uuid(&self) -> Uuid {
        self.uuid.clone()
    }

    fn start(&self) {
        self.actor.lock().unwrap().start();
        self.actor_running_state.replace(actor::State::Running);
    }

    fn restart(&self) {
        self.shutdown();
        {
            let mut actor = self.actor.lock().unwrap();
            *actor = (*self.actor_producer)();
        }
        self.start();
    }

    fn shutdown(&self) {
        self.actor_running_state.replace(actor::State::Stopping);
        self.actor.lock().unwrap().pre_stop();
        self.actor_running_state.replace(actor::State::Halted);
    }
}

impl PartialEq for ACell {
    fn eq(&self, other: &ACell) -> bool {
        self.uuid() == other.uuid()
    }
}
impl Eq for ACell {}
