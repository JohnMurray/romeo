use std::sync::mpsc;

trait Printer {
    fn print<M>(&mut self, val: M)
        where Self: Prints<M>;
}

struct SimplePrinter {}

impl Printer for SimplePrinter {
    fn print<M>(&mut self, val: M)
        where Self: Prints<M>,
    {
        self.print(val);
    }
}

trait Prints<U> {
    fn print(&self, val: U);
}

impl Prints<u8> for SimplePrinter {
    fn print(&self, val: u8) {
        println!("u8: {0}", val);
    }
}
impl Prints<bool> for SimplePrinter {
    fn print(&self, val: bool) {
        if val {
            println!("bool: true");
        } else {
            println!("bool: false");
        }
    }
}

/// Need `Message` so that I can store `Box<Message>` as the type
/// in the PrintManager::queue
trait Message {
    type Underlying;
}
impl Message for u8   {
    type Underlying = u8;
}
impl Message for bool {
    type Underlying = bool;
}

struct PrintManager<P, M>
    where P: Printer,
          P: Prints<M>,
          M: Message::Underlying,
{
    printer: P,
    queue: mpsc::Receiver<Box<Message>>,
    writer: mpsc::Sender<Box<Message>>,
}

impl<P> PrintManager<P>
    where P: Printer
{
    pub fn do_print(&mut self) {
        match self.queue.try_recv() {
            Ok(msg) => self.printer.print(msg),
            _       => (),
        }
    }

    fn new (p: P) -> Self {
        let (tx, rx) = mpsc::channel::<Box<Message>>();
        PrintManager {
            printer: p,
            queue: rx,
            writer: tx,
        }
    }

    fn submit_job<M: 'static>(&mut self, msg: M)
        where P: Prints<M>,
              M: Message,
    {
        self.writer.send(Box::new(msg)).unwrap();
    }
}

fn main() {
    let pm = PrintManager::new(SimplePrinter{});
    pm.submit_job(1u8);
    pm.submit_job(false);

    pm.do_print();
    pm.do_print();
}
