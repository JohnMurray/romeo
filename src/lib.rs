#![feature(fnbox)]
#![allow(dead_code)]

extern crate crossbeam_channel;
#[macro_use]
extern crate log;
extern crate num_cpus;
extern crate rand;
extern crate uuid;

pub mod actor;
pub mod address;
pub mod cell;
pub mod scheduler;
pub mod system;

pub use actor::{Actor, Receives};
pub use address::Address;
pub use system::System;
