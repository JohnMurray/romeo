
# Romeo

[![Crates.io](https://img.shields.io/crates/v/romeo.svg?style=flat-square)](https://crates.io/crates/romeo)

Romeo is an experimental actor framework for Rust. At this point things are really
rough and I'm still getting things ready for eventual use. So go away. `-__-`

## More Information
Still here? Okay, well I'll keep talking. The eventual goal is to create a
distributed actor framework with plug-able behaviors. What do I mean by that?
Well, firstly the goal is to get something that _can_ run on multiple machines
and communicate across them. But once we get to this point, there are a lot of
questions about how the distributed system should be built. What guarantees do
I want to provide? How do I shard? How do I replicate? How do I get consensus
and what do I need consensus on?

This project intends to make all of that plug-able so that user-defined implementations
can be used and experimented with. The goal is to create an actor system that is
customizable/tunable to the end-users desires.

Like I said at the beginning, this is currently just in the beginning phases of me
designing the basic bits and getting everything _working_. Don't expect magic at the
moment, or any real stability if you plan to play around with it. 

## Latest Progress
Actors can be created, addressed, and communicated with (one way). A basic (mostly
useless) blocking runtime exists to execute messages over actors. 

## Currently Working On

The runtime needs to spin up threads to execute the system over and each thread needs
a scheduler to check the inboxes of all actors it is responsible for. 
