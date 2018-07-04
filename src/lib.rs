#![feature(fnbox)]
#![allow(dead_code)]

use std::boxed::FnBox;
use std::cell::{RefCell, RefMut};
use std::collections::{VecDeque};
use std::ops::Fn;
use std::sync::{Arc, Weak, Mutex};
use std::{time, thread};

// ---
// Base Actor Definition
// ---
pub trait Actor {
    fn start();
    fn pre_stop();
}
// This doesn't allow for overrides... hrmm...
// impl<A> Actor for A { // default impl
//     fn start() {}
//     fn pre_stop() {}
// }

// ---
// Constructor Traits
// ---
pub trait Props {}
pub trait ActorConstructable<P: Props>: Actor {
    fn new(props: &P) -> Self;
}

// ---
// Cell
// ---
pub struct Cell<A: Actor> {
    actor: RefCell<A>,
    actor_producer: Box<Fn() -> A>,
    pub msg_queue: Mutex<RefCell<VecDeque<Box<FnBox()>>>>,
}
impl<A: Actor + 'static> Cell<A> {
    fn new(actor_producer: Box<Fn() -> A>) -> Arc<Self> {
        // TODO: does the Cell need to be an Arc? Or can it just hand out weak references from
        //       a Box<T>
        Arc::new(Cell{
            actor: RefCell::new(actor_producer()),
            actor_producer,
            msg_queue: Mutex::new(RefCell::new(VecDeque::new())),
        })
    }

    fn actor_ref(&self) -> RefMut<A> {
        self.actor.borrow_mut()
    }

    pub fn address(cell: Arc<Self>) -> Address<A> {
        Address::new(Arc::downgrade(&cell))
    }

    fn receive(&self, f: Box<FnBox()>) {
        let msg_queue = self.msg_queue.lock().unwrap();
        msg_queue.borrow_mut().push_back(f);
    }
}

// ---
// ACell, a parameter-type-less cell for the runtime
// ---
pub trait ACell {
    fn process(&self) -> bool;
}
impl<A: Actor + 'static> ACell for Cell<A> {

    fn process(&self) -> bool {
        let msg_queue = self.msg_queue.lock().unwrap();
        if let Some(f) = msg_queue.borrow_mut().pop_front() {
            f();
            return true
        }
        false
    }
}

// ---
// Message Handling
// ---
pub trait Receives<M>
    where Self: Actor,
{
    // TODO: should return something, because need to know about failures
    fn receive(&mut self, msg: M);
}

// ---
// Address
// ---
pub struct Address<A: Actor> {
    cell_ref: Weak<Cell<A>>,
}
impl<A: Actor + 'static> Address<A> {
    fn new(cell: Weak<Cell<A>>) -> Self {
        Address {
            cell_ref: cell,
        }
    }

    pub fn send<M: 'static>(&mut self, msg: M)
        where A: Receives<M>
    {
        if let Some(cell) = Weak::upgrade(&self.cell_ref) {
            let lambda_cell = cell.clone();
            let receive: Box<FnBox()> = Box::new(move || -> () {
                let mut act = lambda_cell.actor_ref();
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

// ---
// Public Interface
//   What would typically be defined on the actor system
//   but that we're doing very bare-bones for now.
// ---
// pub fn new_actor<A: 'static, P>(props: P) -> Arc<Box<Cell<A>>>
//     where A: Actor + ActorConstructable<A, P>,
//           P: Props + 'static,
// {
//     let producer = Box::new(move || {
//         A::new(&props)
//     });
//     Cell::new(producer)
// }

// ---
// Runtime/System
// ---
pub struct Runtime {
    cells: Mutex<RefCell<Vec<Arc<ACell>>>>,
}
impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            cells: Mutex::new(RefCell::new(Vec::new())),
        }
    }

    pub fn add_cell(&self, cell: Arc<ACell>) {
        let cells = self.cells.lock().unwrap();
        cells.borrow_mut().push(cell);
    }

    pub fn new_actor<A, P>(&self, props: P) -> Arc<Cell<A>>
        where A: Actor + ActorConstructable<P> + 'static,
              P: Props + 'static,
    {
        let producer = Box::new(move || {
            A::new(&props)
        });
        let cell: Arc<Cell<A>> = Cell::new(producer);
        self.add_cell(cell.clone());
        cell
    }

    // simple, single-threaded, blocking event-loop runtime
    pub fn start(&self) {
        loop {
            let cells = self.cells.lock().unwrap();
            cells.borrow().iter().for_each(|cell| {
                cell.process();
            });
            drop(cells);
            thread::sleep(time::Duration::from_millis(1000));
            println!("completed iteration");
        }
    }
}















// #![allow(dead_code)]
//
// #[macro_use] extern crate log;
//
// pub mod romeo {
//
//     use std::result::Result;
//     use std::sync::{Arc, Mutex, mpsc};
//     use std::collections::HashMap;
//     use std::clone::Clone;
//     use std::fmt::Display;
//     use std::marker::PhantomData;
//
//     /// Props is the "properties" or values needed to construct an instance of
//     /// an actor. It is necessary along with `ActorConstructable` to ensure that
//     /// actors can be (re)created/started as needed throughout the lifetime of
//     /// the actor system.
//     pub trait Props {}
//
//     /// ActorConstructable defines a constructor that uses `Props` in order to
//     /// create an instance of an Actor.
//     /// TODO: Should all actors be created on the heap?
//     pub trait ActorConstructable<P>
//         where P: Props
//     {
//         fn new(p: &P) -> Self;
//     }
//
//     /// ActorCell is the container for the actor. It manages the mailbox for the
//     /// actor as well as managing the necessary data of the actor lifecycle (such
//     /// as props to construct the actor). For each actor instance, there is exactly
//     /// one cell.
//     pub struct ActorCell<A: 'static, P: 'static, M: 'static>
//         where A: ActorConstructable<P>,
//               P: Props,
//               A: Receives<M>,
//     {
//         actor: Box<A>,
//         props: P,
//         mailbox: mpsc::Receiver<Message<A, P, M>>,
//         writer: mpsc::Sender<Message<A, P, M>>,
//     }
//     impl<A, P, M> ActorCell<A, P, M>
//         where A: ActorConstructable<P>,
//               P: Props,
//               A: Receives<M>,
//     {
//         fn new(props: P) -> Self {
//             let (sender, receiver) = mpsc::channel::<Message<A, P, M>>();
//             ActorCell {
//                 actor: Box::new(A::new(&props)),
//                 props: props,
//                 mailbox: receiver,
//                 writer: sender,
//             }
//         }
//
//         fn receive(&mut self, msg: M) {
//             self.receive(msg);
//         }
//
//         fn address(&self) -> ActorAddress<A, P, M> {
//             ActorAddress {
//                 sender: self.writer.clone(),
//                 cell: self,
//                 _phantom: PhantomData
//             }
//         }
//     }
//     trait ACell {
//         fn try_receive(&mut self) -> bool;
//     }
//     impl<A, P, M> ACell for ActorCell<A, P, M>
//         where A: ActorConstructable<P>,
//               P: Props,
//               A: Receives<M>,
//     {
//         fn try_receive(&mut self) -> bool {
//             match self.mailbox.try_recv() {
//                 Ok(msg) => true,
//                 _       => false,
//             }
//         }
//     }
//
//     // A message represents a self-contained executable unit
//     struct Message<A: 'static, P: 'static, M: 'static>
//         where A: ActorConstructable<P>,
//               P: Props,
//               A: Receives<M>,
//     {
//         cell: ActorCell<A, P, M>,
//         msg: M,
//     }
//     impl<A, P, M> Message<A, P, M>
//         where A: ActorConstructable<P>,
//         P: Props,
//         A: Receives<M>,
//     {
//         fn new(msg: M, cell: ActorCell<A, P, M>) -> Self {
//             Message {
//                 cell: cell,
//                 msg: msg,
//             }
//         }
//
//         fn process(mut self) -> bool {
//             self.cell.receive(self.msg);
//             true
//         }
//     }
//
//     /// Address is like an actor ref. It contains a reference to the actor
//     ///
//     pub struct ActorAddress<A: 'static, P: 'static, M: 'static>
//         where A: ActorConstructable<P>,
//               P: Props,
//               A: Receives<M>,
//     {
//         sender: mpsc::Sender<Message<A, P, M>>,
//         cell: &'static ActorCell<A, P, M>,
//         _phantom: PhantomData<A>,
//     }
//     impl<A, P, M> ActorAddress<A, P, M>
//         where A: ActorConstructable<P>,
//               P: Props,
//               A: Receives<M>,
//     {
//         pub fn send(&self, msg: M)
//         {
//             let message = Message::new(msg, self.cell);
//             self.sender.send(msg).unwrap();
//         }
//     }
//     pub trait Addressable<A> {
//         fn send<M>(&self, msg: M)
//             where A: Receives<M>,
//                   M: Display;
//     }
//
//     pub trait Receives<A> {
//         fn send(&mut self, msg: A);
//     }
//
//
//
//     /// System is a handler to the actor-runtime. It does nothing more than contain
//     /// a reference to the actual runtime. Since Actors implicitly live within a
//     /// concurrent context, the System handle ensures that all communication to the
//     /// runtime is thread-safe. So really, it's just a convencience interface over
//     /// the runtime.
//     pub struct System {
//         runtime: Arc<Mutex<SystemRuntime>>
//     }
//
//     impl System {
//         pub fn new(name: &String) -> System {
//             System {
//                 runtime: Arc::new(Mutex::new(SystemRuntime::new(name))),
//             }
//         }
//         /// Start the actor system
//         pub fn start(&mut self) -> Result<(), &'static   str> {
//             let mut runtime = self.runtime.lock().unwrap();
//             if runtime.is_running() {
//                 error!("Attempted to start already running system: {0}", runtime.name());
//                 return Err("System is already running");
//             }
//             trace!("Starting actor system {0}", runtime.name());
//             runtime.start();
//             Ok(())
//         }
//
//         pub fn actor<A: 'static, P: 'static, M: 'static>(&mut self, name: String, props: P) -> ActorAddress<A, P, M>
//             where A: ActorConstructable<P>,
//                   P: Props,
//                   A: Receives<M>
//         {
//             let mut runtime = self.runtime.lock().unwrap();
//
//             let cell = Box::new(ActorCell::new(props));
//             let address = cell.address();
//             runtime.store_actor_cell(name, cell);
//
//             address
//         }
//     }
//
//     /// For a given instatiation of an actor system, there exists a single runtime.
//     /// SystemRuntime is exactly that. At the moment it's rather empty, but it will
//     /// be responsible for all system seutp and configuration as well as tear-down.
//     struct SystemRuntime {
//         name: String,
//         running: bool,
//         actor_registry: HashMap<String, Box<ACell>>,
//     }
//
//     impl SystemRuntime {
//         pub fn new(name: &String) -> Self {
//             SystemRuntime {
//                 name: name.clone(),
//                 running: false,
//                 actor_registry: HashMap::new(),
//             }
//         }
//
//         pub fn name(&self) -> &String {
//             &self.name
//         }
//
//         pub fn is_running(&self) -> bool {
//             self.running
//         }
//
//         pub fn start(&mut self) {
//             self.running = true;
//             // TODO: spawn some threads, each with what is essentially an event-loop scheduler.
//             // Make look into tokio for providing the event-loop.
//         }
//
//         pub fn store_actor_cell(&mut self, name: String, ac: Box<ACell>) {
//             self.actor_registry.insert(name, ac);
//         }
//     }
// }