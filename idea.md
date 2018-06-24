# Ideas

### Initial Design
For the initial design, I want to be true to the actor model:
  + Asynchronous Message Passing
  + Actor Encapsulation
  + Timeless Semantics
  + State-Changing behavior (including how it handles messages)
  + Location Transparency

The first draft will not focus on:
  + Distribution across > 1 machine
  + Performance

One thing I'm not sure I can get away from in Rust (which is fine) is type-safety. The
balance will be between the type-guarantees and the ease of use (less boilerplate)

Exploration as to what is possible within Rust will be required to determine what is
possible in the API we expose. Ideally I would like to mimic a receive pattern similar
to Erland or Scala Akka:

__Idea #1__

The idea of a `system` and a `context` can be taken from Akka to similar use:

```rust
fn main() {
    let system = romeo::System::new();
    let actor = actor!(system, AccountingActor)
    actor.tell(AccountAction::Deposit(5))

    system.gracefulShutdown()
}
```

This creates a new system, and actor within the system, dispatches a message to the actor,
and then gracefully shuts down the system (waits for all actors to finish processing their
mailboxes). The `actor!` macro would take care of constructing the actor in a way that one
could not get to the instance to manipulate state directly. 

I'm not sure how this would work exactly, but I'm currently thinking the macro would expand
to something like:

```rust
// actor! verifies the type implements the `ActorConstructable` trait
let actor = system.actorNew(AccountingActor::actor_new)

// which might look something like this
impl ActorConstructable<AccountingActor> for AccountingActor {
    pub fn actor_new() -> Box<AccountingActor> {
      // implementation
    }
}
```

The actor itself, would look something like this:

```rust
struct AccountingActor {
    balance: u64
    log: Vec<u32>
}
enum AccountAction {
    Deposit(u32),
    Withdrawl(u32),
}

AccountingActor {
    #[actor_new]
    pub fn new() -> Box<Self> {
      // new fn, that gets converted to ActorConstructable with macro
    }

    #[actor_receive]
    fn receive(&mut self, msg: AccountAction) {
        match msg {
            case AccountAction::Deposit(x)   => self.balance += x
            case AccountAction::Withdrawl(x) => self.balance -= x
        }
    }
}
```

We've already seen that the `new` would generate into an `ActorConstructable` instance,
but the `#[actor_receive]` would have to translate into something like:

```rust
impl Receivable<AccountAction> for AccountActor {
  pub fn receive(msg: AccountAction) {
        match msg {
            case AccountAction::Deposit(x)   => self.balance += x
            case AccountAction::Withdrawl(x) => self.balance -= x
        }
  }
}
```

This would allow an actor to implement everything within a single struct.
Which feels more natural (to me) than breaking different message across different
trait implementations (like above).

For sending messages then, the actor reference would be something like

```rust
struct ActorRef<A, M> {
    // where M is implemented as a Receivable for T
    // M is an unknown size, but pushing everything to the heap, which seems
    // like an ugly solution
    mailbox: Vec<M>
    actor:   A
}
impl ActorRef {
    pub fn send<M>(msg: M) {
        mailbox.push(msg);
    }
}
```

When creating an actor, the system keeps track of all `ActorRef` objects and
returns some sort of weak pointer. When looking up actors, we need a way to give
both a name as well as an interface (how else will the client know what messages
are valid).
