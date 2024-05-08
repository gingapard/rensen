use std::env;
use std::io::{self, Write, BufRead};

use rensen_lib::logging::*;

struct Ctl {}

impl Ctl {

    /// Starts the rensen-cli
    fn start(&mut self) -> Result<(), Trap> {
        let input = self.get_input("rensen").unwrap();

        Ok(())
    }

    /// Prints promt and stdin
    fn get_input(&self, prompt: &str) -> Result<Vec<String>, io::Error> {
        print!("<{}> ", prompt);
        io::stdout().flush()?; 

        let mut buffer = String::new();
        io::stdin().lock().read_line(&mut buffer)?; 

        Ok(buffer.split_whitespace().map(String::from).collect()) // Split input and collect into Vec<String>
    }

}

fn main() {
    let mut ctl = Ctl {};
    let _ = ctl.start();
}
