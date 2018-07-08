extern crate romeo;
use romeo::*;
use romeo::actor::*;

struct AccountingActor {
    balance: i32,
}
impl Actor for AccountingActor { }
struct AccountingProps {
    starting_balance: i32,
}
impl Props for AccountingProps {}

impl ActorConstructable<AccountingProps> for AccountingActor {
    fn new(prop: &AccountingProps) -> Self {
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
    let mut system = System::new();
    system.spawn();

    let address = system.new_actor::<AccountingActor, AccountingProps>(AccountingProps { starting_balance: 1 });
    address.send(32u8); 

    // system.graceful_shutdown();
    use std::thread;
    loop { thread::sleep_ms(100); }
}
