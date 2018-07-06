// ---
// Base Actor Definition
// ---
pub trait Actor: Send + Sync{
    fn start();
    fn pre_stop();
}

// ---
// Constructor Traits
// ---
pub trait Props {}
pub trait ActorConstructable<P: Props>: Actor {
    fn new(props: &P) -> Self;
}

// ---
// Message Handling
// ---
pub trait Receives<M>
    where Self: Actor,
{
    // TODO: should return something, because need to know about failures
    fn receive(&mut self, msg: M);
}
