# Romeo

Romeo is an experimental actor framework for Rust. At this point
it is nothing more than a research project for myself. If it
becomes something more serious, I'll update this readme indicating
as much. Until then, no support will be given to anyone who wishes to
use it. Beyond this are notes for myself.

<hr />

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
