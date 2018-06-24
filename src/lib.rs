#[macro_use] extern crate log;

pub mod romeo {

    use std::result::Result;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    pub trait Message {}
    pub trait Props {}

    pub trait ActorConstructable<A, P>
        where P: Props
    {
        fn new(p: &P) -> A;
    }

    pub struct ActorRef<A> {
        actor: A
    }
    pub struct ActorCell<A> {
        actor: A
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

        pub fn actor<A, P>(name: &String, props: &P) -> ActorRef<A>
        where
            A: ActorConstructable<A, P>,
            P: Props,
        {
            ActorRef { actor: A::new(props) }
        }
    }

    /// For a given instatiation of an actor system, there exists a single runtime.
    /// SystemRuntime is exactly that. At the moment it's rather empty, but it will
    /// be responsible for all system seutp and configuration as well as tear-down.
    struct SystemRuntime {
        name: String,
        running: bool,
        actorRegistry: HashMap<String, ActorCell<_>>
    }

    impl SystemRuntime {
        pub fn new(name: &String) -> Self {
            SystemRuntime {
                name: name.clone(),
                running: false,
                actorRegistry: HashMap::new(),
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
    }
}