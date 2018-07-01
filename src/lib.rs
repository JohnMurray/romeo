#![allow(dead_code)]

#[macro_use] extern crate log;

pub mod romeo {

    use std::result::Result;
    use std::sync::{Arc, Mutex, mpsc};
    use std::collections::HashMap;
    use std::clone::Clone;
    use std::fmt::Display;
    use std::marker::PhantomData;

    /// Props is the "properties" or values needed to construct an instance of
    /// an actor. It is necessary along with `ActorConstructable` to ensure that
    /// actors can be (re)created/started as needed throughout the lifetime of
    /// the actor system.
    pub trait Props {}

    /// ActorConstructable defines a constructor that uses `Props` in order to
    /// create an instance of an Actor.
    /// TODO: Should all actors be created on the heap?
    pub trait ActorConstructable<P>
        where P: Props
    {
        fn new(p: &P) -> Self;
    }

    /// ActorCell is the container for the actor. It manages the mailbox for the
    /// actor as well as managing the necessary data of the actor lifecycle (such
    /// as props to construct the actor). For each actor instance, there is exactly
    /// one cell.
    pub struct ActorCell<A: 'static, P: 'static, M: 'static>
        where A: ActorConstructable<P>,
              P: Props,
              A: Receives<M>,
    {
        actor: Box<A>,
        props: P,
        mailbox: mpsc::Receiver<M>,
        writer: mpsc::Sender<M>,
    }
    impl<A, P, M> ActorCell<A, P, M>
        where A: ActorConstructable<P>,
              P: Props,
              A: Receives<M>,
    {
        fn new(props: P) -> Self {
            let (sender, receiver) = mpsc::channel::<M>();
            ActorCell {
                actor: Box::new(A::new(&props)),
                props: props,
                mailbox: receiver,
                writer: sender,
            }
        }

        fn address(&self) -> ActorAddress<A, M> {
            ActorAddress {
                sender: self.writer.clone(),
                _phantom: PhantomData
            }
        }
    }
    trait ACell {
        fn try_receive(&mut self) -> bool;
    }
    impl<A, P, M> ACell for ActorCell<A, P, M>
        where A: ActorConstructable<P>,
              P: Props,
              A: Receives<M>,
    {
        fn try_receive(&mut self) -> bool {
            match self.mailbox.try_recv() {
                Ok(msg) => true,
                _       => false,
            }
        }
    }

    /// Address is like an actor ref. It contains a reference to the actor
    ///
    pub struct ActorAddress<A, M>
        where A: Receives<M>
    {
        sender: mpsc::Sender<M>,
        _phantom: PhantomData<A>,
    }
    impl<A, M> ActorAddress<A, M>
        where A: Receives<M>,
    {
        pub fn send(&self, msg: M)
        {
            self.sender.send(msg).unwrap();
        }
    }
    pub trait Addressable<A> {
        fn send<M>(&self, msg: M)
            where A: Receives<M>,
                  M: Display;
    }

    pub trait Receives<A> {
        fn send(&mut self, msg: A);
    }



    /// System is a handler to the actor-runtime. It does nothing more than contain
    /// a reference to the actual runtime. Since Actors implicitly live within a
    /// concurrent context, the System handle ensures that all communication to the
    /// runtime is thread-safe. So really, it's just a convencience interface over
    /// the runtime.
    pub struct System {
        runtime: Arc<Mutex<SystemRuntime>>
    }

    impl System {
        pub fn new(name: &String) -> System {
            System {
                runtime: Arc::new(Mutex::new(SystemRuntime::new(name))),
            }
        }
        /// Start the actor system
        pub fn start(&mut self) -> Result<(), &'static   str> {
            let mut runtime = self.runtime.lock().unwrap();
            if runtime.is_running() {
                error!("Attempted to start already running system: {0}", runtime.name());
                return Err("System is already running");
            }
            trace!("Starting actor system {0}", runtime.name());
            runtime.start();
            Ok(())
        }

        pub fn actor<A: 'static, P: 'static, M: 'static>(&mut self, name: String, props: P) -> ActorAddress<A, M>
            where A: ActorConstructable<P>,
                  P: Props,
                  A: Receives<M>
        {
            let mut runtime = self.runtime.lock().unwrap();

            let cell = Box::new(ActorCell::new(props));
            let address = cell.address();
            runtime.store_actor_cell(name, cell);

            address
        }
    }

    /// For a given instatiation of an actor system, there exists a single runtime.
    /// SystemRuntime is exactly that. At the moment it's rather empty, but it will
    /// be responsible for all system seutp and configuration as well as tear-down.
    struct SystemRuntime {
        name: String,
        running: bool,
        actor_registry: HashMap<String, Box<ACell>>,
    }

    impl SystemRuntime {
        pub fn new(name: &String) -> Self {
            SystemRuntime {
                name: name.clone(),
                running: false,
                actor_registry: HashMap::new(),
            }
        }

        pub fn name(&self) -> &String {
            &self.name
        }

        pub fn is_running(&self) -> bool {
            self.running
        }

        pub fn start(&mut self) {
            self.running = true;
            // TODO: spawn some threads, each with what is essentially an event-loop scheduler.
            // Make look into tokio for providing the event-loop.
        }

        pub fn store_actor_cell(&mut self, name: String, ac: Box<ACell>) {
            self.actor_registry.insert(name, ac);
        }
    }
}