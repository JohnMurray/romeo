use super::scheduler::Scheduler;

use std::sync::{Weak};

use uuid::Uuid;

// ---
// Base Actor Definition
// ---
pub trait Actor: Send + Sync {
    fn start(&mut self) {}
    fn pre_stop(&mut self) {}
}

// ---
// Constructor Traits
// ---
pub trait Props {}
pub trait ActorConstructable<P: Props>: Actor {
    fn new(props: &P) -> Self;
}

// ---
// Message Handling
// ---
pub trait Receives<M>
where
    Self: Actor,
{
    // TODO: should return something, because need to know about failures
    fn receive(&mut self, msg: M, ctx: &Context);
}


/// Context is the method by which an actor interacts with the running actor system. It
/// is preferable to using a system handle directly because the actions of the context
/// are tailored to the actor. This includes lifecycle actions (such as shutting down
/// or restarting) as well as actor-system events that are altered by the context, such
/// as creating an actor (which establishes a parent-child relationship).
///
/// A context does not have any state that persists between calls, so it doesn't expose any
/// methods for storing state, and should not be sought after as an alternative to defining
/// state within an actor.
pub struct Context {
    parent_cell_uuid: Uuid,
    running_state: State,
    /// In order to execute on some of the responsibilities of the context (such as shutting
    /// down), a weak reference to the scheduler must be maintained
    parent_scheduler: Weak<Scheduler>,
}
impl Context {
    pub(crate) fn new(uuid: Uuid, state: State, scheduler: Weak<Scheduler>) -> Self {
        Context {
            parent_cell_uuid: uuid,
            running_state: state,
            parent_scheduler: scheduler,
        }
    }
    pub fn stop(&self) {
        let scheduler = Weak::upgrade(&self.parent_scheduler);
        if scheduler.is_none() {
            // TODO: clean up error message
            panic!("Actor is no longer owned by a scheduler! I don't know how this happened, but it did.")
        }

        scheduler.unwrap().stop_actor(self.parent_cell_uuid);
    }
}

#[derive(Clone)]
pub(crate) enum State {
    Halted,
    Starting,
    Running,
    Restarting,
    Stopping
}