use super::actor::{Actor, ActorConstructable, Props};
use super::address::Address;
use super::cell::Cell;
use super::scheduler::Scheduler;

use std::fmt::{self, Display, Formatter};
use std::sync::Arc;
use std::thread;

use num_cpus;
use rand::{thread_rng, Rng, ThreadRng};

/// System is the main handle into a running actor system. It is responsible for creating
/// actors, starting the system (spawning threads and schedulers), stopping the system, etc.
///
/// To configure the system, please refer to `romeo::system::Config`.
pub struct System {
    thread_handles: Vec<thread::JoinHandle<()>>,
    thread_schedulers: Vec<Arc<Scheduler>>,
    config: Config,
    state: RunningState,
    rng: ThreadRng,
}

impl System {
    pub fn new() -> Self {
        System {
            thread_handles: vec![],
            thread_schedulers: vec![],
            config: Config::default(),
            state: RunningState::AwaitingStart,
            rng: thread_rng(),
        }
    }

    pub fn with_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Create a new actor in the system and in return get an address to talk to the actor. Note
    /// that Romeo tries it's best to keep the actor away from you. Why? Because this ensures that
    /// you are not able to manipulate the internal state of the actor directly, but must communicate
    /// with it through _pure_ semantics.
    ///
    /// Could you manually pass in a channel to your actor instance to bypasss actor-based
    /// communication? Sure you could, but then why are you using actors? `:-P`.
    ///
    /// __Note:__ You the system must be running (see `spawn`) before you can create actors.
    pub fn new_actor<A, P>(&mut self, props: P) -> Address<A>
    where
        A: Actor + ActorConstructable<P> + 'static,
        P: Props + 'static,
    {
        // If we're not running, we can't create actors. Sorry
        if self.state != RunningState::Running {
            panic!(
                "Cannot create actors on a runtime that is not started, currently in state {}",
                self.state
            );
        }

        // Choose a scheduler for the Cell to live on
        let scheduler_index = Rng::gen_range(&mut self.rng, 0, self.thread_schedulers.len());

        // Create the actor-cell
        let producer = Box::new(move || A::new(&props));
        let cell: Arc<Cell<A>> = Cell::new(producer,
                                           Arc::downgrade(&self.thread_schedulers[scheduler_index]));

        // Hand the cell over to a random scheduler
        self.thread_schedulers[scheduler_index].register_new_cell(cell.clone());

        // Return an address (handle to communicate with the actor in the cell)
        Cell::address(cell)
    }

    /// Spawns threads and creates schedulers (the runtime) in order to operate the actor system
    /// on top of. Before calling this method, ensure all desired configurations have been made
    /// via `System::with_config`. This method will not block the current thread.
    pub fn spawn(&mut self) {
        self.state = RunningState::Starting;
        // Spawn threads and schedulers
        for thread_id in 0..self.config.threads {
            let scheduler = Arc::new(Scheduler::new(thread_id));
            self.thread_schedulers.push(scheduler.clone());
            self.thread_handles.push(thread::spawn(move || {
                println!("Spawning scheduler on new thread");
                scheduler.start();
            }));
        }
        self.state = RunningState::Running;
        // TODO: send all schedulers to each other (for reporting / work-stealing)
    }

    /// __Note:__ This currently does nothing but block on the scheduler threads, which will never
    /// exit. This is to-be-implemented.
    pub fn graceful_shutdown(self) {
        // TODO: send some kind of shutdown flag
        // Wait for all threads to stop
        for handle in self.thread_handles {
            // TODO: join returns a result, failing if the thread panic'ed
            match handle.join() {
                Ok(_) => (),
                Err(_) => error!("Thread panicked during graceful shutdown while waiting to join"),
            }
        }
    }
}

/// As the system is more of a handle to the actor runtime, it should be able to be sent across
/// threads. However, it is not sync to avoid race-conditions in starting/stopping and actor creation,
/// as there is an assumed order. Please look into other primitives, such as a `Mutex` if you require
/// this behavior.
unsafe impl Send for System {}

#[derive(PartialEq, Eq)]
enum RunningState {
    AwaitingStart = 0,
    Starting,
    Running,
    Stopping,
    Stopped,
}
impl Display for RunningState {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self == &RunningState::AwaitingStart {
            write!(f, "awaiting-start")
        } else if self == &RunningState::Starting {
            write!(f, "starting")
        } else if self == &RunningState::Running {
            write!(f, "running")
        } else if self == &RunningState::Stopping {
            write!(f, "stopping")
        } else if self == &RunningState::Stopped {
            write!(f, "stopped")
        } else {
            Ok(())
        }
    }
}

/// Config is the configuration for the `System` and can be specified with `System::with_config`.
/// However, it is optional and a default will be provided if not specified manually.
///
/// Values
/// ------
/// + `threads` - The number of threads the system should use. Defaults to the number of logical
///               CPU cores reported by your system.
pub struct Config {
    threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            threads: num_cpus::get(),
        }
    }
}
