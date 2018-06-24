#[macro_use] extern crate log;

pub mod romeo {

    use std::result::Result;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use std::clone::Clone;
    use std::fmt::{Display, Formatter};
    use std;

    pub trait Props {}

    pub trait ActorConstructable<P>
        where P: Props
    {
        fn new(p: &P) -> Self;
    }

    pub struct ActorAddress<A> {
        actor: A
    }
    pub trait Receives<A> {
        fn send(&mut self, msg: A);
    }
    pub trait Addressable<A> {
        fn send<M>(&self, msg: M)
            where A: Receives<M>,
                  M: Display;
    }
    pub struct ActorCell<A> {
        actor: A
    }


    //--- Test using above code, as verification it works
    struct TestActor {
        count: u8
    }
    impl Clone for TestActor {
        fn clone(&self) -> Self {
            TestActor { count: self.count }
        }
    }
    impl Display for TestActor {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            write!(f, "TestActor[ count: {0} ]", self.count)
        }
    }
    impl Props for TestActor {}
    impl ActorConstructable<TestActor> for TestActor {
        fn new(a: &TestActor) -> TestActor {
            return (*a).clone()
        }
    }
    impl Receives<u8> for TestActor {
        // TODO: I don't have an instance of TestActor here, this is useless
        fn send(&mut self, msg: u8) {
            self.count += msg;
        }
    }
    impl Addressable<TestActor> for ActorAddress<TestActor> {
        fn send<M>(&self, msg: M)
            where TestActor: Receives<M>,
                  M: Display,
        {
            println!("Received {0} for {1}", msg, &self.actor);
        }
    }

    fn main() {
        let props = TestActor { count: 5 };
        let address = ActorAddress { actor: TestActor::new(&props) };
        let x: u8 = 10;
        address.send(x);
        println!("Count: {0}", address.actor.count);
    }
    //---

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

        pub fn actor<A, P>(name: &String, props: &P) -> ActorAddress<A>
        where
            A: ActorConstructable<P>,
            P: Props,
        {
            ActorAddress { actor: A::new(props) }
        }
    }

    /// For a given instatiation of an actor system, there exists a single runtime.
    /// SystemRuntime is exactly that. At the moment it's rather empty, but it will
    /// be responsible for all system seutp and configuration as well as tear-down.
    struct SystemRuntime {
        name: String,
        running: bool,
        // actorRegistry: HashMap<String, ActorCell<_>>
    }

    impl SystemRuntime {
        pub fn new(name: &String) -> Self {
            SystemRuntime {
                name: name.clone(),
                running: false,
                // actorRegistry: HashMap::new(),
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