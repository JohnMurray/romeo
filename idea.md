# Ideas

__TOC__

+ Initial Design
+ Rambling Session #1
+ Rambling Session #2
+ Ergonomic Improvements

## Initial Design
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

## Rambling Session #1

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


## Rambling Session #2

When discussing questions of ownership and design, we need to consider where
the actors will _live_. The current ownersihp model looks like

```text
                                                    +-----------+
                                              +-----> Scheduler |
                                              |     +-----------+
                                              |
                                              |     +-----------+
                                              +-----> Scheduler |
                                              |     +-----------+
                                              |
                                              |     +-----------+
+---------------+         +---------------+   +-----> Scheduler |
|               |         |               |   |     +-----------+
| Actor System  +---------> Actor Runtime +---+
|               |         |               |   |     +-----------+
+---------------+         +---------------+   +-----> Scheduler |
                                              |     +-----------+
                                              |
                                              |     +-----------+
                                              +-----> Scheduler |
                                              |     +-----------+
                                              |
                                              |     +-----------+
                                              +-----> Scheduler |
                                                    +-----------+
```

The `ActorSystem` is simply a handle to the `ActorRuntime`, which exists _somewhere_.
The `Scheduler` is responsible for the management of a single thread and is essentially
an actor-specific event-loop. `ActorRuntime` must run on one of the schedulers or on
it's own thread.

Before I can nail down _where_ the actors live within this ownership model, I need to
consider the message passing flow and how addresses _ideally_ work within my actor
system. Addresses need to handle some notion of the contract which the actor defines.
This means knowing what messages are valid and what aren't. Addresses then end up looking
like:

```rust
let address: ActorAddress<SomeActor> = ...
// this would look like:
struct ActorAddress<A> {}

impl ActorAddress<A> {
    pub fn send<M>(msg: M) where A: Receives<M>;
}
```


## Ergonomic Improvements

### #1 - ActorConstructable & Props
I think it's important to capture the ability to create actors
without direct intervention of the user, but I really would hate
if this interface would need to be exposed to the user of the library.

A nicer interface would be:

```rust
struct AccountingActor {
  balance: i64,
}

#[actor_new(construct)]
impl AccountingActor {
  fn construct(balance: i64) -> Self {
    AccountingActor {
      balance
    }
  }
}
```

The `#[actor_new]` annotation accepts a parameter of the function name that
takes any number of parameters and returns an instance of the actor. Typically
the name will likely be `new`, but should be flexible for varying styles.

Creating a new actor would be as simple as:

```rust
let runtime = Runtime::new(/* ... */);
let actor: Address<AcccountingActor> = actor!(runtime, /* balance = */ 0);
```

These two examples would expand to the following code (note that the
fields for `Props` are both defined and constructedin the order declared).

```rust
struct AccountingActor {
  balance: i64,
}
impl AccountingActor {
  // user-defined function still exists, just not used
  fn construct(...) -> Self { ... }
}

// generated code
struct AccountingActorProps {
  balance: i64,
}
impl Props for AccountingActorProps {}
impl ActorConstructable<AccountingActorProps> for AccountingActor {
  fn new(props: &AccountingActorProps) -> Self {
    AccountingActor {
      balance: props.balance,
    }
  }
}

// creation code
let runtime = Runtime::new(/* ... */);
let actor: Address<AccountingActor> = runtime.new_actor<AccountingActor>(AccountingActorProps {
  balance: 0,
});
```

The main question (since I know nothing of macros yet) is weather or not `actor!`
can pull from the declared (expected) type in the variable declaration or will
instead need to be written like:

```rust
actor!(runtime, AccountingActor, 0);
```