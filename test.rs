use std::sync::mpsc;

trait ActorTrait {
    fn print<M>(&mut self, val: M)
        where Self: Prints<M>;
}
struct Actor {}
impl ActorTrait for Actor {
    fn print<M>(&mut self, val: M)
        where Self: Prints<M>,
    {
        self.print(val);
    }
}

trait Prints<U> {
    fn print(&self, val: U);
}

impl Prints<u8> for Actor {
    fn print(&self, val: u8) {
        println!("u8: {0}", val);
    }
}
impl Prints<f64> for Actor {
    fn print(&self, val: f64) {
        println!("f64: {0}", val);
    }
}
impl Prints<bool> for Actor {
    fn print(&self, val: bool) {
        if val {
            println!("bool: true");
        } else {
            println!("bool: false");
        }
    }
}

/// Need `Message` so that I can store `Box<Message>` as the type
/// in the actor queue (mailbox)
trait Message {}
impl Message for u8   {}
impl Message for f64  {}
impl Message for bool {}

struct Runtime<A>
    where A: ActorTrait
{
    actor: A,
    mailbox: mpsc::Receiver<Box<Message>>,
    writer: mpsc::Sender<Box<Message>>,
}

impl<A> Runtime<A>
    where A: ActorTrait
{
    // This is what I like to call "reverse" dynamic dispatch. In that the correct
    // function on _actor_ needs to be called depending on the value-type retrieved
    // from the mailbox (queue).
    // 
    // The problem is, this doesn't work, which means I need to re-think how to dispatch
    // the appropriate receiver for a given message.
    // Ideas:
    //  - pair the receiver as a lambda with the data, or just store the lambda itself,
    //    pre-paired with the data when all the types are known (within the address obj).
    //  - use multiple queues depending on the receivers
    //    -> Global ordering would be the main challenge, and dynamically generating
    //       the queues within the ActorCell.
    pub fn process(&mut self) {
        match self.mailbox.try_recv() {
            Ok(msg) => self.actor.print(msg),
            _       => (),
        }
    }

    fn new (a: A) -> Self {
        let (tx, rx) = mpsc::channel::<Box<Message>>();
        Runtime {
            actor: a,
            mailbox: rx,
            writer: tx,
        }
    }

    fn send<M: 'static>(&mut self, msg: M)
        where A: Prints<M>,
              M: Message,
    {
        self.writer.send(Box::new(msg)).unwrap();
    }
}

fn main() {
    // Cannot infer type for M... so that means that the monomorphization doesn't
    // automatically expand to find all types for M in which `A: Prints<M>` is valid.
    let r = Runtime::new(Actor{});
    // r.send(1u8);
    // r.send(3.04f64);
    // r.send(false);
    // r.process();
}
