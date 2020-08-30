use lightning::util::logger::{Logger, Record};

pub struct ConsoleLogger {}

impl ConsoleLogger {
    pub fn new() -> ConsoleLogger {
        ConsoleLogger {}
    }
}

unsafe impl Send for ConsoleLogger {}
unsafe impl Sync for ConsoleLogger {}

impl Logger for ConsoleLogger {
    fn log(&self, record: &Record) {
        println!("{} {:?}", record.level, record.args);
    }
}
