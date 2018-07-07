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

        // Create the actor-cell
        let producer = Box::new(move || A::new(&props));
        let cell: Arc<Cell<A>> = Cell::new(producer);

        // Hand the cell over to a random scheduler
        let scheduler_index = Rng::gen_range(&mut self.rng, 0, self.thread_schedulers.len());
        self.thread_schedulers[scheduler_index].add_cell(cell.clone());

        // Return an address (handle to communicate with the actor in the cell)
        Cell::address(cell)
    }

    pub fn spawn(&mut self) {
        self.state = RunningState::Starting;
        // Spawn threads and schedulers
        for _ in 0..self.config.threads {
            let scheduler = Arc::new(Scheduler::new());
            self.thread_schedulers.push(scheduler.clone());
            self.thread_handles.push(thread::spawn(move || {
                info!("Spawning scheduler on new thread");
                scheduler.start();
            }));
        }
        self.state = RunningState::Running;
        // TODO: send all schedulers to each other (for reporting / work-stealing)
    }

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
