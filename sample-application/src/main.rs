extern crate romeo;
extern crate pretty_env_logger;
use romeo::*;
use romeo::actor::*;

struct AccountingActor {
    balance: i32,
}
impl Actor for AccountingActor {
    fn start(&mut self) {
        println!("Starting, yo!");
    }

    fn pre_stop(&mut self) {
        println!("Stopping, yo!");
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
    fn receive(&mut self, msg: u8, ctx: &Context) {
        self.balance += msg as i32;
        println!("Current balance: {0}", self.balance);
        if msg == 0 {
            println!("attempting to stop actor...");
            ctx.stop();
        } else {
            println!("not stopping actor");
        }
    }
}


fn main() {
    pretty_env_logger::init();
    let mut system = System::new();
    system.spawn();

    let address = system.new_actor::<AccountingActor, AccountingProps>(AccountingProps { starting_balance: 1 });
    address.send(32u8); 
    address.send(0u8);

    // system.graceful_shutdown();
    use std::thread;
    loop { thread::sleep_ms(100); }
}
