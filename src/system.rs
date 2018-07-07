use super::scheduler::Scheduler;

use std::sync::Arc;
use std::thread;

use num_cpus;

/// System is the main handle into a running actor system. It is responsible for creating
/// actors, starting the system (spawning threads and schedulers), stopping the system, etc.
pub struct System {
    thread_handles: Vec<thread::JoinHandle<()>>,
    thread_schedulers: Vec<Arc<Scheduler>>,
    config: Config,
    state: RunningState,
}

impl System {
    pub fn new() -> Self {
        System {
            thread_handles: vec![],
            thread_schedulers: vec![],
            config: Config::default(),
            state: RunningState::AwaitingStart,
        }
    }

    pub fn with_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn spawn(&mut self) {
        for _ in 0..self.config.threads {
            let scheduler = Arc::new(Scheduler::new());
            self.thread_schedulers.push(scheduler.clone());
            self.thread_handles.push(thread::spawn(move || {
                info!("Spawning scheduler on new thread");
                scheduler.start();
            }));
        }
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

enum RunningState {
    AwaitingStart,
    Starting,
    Running,
    Stopping,
    Stopped,
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
