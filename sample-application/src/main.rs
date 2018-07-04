extern crate romeo;
use romeo::*;

struct AccountingActor {
    balance: i32,
}
impl Actor for AccountingActor {
    fn start() {
        println!("Starting the actor");
    }
    fn pre_stop() {
        println!("Pre-start hook called");
    }
}
struct AccountingProps {
    starting_balance: i32,
}
impl Props for AccountingProps {}

impl ActorConstructable<AccountingActor, AccountingProps> for AccountingActor {
    fn new(prop: &AccountingProps) -> AccountingActor {
        AccountingActor {
            balance: prop.starting_balance,
        }
    }
}

impl Receives<u8> for AccountingActor {
    fn receive(&mut self, msg: u8) {
        self.balance += msg as i32;
        println!("Current balance: {0}", self.balance);
    }
}


fn main() {
    let runtime = Runtime::new();

    let cell = runtime.new_actor::<AccountingActor, AccountingProps>(AccountingProps { starting_balance: 1 });
    let mut address = Cell::address(cell.clone());
    address.send(32u8); 

    runtime.start();
    // let mut cell = cell.borrow_mut();
    // let queue_fn = cell.msg_queue.pop_front();
    // drop(cell);
    // match queue_fn {
    //     Some(boxed_fn) => boxed_fn(),
    //     _              => ()
    // }
}



// extern crate romeo;
// use std::fmt::{Display, Formatter};
// use std::clone::Clone;
// 
// use romeo::romeo::*;
// 
// struct TestActor {
//     count: u8
// }
// impl Clone for TestActor {
//     fn clone(&self) -> Self {
//         TestActor { count: self.count }
//     }
// }
// impl Display for TestActor {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(f, "TestActor[ count: {0} ]", self.count)
//     }
// }
// impl Props for TestActor {}
// impl ActorConstructable<TestActor> for TestActor {
//     fn new(a: &TestActor) -> TestActor {
//         return (*a).clone()
//     }
// }
// impl Receives<u8> for TestActor {
//     // TODO: I don't have an instance of TestActor here, this is useless
//     fn send(&mut self, msg: u8) {
//         self.count += msg;
//     }
// }
// impl Receives<bool> for TestActor {
//     fn send(&mut self, msg: bool) {
//         self.count += 1;
//     }
// }
// impl Addressable<TestActor> for ActorAddress<TestActor> {
//     fn send<M>(&self, msg: M)
//         where TestActor: Receives<M>,
//               M: Display,
//     {
//         println!("Received {0} for {1}", msg, &self.actor);
//     }
// }
// 
// fn main() {
//     let props = TestActor { count: 5 };
//     let address = ActorAddress { actor: TestActor::new(&props) };
//     let x: u8 = 10;
//     address.send(x);
//     address.send(true);
//     println!("Count: {0}", address.actor.count);
// }
