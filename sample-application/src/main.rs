extern crate romeo;
use romeo::*;
use romeo::actor::*;

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
    let runtime = Runtime::new();

    let address = runtime.new_actor::<AccountingActor, AccountingProps>(AccountingProps { starting_balance: 1 });
    // let mut address = Cell::address(cell.clone());
    address.send(32u8); 

    runtime.start();
}
