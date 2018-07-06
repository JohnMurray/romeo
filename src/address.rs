use super::actor::{Actor, Receives};
use super::cell::Cell;

use std::boxed::FnBox;
use std::sync::Weak;

// ---
// Address
// ---
pub struct Address<A: Actor> {
    cell_ref: Weak<Cell<A>>,
}
impl<A: Actor + 'static> Address<A> {
    pub(crate) fn new(cell: Weak<Cell<A>>) -> Self {
        Address {
            cell_ref: cell,
        }
    }

    pub fn send<M: 'static>(&self, msg: M)
        where A: Receives<M>
    {
        if let Some(cell) = Weak::upgrade(&self.cell_ref) {
            let lambda_cell = cell.clone();
            let receive: Box<FnBox()> = Box::new(move || -> () {
                let actor_mutex = lambda_cell.actor_ref();
                let mut act = actor_mutex.lock().unwrap();
                act.receive(msg);
            });
            cell.receive(receive);
        } else {
            // TODO: raise some kind of error if the address is no longer valid (the cell
            //       it points to not longer exists)
        }
        // TODO: Need to wrap this in a mutex to ensure serial access to the mutable reference
    }
}

/// Implement a Copy for address, which is just to copy the underlying pointer to the cell
impl<A: Actor> Clone for Address<A> {
    fn clone(&self) -> Self {
        Address {
            cell_ref: self.cell_ref.clone(),
        }
    }
}

unsafe impl<A: Actor> Send for Address<A> {}
unsafe impl<A: Actor> Sync for Address<A> {}