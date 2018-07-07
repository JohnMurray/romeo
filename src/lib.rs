#![feature(fnbox)]
#![allow(dead_code)]

#[macro_use]
extern crate log;
extern crate num_cpus;

pub mod actor;
pub mod address;
pub mod cell;
pub mod scheduler;
pub mod system;

pub use actor::{Actor, Receives};
pub use address::Address;
pub use system::System;
